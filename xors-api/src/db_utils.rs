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

use crate::api::jwt::JwtClaims;
use crate::errors::{ApiError, ApiResult};
use crate::schemas::*;
use chrono::Duration;
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
    let now = chrono::Utc::now().naive_utc();
    let jwt_exp = if matches!(std::env::var("XORS_API_TEST"), Ok(status) if status == "true") {
        (now + chrono::Duration::seconds(2)).timestamp()
    } else {
        (now + chrono::Duration::hours(1)).timestamp()
    };

    let refresh_exp = if matches!(std::env::var("XORS_API_TEST"), Ok(status) if status == "true") {
        (now + chrono::Duration::seconds(5)).timestamp()
    } else {
        (now + chrono::Duration::hours(3)).timestamp()
    };

    let refresh_active_after = if matches!(std::env::var("XORS_API_TEST"), Ok(status) if status == "true")
    {
        Some((now + chrono::Duration::seconds(3)).timestamp())
    } else {
        Some((now + chrono::Duration::minutes(58)).timestamp())
    };

    let jwt = jsonwebtoken::encode(
        &Header::default(),
        &JwtClaims::new(user.uuid, None, jwt_exp),
        &jsonwebtoken::EncodingKey::from_secret(secret_key.as_bytes()),
    )
    .expect("JWT encode failed");

    let refresh_token = jsonwebtoken::encode(
        &Header::default(),
        &JwtClaims::new(user.uuid, refresh_active_after, refresh_exp),
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

/// End a game in the database. This will set the `ended_at` column to the current time and remove the `board` column.
pub async fn end_game(conn: &sea_orm::DatabaseConnection, game_uuid: &Uuid) -> ApiResult<()> {
    log::info!("Ending game: {}", game_uuid);

    if let Some(mut game) = GameEntity::find()
        .filter(GameColumn::Uuid.eq(*game_uuid))
        .one(conn)
        .await?
        .map(IntoActiveModel::into_active_model)
    {
        game.ended_at = Set(Some(chrono::Utc::now().naive_utc()));
        game.board = Set(String::new());
        game.save(conn).await?;
    }

    Ok(())
}

/// Create a new game in the database.
pub async fn create_game(
    conn: &sea_orm::DatabaseConnection,
    x_player: Uuid,
    o_player: Uuid,
    move_period: i64,
) -> ApiResult<GameActiveModel> {
    log::info!("Creating game");

    let uuid = loop {
        let uuid = Uuid::new_v4();
        if GameEntity::find()
            .filter(GameColumn::Uuid.eq(uuid))
            .count(conn)
            .await?
            == 0
        {
            break uuid;
        }
    };

    let now = chrono::Utc::now().naive_utc();
    Ok(GameActiveModel {
        uuid: Set(uuid),
        round: Set(1i16),
        auto_play_after: Set(Some(now + Duration::seconds(move_period))),
        rounds_result: Set(RoundsResult::default().to_string()),
        x_player: Set(x_player),
        o_player: Set(o_player),
        board: Set(Board::default().to_string()),
        winner: Set(None),
        reason: Set(None),
        created_at: Set(now),
        ..Default::default()
    }
    .save(conn)
    .await?)
}

/// Get a game from the database by uuid.
///
/// **Note**: This will return the game only if it's ended.
pub async fn get_game(
    conn: &sea_orm::DatabaseConnection,
    game_uuid: &Uuid,
) -> ApiResult<GameModel> {
    log::info!("Getting game: {}", game_uuid);

    GameEntity::find()
        .filter(
            GameColumn::Uuid
                .eq(*game_uuid)
                .and(GameColumn::EndedAt.is_not_null()),
        )
        .one(conn)
        .await?
        .ok_or(ApiError::GameNotFound)
}

/// Returns lastest 10 games from the database.
pub async fn get_lastest_games(conn: &sea_orm::DatabaseConnection) -> ApiResult<Vec<GameModel>> {
    log::info!("Getting lastest games");

    Ok(GameEntity::find()
        .filter(GameColumn::EndedAt.is_not_null())
        .order_by(GameColumn::CreatedAt, Order::Desc)
        .limit(10)
        .all(conn)
        .await?)
}

/// Returns online games from the database.
pub async fn get_online_games(conn: &sea_orm::DatabaseConnection) -> ApiResult<Vec<GameModel>> {
    log::info!("Getting online games");

    Ok(GameEntity::find()
        .filter(GameColumn::EndedAt.is_null())
        .order_by(GameColumn::CreatedAt, Order::Desc)
        .all(conn)
        .await?)
}
