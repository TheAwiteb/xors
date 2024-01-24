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

use entity::prelude::*;
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db_utils,
    errors::{ApiError, ApiResult},
};

use super::*;

/// The game's schema. It's used to return the game's data.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, derive_new::new)]
#[salvo(schema(symbol = "GameSchema", example = json!(GameSchema::default())))]
pub struct GameSchema {
    /// The game's uuid. It's unique.
    pub uuid: Uuid,
    /// The X Player
    pub x_player: UserSchema,
    /// The O Player
    pub o_player: UserSchema,
    /// The game's rounds results. Wins, loses, draws, also the board for each round.
    pub rounds_results: RoundsResult,
    /// The game's winner. will be null if the game ended with a draw.
    pub winner: Option<Uuid>,
    /// The won reason. will be null if the game ended with a draw.
    pub won_reason: Option<GameOverReason>,
    /// The game's creation date.
    pub created_at: chrono::NaiveDateTime,
}

impl Default for GameSchema {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            x_player: UserSchema::default(),
            o_player: UserSchema::default(),
            rounds_results: RoundsResult::default(),
            winner: None,
            won_reason: None,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

impl GameSchema {
    pub(crate) async fn from_game(
        conn: &sea_orm::DatabaseConnection,
        game: GameModel,
    ) -> ApiResult<Self> {
        let get_player = |player_uuid| async move {
            db_utils::get_user(conn, player_uuid)
                .await
                .map(UserSchema::from)
                .or_else(|err| {
                    if let ApiError::UserNotFound = err {
                        Ok(UserSchema::deleted_user())
                    } else {
                        Err(err)
                    }
                })
        };

        Ok(Self::new(
            game.uuid,
            get_player(game.x_player).await?,
            get_player(game.o_player).await?,
            game.rounds_result.parse().expect("Is valid rounds result"),
            game.winner,
            game.reason
                .map(|reason| reason.parse().expect("Is valid game over reason")),
            game.created_at,
        ))
    }
}
