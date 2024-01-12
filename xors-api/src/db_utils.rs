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

use crate::api::jwt::JwtClaims;
use crate::errors::{ApiError, ApiResult};
use crate::schemas::*;
use entity::prelude::*;
use jsonwebtoken::Header;
use uuid::Uuid;

/// Creates a new user in the database.
pub async fn create_user(
    conn: &sea_orm::DatabaseConnection,
    new_user: NewUserSchema,
) -> ApiResult<UserSchema> {
    log::info!("Creating user: {}", new_user.username);

    if UserEntity::find()
        .filter(UserColumn::Username.eq(new_user.username.clone()))
        .count(conn)
        .await?
        != 0
    {
        log::error!("Username already exists: {}", new_user.username);
        Err(ApiError::UsernameAlreadyExists(new_user.username))
    } else {
        log::info!("Hashing password for user: {}", new_user.username);
        let password_hash = bcrypt::hash(&new_user.password, 4)?;

        log::info!("Getting a new uuid for user: {}", new_user.username);
        let uuid = loop {
            let uuid = Uuid::new_v4();
            if UserEntity::find()
                .filter(UserColumn::Uuid.eq(uuid))
                .count(conn)
                .await?
                == 0
            {
                break uuid;
            }
        };
        log::info!("New uuid for user: {}", new_user.username);

        Ok(UserActiveModel {
            uuid: Set(uuid),
            first_name: Set(new_user.first_name),
            last_name: Set(new_user.last_name),
            profile_image_url: Set(format!(
                "https://api.dicebear.com/7.x/thumbs/svg?seed={}",
                &new_user.username
            )),
            username: Set(new_user.username),
            password_hash: Set(password_hash),
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        }
        .save(conn)
        .await?
        .into())
    }
}

/// Signin a user and returns a JWT token with a refresh token.
pub async fn signin_user(user: UserSchema, secret_key: &str) -> ApiResult<UserSigninSchema> {
    log::info!("Logging in user: {}", user.username);

    let jwt = jsonwebtoken::encode(
        &Header::default(),
        &JwtClaims::new(
            user.uuid,
            None,
            (chrono::Utc::now().naive_utc() + chrono::Duration::hours(1)).timestamp(),
        ),
        &jsonwebtoken::EncodingKey::from_secret(secret_key.as_bytes()),
    )
    .expect("JWT encode failed");

    let refresh_token = jsonwebtoken::encode(
        &Header::default(),
        &JwtClaims::new(
            user.uuid,
            Some((chrono::Utc::now().naive_utc() + chrono::Duration::minutes(58)).timestamp()),
            (chrono::Utc::now().naive_utc() + chrono::Duration::hours(3)).timestamp(),
        ),
        &jsonwebtoken::EncodingKey::from_secret(secret_key.as_bytes()),
    )
    .expect("JWT encode failed");

    Ok(UserSigninSchema {
        user,
        jwt,
        refresh_token,
    })
}

/// Get a user from the database by uuid.
pub async fn get_user(
    conn: &sea_orm::DatabaseConnection,
    uuid: Uuid,
) -> ApiResult<UserActiveModel> {
    log::info!("Getting user by uuid");

    UserEntity::find()
        .filter(UserColumn::Uuid.eq(uuid))
        .one(conn)
        .await?
        .map(|u| u.into_active_model())
        .ok_or(ApiError::UserNotFound)
}

/// Get a user from the database by username.
pub async fn get_user_by_username(
    conn: &sea_orm::DatabaseConnection,
    username: String,
) -> ApiResult<UserActiveModel> {
    log::info!("Getting user by username");

    UserEntity::find()
        .filter(UserColumn::Username.eq(username))
        .one(conn)
        .await?
        .map(|u| u.into_active_model())
        .ok_or(ApiError::UserNotFound)
}

/// Creates a new captcha in the database with the given answer.
pub async fn create_captcha(
    conn: &sea_orm::DatabaseConnection,
    answer: String,
) -> ApiResult<CaptchaActiveModel> {
    log::info!("Creating captcha");

    let uuid = loop {
        let uuid = Uuid::new_v4();
        if CaptchaEntity::find()
            .filter(CaptchaColumn::Uuid.eq(uuid))
            .count(conn)
            .await?
            == 0
        {
            break uuid;
        }
    };

    Ok(CaptchaActiveModel {
        uuid: Set(uuid),
        answer: Set(answer),
        expired_at: Set(chrono::Utc::now().naive_utc() + chrono::Duration::minutes(5)),
        ..Default::default()
    }
    .save(conn)
    .await?)
}
