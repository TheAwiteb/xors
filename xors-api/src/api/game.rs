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

use crate::{db_utils, errors::ApiResult, schemas::*};

use futures_util::StreamExt;
use salvo::oapi::extract::PathParam;
use salvo::prelude::*;
use salvo::{oapi::endpoint, writing::Json};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use std::sync::Arc;

/// Get the game by uuid.
///
/// **Note**: This will return the game only if it's ended.
#[endpoint(
    operation_id = "get_game_by_uuid",
    tags("Game"),
    parameters(
        ("uuid" = Uuid, Path, description = "The requested game's uuid"),
    ),
    responses(
        (status_code = 200, description = "The game's info", content_type = "application/json", body = GameSchema),
        (status_code = 400, description = "The uuid is invalid", content_type = "application/json", body = MessageSchema),
        (status_code = 404, description = "Game not found", content_type = "application/json", body = MessageSchema),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    ),
)]
pub async fn get_game_by_uuid(
    depot: &mut Depot,
    uuid: PathParam<Uuid>,
) -> ApiResult<Json<GameSchema>> {
    let conn = depot.obtain::<Arc<DatabaseConnection>>().unwrap().as_ref();

    GameSchema::from_game(
        conn,
        db_utils::get_game::<true>(conn, &uuid.into_inner()).await?,
    )
    .await
    .map(Json)
}

/// Get the lastest 10 games.
///
/// This endpoint will return the lastest 10 games, sorted by the creation date (newest first)
///
/// **Note**: If the O player or the X player is deleted, the game will return it as deleted user, which it's uuid is `00000000-0000-0000-0000-000000000000` and username is `Deleted`.
#[endpoint(
    operation_id = "get_lastest_games",
    tags("Game"),
    responses(
        (status_code = 200, description = "The lastest 10 games", content_type = "application/json", body = Vec<GameSchema>),
        (status_code = 500, description = "Internal server error", content_type = "application/json", body = MessageSchema),
        (status_code = 429, description = "Too many requests", content_type = "application/json", body = MessageSchema),
    ),
)]
pub async fn get_lastest_games(depot: &mut Depot) -> ApiResult<Json<Vec<GameSchema>>> {
    let conn = depot.obtain::<Arc<DatabaseConnection>>().unwrap().as_ref();

    Ok(Json(
        futures_util::stream::iter(db_utils::get_lastest_games(conn).await?)
            .then(|game| GameSchema::from_game(conn, game))
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<ApiResult<_>>()?,
    ))
}
