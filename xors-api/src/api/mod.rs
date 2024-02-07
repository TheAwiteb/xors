// A RESTful tic tac toy API for XORS project
// Copyright (C) 2024  Awiteb <Awiteb@pm.me>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use salvo::http::ResBody;
use salvo::hyper::header::HeaderName;
use salvo::jwt_auth::{ConstDecoder, HeaderFinder};
use salvo::oapi::security::{Http, HttpAuthScheme};
use salvo::oapi::{Info, License, SecurityScheme};
use salvo::rate_limiter::*;
use salvo::{catcher::Catcher, http::HeaderValue, hyper::header, logging::Logger, prelude::*};
use salvo_captcha::*;

use crate::schemas::MessageSchema;

pub mod exts;
pub mod game;
pub mod jwt;
pub mod user;
pub mod xo;

pub fn write_json_body(res: &mut Response, json_body: impl serde::Serialize) {
    res.write_body(serde_json::to_string(&json_body).unwrap())
        .ok();
}

#[handler]
async fn handle404(res: &mut Response, ctrl: &mut FlowCtrl) {
    if let Some(StatusCode::NOT_FOUND) = res.status_code {
        write_json_body(res, MessageSchema::new("Not Found".to_owned()));
        ctrl.skip_rest();
    }
}

#[handler]
async fn handle_server_errors(res: &mut Response, ctrl: &mut FlowCtrl) {
    log::info!("New response catched: {res:#?}");
    if matches!(res.status_code, Some(status) if !status.is_success()) {
        if res.status_code == Some(StatusCode::TOO_MANY_REQUESTS) {
            write_json_body(
                res,
                MessageSchema::new("Too many requests, please try again later".to_owned()),
            );
            ctrl.skip_rest();
        } else if let ResBody::Error(err) = &res.body {
            log::error!("Error: {err}");
            write_json_body(
                res,
                MessageSchema::new(format!(
                    "{}, {}: {}",
                    err.name,
                    err.brief.trim_end_matches('.'),
                    err.cause
                        .as_deref()
                        .map_or_else(|| "".to_owned(), ToString::to_string)
                        .trim_end_matches('.')
                        .split(':')
                        .last()
                        .unwrap_or_default()
                        .trim()
                )),
            );
            ctrl.skip_rest();
        } else {
            log::warn!("Unknown error uncatched: {res:#?}");
        }
    } else {
        log::warn!("Unknown response uncatched: {res:#?}");
    }
}

#[handler]
async fn add_server_headers(res: &mut Response) {
    let headers = res.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    // Yeah, Rusty programmer
    headers.insert("X-Powered-By", HeaderValue::from_static("Rust/Salvo"));
}

