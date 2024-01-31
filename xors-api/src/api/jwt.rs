// A RESTful tic tac toy API for XORS project
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

use std::{str::FromStr, sync::Arc};

use crate::{
    db_utils,
    errors::{ApiError, ApiResult},
    schemas::*,
    utils,
};

use base64::Engine;
use chrono::Duration;
use salvo::{oapi::extract::JsonBody, prelude::*};
use salvo_captcha::{
    CacacheStorage, CaptchaDepotExt, CaptchaDifficulty, CaptchaGenerator, CaptchaName,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::exts::UserExt;

#[derive(Debug, Serialize, Deserialize, derive_new::new)]
pub struct JwtClaims {
    /// The user's uuid.
    pub uuid: Uuid,
    /// The refresh token activate date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_after: Option<i64>,
    /// The token's expiration date.
    pub exp: i64,
}

impl JwtClaims {
    /// Returns whether if the jwt is a refresh token or not.
    pub fn is_refresh_token(&self) -> bool {
        self.active_after.is_some()
    }

    /// Returns whether if the token is expired or not.
    pub fn is_expired(&self) -> bool {
        self.exp <= chrono::Utc::now().timestamp()
    }
}

/// Create a new captcha.
///
/// This endpoint will create a new captcha and return the captcha token and the captcha image as base64.
/// - The token are valid for 5 minutes.
/// - The token can only be used one time correctly.
/// - If the token is used incorrectly, it will not be deleted and can be used again until it expires.
#[endpoint(
    operation_id = "create_captcha",
    tags("Auth"),
    responses(
        (status_code = 200, description = "Captcha created", content_type = "application/json", body = CaptchaSchema),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    )
)]
pub async fn captcha(depot: &mut Depot) -> ApiResult<Json<CaptchaSchema>> {
    let captcha_storage = depot.obtain::<Arc<CacacheStorage>>().unwrap();
    let (captcha_token, captcha_image) = captcha_storage
        .new_captcha(CaptchaName::Amelia, CaptchaDifficulty::Medium)
        .await
        .map_err(|_| ApiError::InternalServer)?
        .ok_or(ApiError::InternalServer)?;

    Ok(Json(CaptchaSchema {
        captcha_token: Uuid::from_str(&captcha_token)
            .expect("The cacache storage should return a valid uuid"),
        captcha_image: format!(
            "data:image/png;base64,{}",
            crate::BASE_64_ENGINE.encode(captcha_image)
        ),
        expired_at: chrono::Utc::now().naive_utc() + Duration::minutes(5),
    }))
}

/// Sign up a new user.
///
/// This endpoint will create a new user and return a JWT token.
/// - `captcha_token`: The captcha token. Get it from the `/auth/captcha` endpoint.
/// - `captcha_answer`: The captcha answer. The text that in the captcha image.
#[endpoint(
    operation_id = "signup_user",
    tags("Auth"),
    request_body(
        content = NewUserSchema,
        description = "New user data",
        example = json!(NewUserSchema::default()),
        content_type = "application/json",
    ),
    parameters(
        ("X-Captcha-Token" = String, Header, description = "The captcha token, get it from the `/auth/captcha` endpoint"),
        ("X-Captcha-Answer" = String, Header, description = "The captcha answer, the text that in the captcha image"),
    ),
    responses(
        (status_code = 200, description = "User created", content_type = "application/json", body = UserSigninSchema),
        (status_code = 400, description = "Username already exists", content_type = "application/json", body = MessageSchema),
        (status_code = 400, description = "Unprovided captcha token or answer", content_type = "application/json", body = MessageSchema),
        (status_code = 403, description = "Invalid captcha token", content_type = "application/json", body = MessageSchema),
        (status_code = 403, description = "Invalid captcha answer", content_type = "application/json", body = MessageSchema),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    )
)]
pub async fn signup(
    depot: &mut Depot,
    new_user: JsonBody<NewUserSchema>,
) -> ApiResult<Json<UserSigninSchema>> {
    let conn = depot.obtain::<Arc<sea_orm::DatabaseConnection>>().unwrap();
    let secret_key = depot.get::<Arc<String>>("secret_key").unwrap();
    let user = new_user.into_inner();

    utils::handle_captcha_state(
        depot
            .get_captcha_state()
            .expect("This route is protected by the `salvo_captcha` middleware"),
    )?;
    utils::validate_user_registration(&user)?;

    db_utils::signin_user(
        db_utils::create_user(conn.as_ref(), user).await?,
        secret_key,
    )
    .await
    .map(Json)
}

