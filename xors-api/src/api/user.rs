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

use crate::api::exts::*;
use crate::utils;
use crate::{
    errors::{ApiError, ApiResult},
    schemas::*,
};

use entity::prelude::*;
use salvo::oapi::extract::{JsonBody, QueryParam};
use salvo::prelude::*;
use salvo::{oapi::endpoint, writing::Json};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

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
///
/// Will returns error if there is no changes.
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

    if updated_user
        .first_name
        .as_ref()
        .is_some_and(|s| s == &user.first_name)
        && updated_user.last_name == user.last_name
    {
        Err(ApiError::NoChanges)
    } else {
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
        let user = user.save(conn.as_ref()).await?;
        Ok(Json(user.into()))
    }
}
