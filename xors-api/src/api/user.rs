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

use crate::api::exts::*;
use crate::{db_utils, utils};
use crate::{
    errors::{ApiError, ApiResult},
    schemas::*,
};

use base64::Engine;
use entity::prelude::*;
use salvo::oapi::extract::{JsonBody, PathParam, QueryParam};
use salvo::prelude::*;
use salvo::{oapi::endpoint, writing::Json};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use std::path::Path;
use std::sync::Arc;

/// Get me, the user that make the request.
///
/// This endpoint will return the user that make the request.
#[endpoint(
    operation_id = "get_me",
    tags("User"),
    responses(
        (status_code = 200, description = "The user's info", content_type = "application/json", body = UserSchema),
        (status_code = 400, description = "The token is not a user token", content_type = "application/json", body = MessageSchema),
        (status_code = 401, description = "The token is expired", content_type = "application/json", body = MessageSchema),
        (status_code = 401, description = "Unauthorized, missing JWT", content_type = "application/json", body = MessageSchema),
        (status_code = 404, description = "User not found", content_type = "application/json", body = MessageSchema),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn get_me(depot: &mut Depot) -> ApiResult<Json<UserSchema>> {
    let conn = depot.obtain::<Arc<DatabaseConnection>>().unwrap();
    let user = depot.user(conn.as_ref()).await?;
    Ok(Json(user.into_active_model().into()))
}

/// Get the user's info.
#[endpoint(
    operation_id = "get_user_info",
    tags("User"),
    parameters(
        ("uuid" = Uuid, Query, description = "The requested user's uuid"),
    ),
    responses(
        (status_code = 200, description = "The user's info", content_type = "application/json", body = UserSchema),
        (status_code = 404, description = "User not found", content_type = "application/json", body = MessageSchema),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    ),
)]
pub async fn get_user_info(
    depot: &mut Depot,
    uuid: QueryParam<Uuid, true>,
) -> ApiResult<Json<UserSchema>> {
    let conn = depot.obtain::<Arc<DatabaseConnection>>().unwrap();
    let requested_user_uuid = uuid.into_inner();

    UserEntity::find()
        .filter(UserColumn::Uuid.eq(requested_user_uuid))
        .one(conn.as_ref())
        .await?
        .map(|u| Json(u.into_active_model().into()))
        .ok_or_else(|| ApiError::UserNotFound)
}

