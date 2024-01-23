// A API for xors (XO game)
// Copyright (C) 2024  Awiteb <awitb@hotmail.com>
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

use std::sync::Arc;

use salvo::http::ResBody;
use salvo::jwt_auth::{ConstDecoder, HeaderFinder};
use salvo::oapi::security::{Http, HttpAuthScheme};
use salvo::oapi::{Info, License, SecurityScheme};
use salvo::rate_limiter::*;
use salvo::{catcher::Catcher, http::HeaderValue, hyper::header, logging::Logger, prelude::*};

use crate::schemas::MessageSchema;

pub mod exts;
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
) -> Service {
    let auth_handler: JwtAuth<jwt::JwtClaims, _> =
        JwtAuth::new(ConstDecoder::from_secret(secret_key.as_bytes()))
            .finders(vec![Box::new(
                HeaderFinder::new().header_names(vec![header::AUTHORIZATION]),
            )])
            .force_passed(false);

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
                        .push(Router::with_path("signup").post(jwt::signup))
                        .push(Router::with_path("signin").post(jwt::signin))
                        .push(Router::with_path("captcha").get(jwt::captcha)),
                )
                .push(Router::with_path("user").get(user::get_user_info)),
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
                        .push(Router::with_path("me").get(user::get_me)),
                )
                .push(Router::with_path("xo").goal(xo::user_connected)),
        );

    let doc = OpenApi::new("XORS API", env!("CARGO_PKG_VERSION"))
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
        .unshift(doc.into_router("/api-doc/openapi.json"))
        .unshift(
            SwaggerUi::new("/api-doc/openapi.json")
                .title("XORS - Swagger Ui")
                .description("XORS is a simple API of XO game, supports multiplayer")
                .keywords("XORS, XO, Game, Multiplayer, API, Rust, Salvo, SeaORM")
                .into_router("/api-doc/swagger-ui"),
        );

    Service::new(router).catcher(
        Catcher::default()
            .hoop(handle404)
            .hoop(handle_server_errors),
    )
}
