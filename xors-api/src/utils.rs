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

use entity::prelude::*;
use passwords::{analyzer, scorer};
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    schemas::NewUserSchema,
};

/// Validates a user name. not the user signin, but the first name and last name.
#[must_use = "This function returns a `ApiResult<()>` instead of panicking"]
pub fn validate_user_name(name: &str) -> ApiResult<()> {
    if name.chars().count() > 32
        || name.chars().count() == 0
        || name.chars().any(|c: char| c.is_whitespace())
    {
        return Err(ApiError::InvalidFirstName);
    }
    Ok(())
}

/// Validates the user's password.
///
/// This will be check for:
/// - The password's length. It must be between 8 and 64 characters.
/// - The password's spaces. It can't contain spaces.
/// - The password's uppercase and lowercase letters. It must contain at least one uppercase and lowercase letter.
/// - The password's numbers. It must contain at least one number.
/// - The password's symbols. It must contain at least one symbol.
/// - The password's commonness. It can't be a common password.
/// - The password's strength. It must be at least 80% strong.
#[must_use = "This function returns a `ApiResult<()>` instead of panicking"]
pub fn validate_password(password: &str) -> ApiResult<()> {
    let password_analyzer = analyzer::analyze(password);

    if password.chars().count() > 64 || password.chars().count() < 8 {
        return Err(ApiError::InvalidPassword(
            "The password must be between 8 and 64 characters".to_owned(),
        ));
    }
    if password.chars().any(|c: char| c.is_whitespace()) {
        return Err(ApiError::InvalidPassword(
            "The password contains spaces".to_owned(),
        ));
    }
    if password_analyzer.uppercase_letters_count() == 0
        || password_analyzer.lowercase_letters_count() == 0
    {
        return Err(ApiError::InvalidPassword(
            "The password must contain at least one uppercase and lowercase letter".to_owned(),
        ));
    }
    if password_analyzer.numbers_count() == 0 {
        return Err(ApiError::InvalidPassword(
            "The password must contain at least one number".to_owned(),
        ));
    }
    if password_analyzer.symbols_count() == 0 {
        return Err(ApiError::InvalidPassword(
            "The password must contain at least one symbol".to_owned(),
        ));
    }
    if password_analyzer.is_common() {
        return Err(ApiError::InvalidPassword(
            "The password is common".to_owned(),
        ));
    }
    if scorer::score(&password_analyzer) < 80.0 {
        return Err(ApiError::InvalidPassword(
            "The password is too weak".to_owned(),
        ));
    }

    Ok(())
}

/// Validates a user signin.
///
/// This will be check for:
/// - The user's username length. It must be between 3 and 32 characters.
/// - The user's username spaces. It can't contain spaces.
/// - The user's username characters. It must start with a letter and only contain English letters, numbers, and underscores.
pub fn validate_user_signin(usersignin: &str) -> ApiResult<()> {
    if usersignin.chars().count() > 32
        || usersignin.chars().count() < 3
        || !usersignin.starts_with(|c: char| c.is_ascii_alphabetic())
        || !usersignin
            .chars()
            .all(|c: char| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(ApiError::InvalidUsername);
    }
    Ok(())
}

/// Validates a user registration.
///
/// This will validate:
/// - The user's first name.
/// - The user's last name.
/// - The user's username.
/// - The user's password.
#[must_use = "This function returns a `ApiResult<()>` instead of panicking"]
pub fn validate_user_registration(user: &NewUserSchema) -> ApiResult<()> {
    validate_user_name(&user.first_name)?;
    user.last_name
        .as_deref()
        .map(validate_user_name)
        .transpose()?;
    validate_password(&user.password)?;
    validate_user_signin(&user.username)?;
    Ok(())
}

/// Checks if the given captcha token and answer are valid.
/// If the captcha token is valid, it will be marked as used.
#[must_use = "This function will not panic, but it will return an error if the captcha token is invalid"]
pub async fn check_captcha_answer(
    conn: &sea_orm::DatabaseConnection,
    captcha_token: Uuid,
    user_answer: &str,
) -> ApiResult<()> {
    log::info!("Checking captcha answer");

    let captcha = CaptchaEntity::find()
        .filter(CaptchaColumn::Uuid.eq(captcha_token))
        .one(conn)
        .await?
        .ok_or(ApiError::InvalidCaptchaToken)?;
    if chrono::Utc::now().naive_utc() > captcha.expired_at {
        Err(ApiError::InvalidCaptchaToken)
    } else if !captcha.answer.eq_ignore_ascii_case(user_answer) {
        Err(ApiError::InvalidCaptchaAnswer)
    } else {
        CaptchaEntity::delete(captcha.into_active_model())
            .exec(conn)
            .await?;
        Ok(())
    }
}