/// Delete the user's account.
///
/// This endpoint will delete the user's account and all the user's data. Forever.
#[endpoint(
    operation_id = "delete_user_info",
    tags("User"),
    responses(
        (status_code = 200, description = "The user's account has been deleted", content_type = "application/json", body = MessageSchema),
        (status_code = 400, description = "The token is not a user token", content_type = "application/json", body = MessageSchema),
        (status_code = 400, description = "Invalid password: The password is incorrect", content_type = "application/json", body = MessageSchema),
        (status_code = 401, description = "The token is expired", content_type = "application/json", body = MessageSchema),
        (status_code = 401, description = "Unauthorized, missing JWT", content_type = "application/json", body = MessageSchema),
        (status_code = 404, description = "User not found", content_type = "application/json", body = MessageSchema),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn delete_user(
    depot: &mut Depot,
    delete_user_schema: JsonBody<DeleteUserSchema>,
) -> ApiResult<Json<MessageSchema>> {
    let conn = depot.obtain::<Arc<DatabaseConnection>>().unwrap();
    let user = depot.user(conn.as_ref()).await?;

    if bcrypt::verify(
        delete_user_schema.into_inner().password,
        user.password_hash.as_ref(),
    )? {
        UserEntity::delete(user.into_active_model())
            .exec(conn.as_ref())
            .await?;

        Ok(Json(MessageSchema::new(
            "The user's account has been deleted".to_owned(),
        )))
    } else {
        Err(ApiError::InvalidPassword(
            "The password is incorrect".to_owned(),
        ))
    }
}

/// Update the user's info.
#[endpoint(
    operation_id = "update_user_info",
    tags("User"),
    responses(
        (status_code = 200, description = "The user's info has been updated", content_type = "application/json", body = UserSchema),
        (status_code = 400, description = "There is no changes", content_type = "application/json", body = MessageSchema),
        (status_code = 400, description = "The token is not a user token", content_type = "application/json", body = MessageSchema),
        (status_code = 401, description = "The token is expired", content_type = "application/json", body = MessageSchema),
        (status_code = 401, description = "Unauthorized, missing JWT", content_type = "application/json", body = MessageSchema),
        (status_code = 404, description = "User not found", content_type = "application/json", body = MessageSchema),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn update_user(
    depot: &mut Depot,
    updated_user: JsonBody<UpdateUserSchema>,
) -> ApiResult<Json<UserSchema>> {
    let conn = depot.obtain::<Arc<DatabaseConnection>>().unwrap();
    let user = depot.user(conn.as_ref()).await?;
    let updated_user = updated_user.into_inner();

    let mut user = user.into_active_model();

    if let Some(first_name) = updated_user.first_name {
        if &first_name != user.first_name.as_ref() {
            utils::validate_user_name::<true>(&first_name)?;
            user.first_name = Set(first_name);
        }
    } else {
        return Err(ApiError::InvalidFirstName);
    }

    if &updated_user.last_name != user.last_name.as_ref() {
        updated_user
            .last_name
            .as_deref()
            .map(utils::validate_user_name::<false>)
            .transpose()?;
        user.last_name = Set(updated_user.last_name);
    }

    let profile_image_path =
        db_utils::update_profile_image_path(*user.uuid.as_ref(), updated_user.profile_image)?;
    user.profile_image_path = Set(profile_image_path);
    let user = user.save(conn.as_ref()).await?;
    Ok(Json(user.into()))
}

/// Reset the user's password.
#[endpoint(
    operation_id = "reset_user_password",
    tags("User"),
    responses(
        (status_code = 200, description = "The user's password has been reset", content_type = "application/json", body = MessageSchema),
        (status_code = 400, description = "Incorrect old password", content_type = "application/json", body = MessageSchema),
        (status_code = 400, description = "The password does not change", content_type = "application/json", body = MessageSchema),
        (status_code = 400, description = "The password dose not meet the requirements", content_type = "application/json", body = MessageSchema),
        (status_code = 400, description = "The token is not a user token", content_type = "application/json", body = MessageSchema),
        (status_code = 401, description = "The token is expired", content_type = "application/json", body = MessageSchema),
        (status_code = 401, description = "Unauthorized, missing JWT", content_type = "application/json", body = MessageSchema),
        (status_code = 404, description = "User not found", content_type = "application/json", body = MessageSchema),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn reset_user_password(
    depot: &mut Depot,
    reset_password: JsonBody<ResetPasswordSchema>,
) -> ApiResult<Json<MessageSchema>> {
    let conn = depot.obtain::<Arc<DatabaseConnection>>().unwrap();
    let user = depot.user(conn.as_ref()).await?;
    let reset_password = reset_password.into_inner();

    utils::validate_password(&reset_password.new_password)?;
    if bcrypt::verify(&reset_password.new_password, user.password_hash.as_ref())? {
        return Err(ApiError::PasswordNotChanged);
    }

    if bcrypt::verify(&reset_password.old_password, user.password_hash.as_ref())? {
        db_utils::reset_password(conn.as_ref(), user, &reset_password.new_password).await?;
        Ok(Json(MessageSchema::new(
            "The user's password has been reset".to_owned(),
        )))
    } else {
        Err(ApiError::InvalidPassword(
            "The old password is incorrect".to_owned(),
        ))
    }
}

/// Returns the user's profile image.
#[endpoint(
    operation_id = "get_user_profile_image",
    tags("User"),
    parameters(
        ("uuid" = Uuid, Path, description = "The requested user's uuid"),
    ),
    responses(
        (status_code = 200, description = "The user's profile image", content_type = "application/json", body = ImageSchema),
        (status_code = 404, description = "User not found", content_type = "application/json", body = MessageSchema),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    ),
)]
pub async fn get_user_profile_image(uuid: PathParam<Uuid>) -> ApiResult<Json<ImageSchema>> {
    let requested_user_uuid = uuid.into_inner();

    let image_path = if Path::new(&utils::get_image_disk_path(
        &requested_user_uuid.to_string(),
    ))
    .exists()
    {
        utils::get_image_disk_path(&requested_user_uuid.to_string())
    } else {
        utils::get_image_disk_path("default")
    };

    let image_base64 = crate::BASE_64_ENGINE
        .encode(std::fs::read(image_path).map_err(|_| ApiError::InternalServer)?);
    Ok(Json(ImageSchema::new(image_base64)))
}