pub fn service(
    conn: sea_orm::DatabaseConnection,
    max_online_games: usize,
    move_period: i64,
    secret_key: String,
) -> (Service, OpenApi) {
    let auth_handler: JwtAuth<jwt::JwtClaims, _> =
        JwtAuth::new(ConstDecoder::from_secret(secret_key.as_bytes()))
            .finders(vec![Box::new(
                HeaderFinder::new().header_names(vec![header::AUTHORIZATION]),
            )])
            .force_passed(false);
    let captcha_middleware = Captcha::<CacacheStorage, CaptchaHeaderFinder<String, String>>::new(
        CacacheStorage::new("chapcha_cache"),
        CaptchaHeaderFinder::new()
            .token_header(HeaderName::from_str("X-Captcha-Token").expect("Is valid header name"))
            .answer_header(HeaderName::from_str("X-Captcha-Answer").expect("Is valid header name")),
    )
    .skipper(|_: &mut Request, _: &Depot| {
        // Skip the captcha middleware if we are in the test environment
        // The captcha logic is tested in the `salvo_captcha` crate
        if matches!(env::var("XORS_API_TEST"), Ok(val) if val == "true") {
            return true;
        }
        false
    });

    let captcha_storage = Arc::new(captcha_middleware.storage().clone());

    let unauth_limiter = RateLimiter::new(
        SlidingGuard::new(),
        MokaStore::<String, SlidingGuard>::new(),
        RemoteIpIssuer,
        CelledQuota::per_minute(30, 1),
    )
    .add_headers(true);
    let auth_limiter = RateLimiter::new(
        FixedGuard::new(),
        MokaStore::new(),
        RemoteIpIssuer,
        BasicQuota::per_minute(1500),
    )
    .add_headers(true);

    let router = Router::new()
        .hoop(Logger::new())
        .hoop(
            affix::inject(Arc::new(conn))
                .inject(captcha_storage.clone())
                .insert("secret_key", Arc::new(secret_key))
                .insert("max_online_games", Arc::new(max_online_games))
                .insert("move_period", Arc::new(move_period)),
        )
        // Unauthorized routes
        .push(
            Router::new()
                .hoop(unauth_limiter)
                .hoop(add_server_headers)
                .push(
                    Router::with_path("auth")
                        .push(
                            Router::with_hoop(captcha_middleware)
                                .push(Router::with_path("signup").post(jwt::signup))
                                .push(Router::with_path("signin").post(jwt::signin)),
                        )
                        .push(Router::with_path("captcha").get(jwt::captcha)),
                )
                .push(Router::with_path("user").get(user::get_user_info))
                .push(Router::with_path("profiles/<uuid>").get(user::get_user_profile_image))
                .push(Router::with_path("game/<uuid>").get(game::get_game_by_uuid))
                .push(Router::with_path("games").get(game::get_lastest_games)),
        )
        // Authorized routes
        .push(
            Router::new()
                .hoop(auth_limiter)
                .hoop(auth_handler)
                .hoop(add_server_headers)
                .push(
                    Router::with_path("auth").push(Router::with_path("refresh").get(jwt::refresh)),
                )
                .push(
                    Router::with_path("user")
                        .put(user::update_user)
                        .delete(user::delete_user)
                        .push(Router::with_path("reset_password").post(user::reset_user_password))
                        .push(Router::with_path("me").get(user::get_me)),
                )
                .push(Router::with_path("xo").goal(xo::user_connected)),
        );

    let openapi = OpenApi::new("XORS API", env!("CARGO_PKG_VERSION"))
        .info(
            Info::new("XORS API", env!("CARGO_PKG_VERSION"))
                .description(include_str!("../../api_des.md"))
                .license(
                    License::new("AGPL-3.0-or-later")
                        .url("https://www.gnu.org/licenses/agpl-3.0.en.html"),
                ),
        )
        .add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                Http::new(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(
                        "The JWT token of the user. Get it from the `/auth/signin` endpoint.",
                    ),
            ),
        )
        .merge_router(&router);

    let router = router
        .unshift(openapi.clone().into_router("/api-doc/openapi.json"))
        .unshift(
            SwaggerUi::new("/api-doc/openapi.json")
                .title("XORS - Swagger Ui")
                .description("XORS is a simple API of XO game, supports multiplayer")
                .keywords("XORS, XO, Game, Multiplayer, API, Rust, Salvo, SeaORM")
                .into_router("/api-doc/swagger-ui"),
        );

    tokio::spawn({
        log::info!("Start the captcha cleaner...");
        let cleanner_storage = captcha_storage.clone();
        async move {
            let captcha_expired_after = Duration::from_secs(60 * 5);
            let clean_interval = Duration::from_secs(60);

            loop {
                if let Err(err) = cleanner_storage.clear_expired(captcha_expired_after).await {
                    log::error!("Failed to clean captcha storage: {err}")
                }
                tokio::time::sleep(clean_interval).await;
            }
        }
    });

    (
        Service::new(router).catcher(
            Catcher::default()
                .hoop(handle404)
                .hoop(handle_server_errors),
        ),
        openapi,
    )
}
