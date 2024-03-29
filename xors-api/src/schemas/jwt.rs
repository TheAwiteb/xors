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

use entity::prelude::*;
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db_utils, errors::ApiResult};

/// The user's schema. It's used to return the user's data.
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, ToSchema)]
#[salvo(schema(symbol = "UserSchema", example = json!(UserSchema::default())))]
pub struct UserSchema {
    /// The user's uuid. It's unique.
    pub uuid: Uuid,
    /// The user's first name.
    pub first_name: String,
    /// The user's last name.
    pub last_name: Option<String>,
    /// The user's username. It's unique.
    pub username: String,
    /// The user's profile image endpoint path.
    pub profile_image_path: String,
    /// The user's wins games.
    pub wins: i64,
    /// The user's losts games.
    pub losts: i64,
    /// The user's draw games.
    pub draw: i64,
    /// The last 10 games the player has played.
    pub latest_games: Vec<Uuid>,
    /// The user's creation date. Joined date.
    pub created_at: chrono::NaiveDateTime,
}

/// The new user's schema. It's used to create a new user.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[salvo(schema(symbol = "NewUserSchema", example = json!(NewUserSchema::default())))]
pub struct NewUserSchema {
    /// The user's first name. Can't contain spaces.
    #[salvo(schema(min_length = 1, max_length = 32))]
    pub first_name: String,
    /// The user's last name. Can't contain spaces.
    #[salvo(schema(min_length = 1, max_length = 32))]
    pub last_name: Option<String>,
    /// The user's username. It must be unique and start with a letter.
    /// It can only contain English letters, numbers, and underscores.
    #[salvo(schema(min_length = 3, max_length = 32))]
    pub username: String,
    /// The user's password.
    /// - It must be between 8 and 64 characters.
    /// - It can't contain spaces.
    /// - It must contain at least one uppercase and lowercase letter.
    /// - It must contain at least one number.
    /// - It must contain at least one symbol.
    /// - It can't be a common password.
    #[salvo(schema(min_length = 8, max_length = 64))]
    pub password: String,
}

/// The signin schema. It's used to signin a user.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[salvo(schema(symbol = "SigninSchema", example = json!(SigninSchema::default())))]
pub struct SigninSchema {
    /// The user's username.
    #[salvo(schema(min_length = 3, max_length = 32))]
    pub username: String,
    /// The user's password.
    #[salvo(schema(min_length = 8, max_length = 64))]
    pub password: String,
}

/// The user's signin schema. It's used to return the user's data and the JWT token.
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, ToSchema)]
#[salvo(schema(symbol = "UserSigninSchema", example = json!(UserSigninSchema::default())))]
pub struct UserSigninSchema {
    #[serde(flatten)]
    pub user: UserSchema,
    /// The JWT token. It must be sent in the `Authorization` header.
    /// Will expire in 1 hour.
    pub jwt: String,
    /// The refresh token. It must be sent in the `Authorization` header.
    /// Will be available after 58 minutes and will expire in 3 hours.
    pub refresh_token: String,
}

/// The captcha schema. It's used to return the captcha token and the captcha image.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[salvo(schema(symbol = "CaptchaSchema", example = json!(CaptchaSchema::default())))]
pub struct CaptchaSchema {
    /// The captcha token. It's used to verify that the user is not a robot.
    pub captcha_token: Uuid,
    /// The captcha image. It's a base64 string.
    pub captcha_image: String,
    /// The expiration date of the captcha token.
    pub expired_at: chrono::NaiveDateTime,
}

impl UserSchema {
    /// Returns a deleted user.
    ///
    /// This only used when the game's player is deleted, so we can still return the game's data. (It's not saving the real player's data)
    pub(crate) fn deleted_user() -> Self {
        let nil_uuid = Uuid::nil();
        Self {
            uuid: nil_uuid,
            first_name: "Deleted".to_owned(),
            last_name: Some("Player".to_owned()),
            username: "Deleted".to_owned(),
            profile_image_path: format!("profiles/{nil_uuid}"),
            wins: 0,
            losts: 0,
            draw: 0,
            latest_games: Vec::new(),
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    /// Create new [`UserSchema`] instance from [`UserActiveModel`]
    pub async fn from_active_model(
        conn: &sea_orm::DatabaseConnection,
        user: UserActiveModel,
    ) -> ApiResult<Self> {
        let user_uuid = user.uuid.as_ref();

        Ok(Self {
            uuid: *user_uuid,
            first_name: user.first_name.unwrap(),
            last_name: user.last_name.unwrap(),
            username: user.username.unwrap(),
            profile_image_path: user.profile_image_path.unwrap(),
            wins: user.wins.unwrap(),
            losts: user.losts.unwrap(),
            draw: user.draw.unwrap(),
            latest_games: db_utils::latest_player_games(conn, user_uuid)
                .await?
                .into_iter()
                .map(|g| g.uuid)
                .collect(),
            created_at: user.created_at.unwrap(),
        })
    }
}

impl Default for UserSchema {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "Username".to_owned(),
            profile_image_path: "/profiles/default".to_owned(),
            wins: 0,
            losts: 0,
            draw: 0,
            latest_games: vec![Uuid::new_v4()],
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

impl Default for NewUserSchema {
    fn default() -> Self {
        Self {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "Username".to_owned(),
            password: "Password".to_owned(),
        }
    }
}

impl Default for UserSigninSchema {
    fn default() -> Self {
        Self {
            user: UserSchema::default(),
            jwt: "<JWT>".to_owned(),
            refresh_token: "<REFRESH_TOKEN>".to_owned(),
        }
    }
}

impl Default for SigninSchema {
    fn default() -> Self {
        Self {
            username: "Username".to_owned(),
            password: "Password".to_owned(),
        }
    }
}

impl Default for CaptchaSchema {
    fn default() -> Self {
        Self {
            captcha_token: Uuid::new_v4(),
            captcha_image: "<CAPTCHA_IMAGE_BASE64>".to_owned(),
            expired_at: chrono::Utc::now().naive_utc(),
        }
    }
}

impl PartialEq<NewUserSchema> for UserSchema {
    fn eq(&self, other: &NewUserSchema) -> bool {
        self.first_name == other.first_name
            && self.last_name == other.last_name
            && self.username == other.username
    }
}
