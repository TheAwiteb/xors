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

use std::{collections::VecDeque, sync::Arc};

use easy_ext::ext;
use entity::prelude::*;
use salvo::{prelude::*, websocket::Message};
use sea_orm::DatabaseConnection;
use tokio::sync::{mpsc::UnboundedSender, RwLock};
use uuid::Uuid;

use crate::{
    db_utils,
    errors::{ApiError, ApiResult},
    schemas::{GameOverReason, XoServerEventData},
};

use super::{jwt::JwtClaims, xo::Player};

#[ext(UserExt)]
impl Depot {
    pub(crate) async fn user(&self, conn: &DatabaseConnection) -> ApiResult<UserModel> {
        // Note: The `Unauthorized` and `Forbidden` errors are handled by the `JwtAuth` middleware.
        let jwt = self
            .jwt_auth_data::<JwtClaims>()
            .expect("The user is authorized so it should be here");

        if jwt.claims.is_refresh_token() {
            Err(ApiError::NotUserJwt)
        } else if jwt.claims.is_expired() {
            Err(ApiError::ExpiredToken)
        } else {
            UserEntity::find()
                .filter(UserColumn::Uuid.eq(jwt.claims.uuid))
                .one(conn)
                .await?
                .ok_or_else(|| ApiError::UserNotFound)
        }
    }

    pub(crate) fn jwt_claims(&self) -> &JwtClaims {
        &self
            .jwt_auth_data::<JwtClaims>()
            .expect("The user is authorized so it should be here")
            .claims
    }
}

#[ext(WriteGamesExt)]
impl RwLock<super::xo::Games> {
    pub(crate) async fn add_game(&self, game_uuid: Uuid, player1: Player, player2: Player) {
        self.write().await.insert(game_uuid, (player1, player2));
    }

    /// This will remove the game from the database and the in-memory map.
    /// So if there is brodcast you should do it before calling this function.
    pub(crate) async fn remove_game(
        &self,
        conn: &sea_orm::DatabaseConnection,
        game_uuid: &Uuid,
        winner: Option<Uuid>,
        win_reason: &GameOverReason,
    ) -> ApiResult<()> {
        db_utils::end_game(conn, game_uuid, winner, win_reason).await?;
        self.write().await.remove(game_uuid);
        Ok(())
    }
}

#[ext(WriteSearchUsersExt)]
impl RwLock<VecDeque<Player>> {
    pub(crate) async fn add_user(&self, player: Player) {
        self.write().await.push_back(player);
    }

    pub(crate) async fn pop_front(&self) -> Option<Player> {
        self.write().await.pop_front()
    }

    pub(crate) async fn remove_user(&self, user_uuid: &Uuid) {
        self.write()
            .await
            .retain(|player| player.0.as_ref() != user_uuid);
    }
}

#[ext(ReadGamesExt)]
impl RwLock<super::xo::Games> {
    pub(crate) async fn broadcast_messages(&self, game_uuid: Uuid, messages: &[XoServerEventData]) {
        if let Some((player1, player2)) = self.get_game_players(&game_uuid).await {
            messages.iter().for_each(|event| {
                player1.1.send_server_event(event.clone());
                player2.1.send_server_event(event.clone());
            })
        }
    }

    pub(crate) async fn broadcast_message(&self, game_uuid: Uuid, message: XoServerEventData) {
        self.broadcast_messages(game_uuid, &[message]).await;
    }

    pub(crate) async fn get_user_game(&self, user_uuid: &Uuid) -> Option<(Uuid, Player)> {
        self.read()
            .await
            .iter()
            .find_map(|(game_uuid, (player1, player2))| {
                if player1.0.as_ref() == user_uuid {
                    Some((*game_uuid, player2.clone()))
                } else if player2.0.as_ref() == user_uuid {
                    Some((*game_uuid, player1.clone()))
                } else {
                    None
                }
            })
    }

    pub(crate) async fn get_game_players(&self, game_uuid: &Uuid) -> Option<(Player, Player)> {
        self.read()
            .await
            .get(game_uuid)
            .map(|(player1, player2)| (player1.clone(), player2.clone()))
    }

    pub(crate) async fn is_user_in_game(&self, user_uuid: &Uuid) -> bool {
        self.read().await.iter().any(|(_, (player1, player2))| {
            player1.0.as_ref() == user_uuid || player2.0.as_ref() == user_uuid
        })
    }

    pub(crate) async fn online_games_count(&self) -> usize {
        self.read().await.len()
    }
}

#[ext(ReadSearchUsersExt)]
impl RwLock<VecDeque<Player>> {
    pub(crate) async fn is_user_in_search(&self, user_uuid: &Uuid) -> bool {
        self.read()
            .await
            .iter()
            .any(|player| player.0.as_ref() == user_uuid)
    }

    pub(crate) async fn search_users_count(&self) -> usize {
        self.read().await.len()
    }
}

#[ext(SendServerEventExt)]
impl Arc<UnboundedSender<Result<Message, salvo::Error>>> {
    pub(crate) fn send_server_event(&self, event: XoServerEventData) {
        let _ = self.send(Ok(event.to_message()));
    }
}
