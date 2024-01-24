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

use salvo::{hyper::StatusCode, oapi::EndpointOutRegister, Scribe};

use crate::schemas::MessageSchema;

pub type ApiResult<T> = std::result::Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    SeaOrm(#[from] sea_orm::DbErr),
    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("{0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("{0}")]
    Salvo(#[from] salvo::http::StatusError),

    #[error("Username `{0}` already exists")]
    UsernameAlreadyExists(String),
    #[error("The token is not a refresh token")]
    NotRefreshToken,
    #[error("The token is not user jwt")]
    NotUserJwt,
    #[error("The refresh token is not active yet")]
    UnActiveRefreshToken,
    #[error("The token is expired")]
    ExpiredToken,
    #[error("User not found")]
    UserNotFound,
    #[error("Game not found")]
    GameNotFound,
    #[error("Invalid first name: The first name must be between 1 and 32 characters and not contain spaces")]
    InvalidFirstName,
    #[error("Invalid last name: The last name must be between 1 and 32 characters and not contain spaces")]
    InvalidLastName,
    #[error("Invalid username: The username must be between 3 and 32 characters and start with a letter and only contain English letters, numbers, and underscores")]
    InvalidUsername,
    #[error("Invalid password: {0}")]
    InvalidPassword(String),
    #[error("Invalid signin credentials: The username or password is incorrect")]
    InvalidSigninCredentials,
    #[error("Captcha token is invalid or expired")]
    InvalidCaptchaToken,
    #[error("The captcha answer is incorrect")]
    InvalidCaptchaAnswer,
    #[error("No changes were made")]
    NoChanges,

    #[error("Internal server error")]
    InternalServer,
}

impl EndpointOutRegister for ApiError {
    fn register(_: &mut salvo::oapi::Components, _: &mut salvo::oapi::Operation) {}
}

impl Scribe for ApiError {
    fn render(self, res: &mut salvo::prelude::Response) {
        log::error!("Error: {self}");

        match &self {
            ApiError::SeaOrm(_) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                crate::api::write_json_body(
                    res,
                    MessageSchema::new("Internal server error".to_owned()),
                );
            }
            ApiError::SerdeJson(err) => {
                // This is deserialization error, so it's a bad request.
                res.status_code(StatusCode::BAD_REQUEST);
                crate::api::write_json_body(
                    res,
                    MessageSchema::new(format!("Deserialization error: {err}")),
                );
            }
            ApiError::Bcrypt(err) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                crate::api::write_json_body(
                    res,
                    MessageSchema::new(format!("Internal server error: {err}")),
                );
            }
            ApiError::InternalServer => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                crate::api::write_json_body(
                    res,
                    MessageSchema::new("Internal server error".to_owned()),
                );
            }
            ApiError::UsernameAlreadyExists(_)
            | ApiError::NotRefreshToken
            | ApiError::NotUserJwt
            | ApiError::NoChanges
            | ApiError::InvalidFirstName
            | ApiError::InvalidLastName
            | ApiError::InvalidUsername
            | ApiError::InvalidPassword(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
                crate::api::write_json_body(res, MessageSchema::new(self.to_string()));
            }
            ApiError::UserNotFound | ApiError::GameNotFound => {
                res.status_code(StatusCode::NOT_FOUND);
                crate::api::write_json_body(res, MessageSchema::new(self.to_string()));
            }
            ApiError::ExpiredToken => {
                res.status_code(StatusCode::UNAUTHORIZED);
                crate::api::write_json_body(res, MessageSchema::new(self.to_string()));
            }
            ApiError::UnActiveRefreshToken
            | ApiError::InvalidSigninCredentials
            | ApiError::InvalidCaptchaAnswer
            | ApiError::InvalidCaptchaToken => {
                res.status_code(StatusCode::FORBIDDEN);
                crate::api::write_json_body(res, MessageSchema::new(self.to_string()));
            }
            ApiError::Salvo(err) => {
                res.status_code(err.code);
                crate::api::write_json_body(res, MessageSchema::new(self.to_string()));
            }
        }
    }
}