/// Signin a user.
///
/// This endpoint will return a JWT token with a refresh token.
#[endpoint(
    operation_id = "signin_user",
    tags("Auth"),
    request_body(
        content = SigninSchema,
        description = "User signin data",
        example = json!(SigninSchema::default()),
        content_type = "application/json",
    ),
    parameters(
        ("X-Captcha-Token" = String, Header, description = "The captcha token, get it from the `/auth/captcha` endpoint"),
        ("X-Captcha-Answer" = String, Header, description = "The captcha answer, the text that in the captcha image"),
    ),
    responses(
        (status_code = 200, description = "User signed in", content_type = "application/json", body = UserSigninSchema),
        (status_code = 400, description = "Invalid username or password", content_type = "application/json", body = MessageSchema),
        (status_code = 400, description = "Unprovided captcha token or answer", content_type = "application/json", body = MessageSchema),
        (status_code = 403, description = "Invalid captcha token", content_type = "application/json", body = MessageSchema),
        (status_code = 403, description = "Invalid captcha answer", content_type = "application/json", body = MessageSchema),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    )
)]
pub async fn signin(
    depot: &mut Depot,
    signin_schema: JsonBody<SigninSchema>,
) -> ApiResult<Json<UserSigninSchema>> {
    let conn = depot.obtain::<Arc<sea_orm::DatabaseConnection>>().unwrap();
    let secret_key = depot.get::<Arc<String>>("secret_key").unwrap();
    let signin_schema = signin_schema.into_inner();

    utils::handle_captcha_state(
        depot
            .get_captcha_state()
            .expect("This route is protected by the `salvo_captcha` middleware"),
    )?;

    utils::validate_password(&signin_schema.password)?;
    utils::validate_user_signin(&signin_schema.username)?;

    if let Ok(user) = db_utils::get_user_by_username(conn.as_ref(), signin_schema.username).await {
        if bcrypt::verify(&signin_schema.password, user.password_hash.as_ref()).unwrap_or_default()
        {
            return db_utils::signin_user(user.into(), secret_key)
                .await
                .map(Json);
        }
    }
    Err(ApiError::InvalidSigninCredentials)
}

/// Refresh a JWT token.
///
/// This endpoint will return a new JWT token with the refresh token.
/// Note: You need to authorize with the refresh token to get a new JWT token.
#[endpoint(
    operation_id = "refresh_token",
    tags("Auth"),
    responses(
        (status_code = 200, description = "JWT token refreshed", content_type = "application/json", body = UserSigninSchema),
        (status_code = 400, description = "The token is not a refresh token", content_type = "application/json", body = MessageSchema),
        (status_code = 403, description = "The refresh token is not active yet", content_type = "application/json", body = MessageSchema),
        (status_code = 401, description = "The token is expired", content_type = "application/json", body = MessageSchema),
        (status_code = 401, description = "Unauthorized, missing JWT", content_type = "application/json", body = MessageSchema),
        (status_code = 404, description = "User not found", content_type = "application/json", body = MessageSchema),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn refresh(depot: &mut Depot) -> ApiResult<Json<UserSigninSchema>> {
    let conn = depot.obtain::<Arc<sea_orm::DatabaseConnection>>().unwrap();
    let secret_key = depot.get::<Arc<String>>("secret_key").unwrap();

    // Note: The `Unauthorized` and `Forbidden` errors are handled by the `JwtAuth` middleware.
    let refresh_token = depot.jwt_claims();
    if let Some(active_after) = refresh_token.active_after {
        if !refresh_token.is_expired() {
            if active_after <= chrono::Utc::now().timestamp() {
                db_utils::signin_user(
                    db_utils::get_user(conn.as_ref(), refresh_token.uuid)
                        .await?
                        .into(),
                    secret_key,
                )
                .await
                .map(Json)
            } else {
                Err(ApiError::UnActiveRefreshToken)
            }
        } else {
            Err(ApiError::ExpiredToken)
        }
    } else {
        Err(ApiError::NotRefreshToken)
    }
}
