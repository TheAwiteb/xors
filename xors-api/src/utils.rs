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

use std::cmp::Ordering;

use base64::Engine;
use image::GenericImageView;
use passwords::{analyzer, scorer};
use uuid::Uuid;

use crate::{
    api::xo::PlayerData,
    errors::{ApiError, ApiResult},
    schemas::*,
};

/// Validates a user name. not the user signin, but the first name and last name.
#[must_use = "This function returns a `ApiResult<()>` instead of panicking"]
pub fn validate_user_name<const IS_FIRST_NAME: bool>(name: &str) -> ApiResult<()> {
    if name.chars().count() > 32
        || name.chars().count() == 0
        || name.chars().any(|c: char| c.is_whitespace())
    {
        if IS_FIRST_NAME {
            return Err(ApiError::InvalidFirstName);
        } else {
            return Err(ApiError::InvalidLastName);
        }
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
    validate_user_name::<true>(&user.first_name)?;
    user.last_name
        .as_deref()
        .map(validate_user_name::<false>)
        .transpose()?;
    validate_password(&user.password)?;
    validate_user_signin(&user.username)?;
    Ok(())
}

/// Returns the game over data for the given game
pub(crate) fn game_over_data(
    game_uuid: Uuid,
    rounds_result: &RoundsResult,
    player: &PlayerData,
    versus_player: &PlayerData,
) -> GameOverData {
    match rounds_result
        .wins(&player.symbol)
        .cmp(&rounds_result.wins(&versus_player.symbol))
    {
        Ordering::Greater => {
            GameOverData::new(game_uuid, Some(*player.uuid), GameOverReason::PlayerWon)
        }
        Ordering::Less => {
            GameOverData::new(game_uuid, Some(*player.uuid), GameOverReason::PlayerWon)
        }
        Ordering::Equal => GameOverData::new(game_uuid, None, GameOverReason::Draw),
    }
}

/// Check if the player played a valid move. Returns true if the move is valid, false otherwise.
///
/// Note: This function will send an error message to the player if the move is invalid.
pub(crate) fn check_move_validity(board: &Board, player: &PlayerData, place: u8) -> bool {
    if board.turn() != player.symbol {
        log::error!("Player {} is playing while it's not his turn", player.uuid);

        let _ = player.tx.send(Ok(
            XoServerEventData::Error(ErrorData::NotYourTurn).to_message()
        ));
    } else if place > 8 || !board.is_empty_cell(place) {
        log::error!(
            "Player {} is playing in an non empty cell or invalid place",
            player.uuid
        );

        let _ = player.tx.send(Ok(
            XoServerEventData::Error(ErrorData::InvalidPlace).to_message()
        ));
    } else if board.is_end() {
        log::error!("Player {} is playing while the round is over", player.uuid);

        let _ = player.tx.send(Ok(
            XoServerEventData::Error(ErrorData::NotYourTurn).to_message()
        ));
    } else {
        return true;
    }
    false
}

/// Handle the captcha state and return an error if the captcha state is invalid. Otherwise, return Ok.
pub(crate) fn handle_captcha_state(captcha_state: &salvo_captcha::CaptchaState) -> ApiResult<()> {
    use salvo_captcha::CaptchaState::*;

    let err = match captcha_state {
        TokenNotFound => ApiError::UnProvidedCaptchaToken,
        AnswerNotFound => ApiError::UnProvidedCaptchaAnswer,
        WrongToken => ApiError::InvalidCaptchaToken,
        WrongAnswer => ApiError::InvalidCaptchaAnswer,
        StorageError => ApiError::InternalServer,
        _ => return Ok(()),
    };

    Err(err)
}

/// Validates a user profile image.
///
/// Checks
/// - The image's size. It must be less than 1MB.
/// - The image's format. only png format is allowed.
/// - The image's dimensions. It must be 128x128 pixels.
pub(crate) fn validate_user_profile_image(profile_image: Option<&String>) -> ApiResult<()> {
    log::info!("Validating user profile image");

    if let Some(profile_image) = profile_image {
        let image_bytes = crate::BASE_64_ENGINE
            .decode(profile_image)
            .map_err(|_| ApiError::InvalidProfileImage("Invalid base64 string".to_owned()))?;
        if image_bytes.len() >= 1_000_000 {
            // 1MB
            return Err(ApiError::InvalidProfileImage(
                "The image's size must be less than 1MB".to_owned(),
            ));
        }
        if image::guess_format(&image_bytes).is_ok_and(|f| f != image::ImageFormat::Png) {
            return Err(ApiError::InvalidProfileImage(
                "The image's format must be png".to_owned(),
            ));
        }

        // Safe to unwrap because we already checked the image's format and size.
        let image = image::load_from_memory(&image_bytes).unwrap();
        let dimensions = image.dimensions();
        if dimensions.0 != 128 || dimensions.1 != 128 {
            return Err(ApiError::InvalidProfileImage(
                "The image's dimensions must be 128x128 pixels".to_owned(),
            ));
        }
        log::info!("User profile image is valid");
    }
    Ok(())
}

/// Returns image disk path by file name
pub(crate) fn get_image_disk_path(file_name: &str) -> String {
    // For security reasons, will check the file name to prevent directory traversal attacks.
    let file_name = file_name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_' || *c == '-' || *c == '/')
        .collect::<String>();
    format!("./profile_images/{file_name}")
}
