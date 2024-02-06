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

use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
    sync::Arc,
};

use chrono::Duration;
use entity::prelude::*;
use futures_util::{FutureExt, StreamExt};
use once_cell::sync::Lazy;
use pgp::{Deserializable, Message as PGPMessage, SignedPublicKey};
use rand::prelude::SliceRandom;
use salvo::{prelude::*, websocket::Message};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

use crate::{db_utils, errors::ApiResult, schemas::*, utils};

use super::exts::*;

/// The XO playe type
pub type Player = (
    Arc<Uuid>,
    Arc<mpsc::UnboundedSender<Result<Message, salvo::Error>>>,
);

/// The XO games, for each game there is the game uuid and tow channels for the players.
///
/// Note: The first player is the X player and the second player is the O player.
pub type Games = HashMap<Uuid, (Player, Player)>;

static ONLINE_GAMES: Lazy<RwLock<Games>> = Lazy::new(RwLock::default);
static SEARCH_FOR_GAME: Lazy<RwLock<VecDeque<Player>>> = Lazy::new(|| RwLock::new(VecDeque::new()));

/// XO winning combinations.
pub const WINNING_COMBINATIONS: &[(usize, usize, usize)] = &[
    (0, 1, 2), // Rows
    (3, 4, 5),
    (6, 7, 8),
    (0, 3, 6), // Columns
    (1, 4, 7),
    (2, 5, 8),
    (0, 4, 8), // Diagonals
    (2, 4, 6),
];

/// Player data
#[derive(derive_new::new)]
pub(crate) struct PlayerData {
    pub uuid: Arc<Uuid>,
    pub tx: Arc<mpsc::UnboundedSender<Result<Message, salvo::Error>>>,
    pub symbol: XoSymbol,
}

#[handler]
pub async fn user_connected(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> ApiResult<()> {
    let conn = depot
        .obtain::<Arc<sea_orm::DatabaseConnection>>()
        .unwrap()
        .clone();
    let user_uuid = Arc::new(depot.user(&conn).await?.uuid);
    let max_online_games = depot.get::<Arc<usize>>("max_online_games").unwrap().clone();
    let move_period = depot.get::<Arc<i64>>("move_period").unwrap().clone();

    WebSocketUpgrade::new()
        .upgrade(req, res, |ws| async move {
            log::debug!("User connected to the XO websocket: {user_uuid}");

            let (user_ws_tx, mut user_ws_rx) = ws.split();

            let (tx, rx) = mpsc::unbounded_channel();
            let rx = UnboundedReceiverStream::new(rx);
            let tx = Arc::new(tx);
            let fut = rx.forward(user_ws_tx).map(|result| {
                if let Err(e) = result {
                    log::error!("websocket send error: {e}");
                }
            });
            tokio::task::spawn(fut);
            let fut = async move {
                while let Some(result) = user_ws_rx.next().await {
                    let msg = match result {
                        Ok(msg) => msg,
                        Err(e) => {
                            log::error!("websocket error: {e}");
                            break;
                        }
                    };
                    // Handle message
                    if msg.is_text() {
                        log::debug!("Received a text message: {msg:?}");
                        if let Ok(event) = serde_json::from_str::<XoClientEvent>(
                            msg.to_str().expect("The message is text"),
                        ) {
                            log::debug!("Received a valid XO event: {event:?}");
                            if let Err(err) = handle_event(
                                event,
                                &conn,
                                max_online_games.as_ref(),
                                *move_period,
                                tx.clone(),
                                user_uuid.clone(),
                            )
                            .await
                            {
                                tx.send_server_event(XoServerEventData::Error(ErrorData::Other(
                                    err.to_string(),
                                )));
                            }
                        } else {
                            tx.send_server_event(XoServerEventData::Error(ErrorData::UnknownEvent));
                        }
                    } else {
                        tx.send_server_event(XoServerEventData::Error(ErrorData::InvalidBody));
                    }
                }

                player_disconnected(&conn, (user_uuid.clone(), tx.clone()))
                    .await
                    .ok();
            };
            tokio::task::spawn(fut);
        })
        .await?;
    Ok(())
}

/// Handle the XO client event.
async fn handle_event(
    event: XoClientEvent,
    conn: &Arc<sea_orm::DatabaseConnection>,
    max_online_games: &usize,
    move_period: i64,
    tx: Arc<mpsc::UnboundedSender<Result<Message, salvo::Error>>>,
    user: Arc<Uuid>,
) -> ApiResult<()> {
    match (event.event, event.data) {
        (XoClientEventKind::Search, None) => {
            search_for_game(conn, max_online_games, move_period, (user, tx)).await?
        }
        (XoClientEventKind::Play, Some(XoClientEventsData::Play { place })) => {
            play(conn, (user, tx), place, move_period).await?
        }
        (XoClientEventKind::Wellcome, Some(XoClientEventsData::Wellcome { public_key })) => {
            wellcome(conn, (user, tx), public_key).await?
        }
        (
            XoClientEventKind::Chat,
            Some(XoClientEventsData::Chat {
                encrypted_message,
                signature,
            }),
        ) => chat(conn, (user, tx), encrypted_message, signature).await?,
        _ => {
            tx.send_server_event(XoServerEventData::Error(ErrorData::InvalidBody));
        }
    }

    Ok(())
}

/// Search for a game.
async fn search_for_game(
    conn: &sea_orm::DatabaseConnection,
    max_online_games: &usize,
    move_period: i64,
    player: Player,
) -> ApiResult<()> {
    log::info!("Player {} is searching for a game", player.0);

    if ONLINE_GAMES.is_user_in_game(&player.0).await {
        log::error!("Player {} is already in a game", player.0);
        player
            .1
            .send_server_event(XoServerEventData::Error(ErrorData::AlreadyInGame));
    } else if &ONLINE_GAMES.online_games_count().await >= max_online_games {
        log::error!(
            "Player {} can't join the game because the max online games is reached",
            player.0
        );
        player
            .1
            .send_server_event(XoServerEventData::Error(ErrorData::MaxGamesReached));
    } else if SEARCH_FOR_GAME.is_user_in_search(player.0.as_ref()).await {
        log::error!("Player {} is already in the search queue", player.0);
        player
            .1
            .send_server_event(XoServerEventData::Error(ErrorData::AlreadyInSearch));
    } else if SEARCH_FOR_GAME.search_users_count().await > 0 {
        let Some(other_player) = SEARCH_FOR_GAME.pop_front().await else {
            unreachable!("The search queue is not empty")
        };
        let player = PlayerData::new(player.0, player.1, XoSymbol::O);
        let other_player = PlayerData::new(other_player.0, other_player.1, XoSymbol::X);

        log::info!(
            "Player {} found a game with player {}",
            player.uuid,
            other_player.uuid
        );

        let game =
            db_utils::create_game(conn, *other_player.uuid, *player.uuid, move_period).await?;

        ONLINE_GAMES
            .add_game(
                *game.uuid.as_ref(),
                (other_player.uuid.clone(), other_player.tx.clone()),
                (player.uuid.clone(), player.tx.clone()),
            )
            .await;

        ONLINE_GAMES
            .broadcast_messages(
                *game.uuid.as_ref(),
                &[
                    XoServerEventData::GameFound {
                        x_player: *other_player.uuid,
                        o_player: *player.uuid,
                    },
                    XoServerEventData::RoundStart { round: 1 },
                ],
            )
            .await;

        other_player
            .tx
            .send_server_event(XoServerEventData::YourTurn {
                auto_play_after: game.auto_play_after.as_ref().unwrap().timestamp(),
            });
    } else {
        log::info!("Player {} is added to the search queue", player.0);
        SEARCH_FOR_GAME.add_user(player).await;
    }

    Ok(())
}

/// Play a move.
async fn play(
    conn: &sea_orm::DatabaseConnection,
    player: Player,
    place: u8,
    move_period: i64,
) -> ApiResult<()> {
    if let Some((game_uuid, versus_player)) = ONLINE_GAMES.get_user_game(&player.0).await {
        log::info!("Player {} is playing in place {}", player.0, place);

        let mut game = db_utils::get_game::<false>(conn, &game_uuid).await?;
        let mut board = Board::from_str(&game.board).expect("The board is valid");
        let (player_symbol, versus_symbol) = if game.x_player == *player.0 {
            (XoSymbol::X, XoSymbol::O)
        } else {
            (XoSymbol::O, XoSymbol::X)
        };

        let player = PlayerData::new(player.0, player.1, player_symbol);
        let versus_player = PlayerData::new(versus_player.0, versus_player.1, versus_symbol);

        if utils::check_move_validity(&board, &player, place) {
            versus_player
                .tx
                .send_server_event(XoServerEventData::Play(PlayData::new(place, *player.uuid)));
            game.auto_play_after =
                Some((chrono::Utc::now() + Duration::seconds(move_period)).naive_utc());
            board.set_cell(place, player.symbol);
            let mut rounds_result =
                RoundsResult::from_str(&game.rounds_result).expect("The rounds result is valid");

            if board.is_win(&player.symbol) {
                log::info!("Player {} won the round {}", player.uuid, game.round);
                rounds_result.add_win(&player.symbol);
            } else if board.is_draw() {
                log::info!("The round {} is a draw", game.round);
                rounds_result.draws += 1;
            }

            // Check if the game is over.
            // Game is over when the board is end and the round is 3 or the round is 2 and one of the players won 2 rounds.
            if board.is_end()
                && (game.round == 3
                    || (game.round == 2
                        && (rounds_result.x_player == 2 || rounds_result.o_player == 2)))
            {
                rounds_result.add_board(board.clone());
                let game_over_data =
                    utils::game_over_data(game.uuid, &rounds_result, &player, &versus_player);

                ONLINE_GAMES
                    .broadcast_message(
                        game.uuid,
                        XoServerEventData::GameOver(game_over_data.clone()),
                    )
                    .await;
                ONLINE_GAMES
                    .remove_game(
                        conn,
                        &game.uuid,
                        game_over_data.winner,
                        &game_over_data.reason,
                    )
                    .await?;
            } else if board.is_end() {
                // ^^ Check if the round is over
                rounds_result.add_board(board.clone());
                ONLINE_GAMES
                    .broadcast_messages(
                        game.uuid,
                        &[
                            XoServerEventData::RoundEnd(RoundData::new(
                                game.round,
                                if board.is_win(&player.symbol) {
                                    Some(*player.uuid)
                                } else {
                                    None
                                },
                            )),
                            XoServerEventData::RoundStart {
                                round: game.round + 1,
                            },
                        ],
                    )
                    .await;

                board = Board::default();
                game.round += 1;
                let data = XoServerEventData::YourTurn {
                    auto_play_after: game.auto_play_after.unwrap().timestamp(),
                };
                if player.symbol == XoSymbol::X {
                    player.tx.send_server_event(data);
                } else {
                    versus_player.tx.send_server_event(data);
                }
            } else {
                versus_player
                    .tx
                    .send_server_event(XoServerEventData::YourTurn {
                        auto_play_after: game.auto_play_after.unwrap().timestamp(),
                    });
            }

            let mut game = game.into_active_model();
            game.board = Set(board.to_string());
            game.rounds_result = Set(rounds_result.to_string());
            game.round = Set(game.round.unwrap());
            game.auto_play_after = Set(game.auto_play_after.unwrap());
            game.save(conn).await?;
        }
    } else {
        player
            .1
            .send_server_event(XoServerEventData::Error(ErrorData::NotInGame));
    }
    Ok(())
}

/// Wellcome event handler.
async fn wellcome(
    conn: &sea_orm::DatabaseConnection,
    player: Player,
    public_key: String,
) -> ApiResult<()> {
    log::info!("Player {} sent a wellcome event", player.0);

    if let Some((game_uuid, versus_player)) = ONLINE_GAMES.get_user_game(&player.0).await {
        log::info!("Player {} is in a game", player.0);

        if SignedPublicKey::from_string(&public_key).is_err() {
            log::error!("Player {} sent an invalid public key", player.0);
            player
                .1
                .send_server_event(XoServerEventData::Error(ErrorData::InvalidPublicKey));
            return Ok(());
        }

        let game = db_utils::get_game::<false>(conn, &game_uuid).await?;
        let (player_symbol, versus_symbol) = if game.x_player == *player.0 {
            (XoSymbol::X, XoSymbol::O)
        } else {
            (XoSymbol::O, XoSymbol::X)
        };
        let player = PlayerData::new(player.0, player.1, player_symbol);
        let versus_player = PlayerData::new(versus_player.0, versus_player.1, versus_symbol);

        if (player.uuid.as_ref() == &game.x_player && game.x_start_chat)
            || (player.uuid.as_ref() == &game.o_player && game.o_start_chat)
        {
            player
                .tx
                .send_server_event(XoServerEventData::Error(ErrorData::AlreadyWellcomed));
            return Ok(());
        }

        let mut game = game.into_active_model();
        if game.x_player.as_ref() == player.uuid.as_ref() {
            game.x_start_chat = Set(true);
        } else {
            game.o_start_chat = Set(true);
        }
        game.save(conn).await?;

        versus_player
            .tx
            .send_server_event(XoServerEventData::Wellcome { public_key });
    } else {
        player
            .1
            .send_server_event(XoServerEventData::Error(ErrorData::NotInGame));
    }
    Ok(())
}

/// Chat event handler.
async fn chat(
    conn: &sea_orm::DatabaseConnection,
    player: Player,
    encrypted_message: String,
    signature: String,
) -> ApiResult<()> {
    log::info!("Player {} sent a chat event", player.0);

    if let Some((game_uuid, versus_player)) = ONLINE_GAMES.get_user_game(&player.0).await {
        log::info!("Player {} is in a game", player.0);
        if PGPMessage::from_string(&encrypted_message).is_err() {
            log::error!("Player {} sent an invalid encrypted message", player.0);
            player
                .1
                .send_server_event(XoServerEventData::Error(ErrorData::InvalidChatMessage));
            return Ok(());
        }

        if !signature.starts_with("-----BEGIN PGP SIGNED MESSAGE-----")
            && !signature.ends_with("-----END PGP SIGNATURE-----")
        {
            log::error!("Player {} sent an invalid signature", player.0);
            player
                .1
                .send_server_event(XoServerEventData::Error(ErrorData::InvalidChatSignature));
            return Ok(());
        }

        let game = db_utils::get_game::<false>(conn, &game_uuid).await?;
        let (player_symbol, versus_symbol) = if game.x_player == *player.0 {
            (XoSymbol::X, XoSymbol::O)
        } else {
            (XoSymbol::O, XoSymbol::X)
        };
        let player = PlayerData::new(player.0, player.1, player_symbol);
        let versus_player = PlayerData::new(versus_player.0, versus_player.1, versus_symbol);

        if (player.uuid.as_ref() == &game.x_player && !game.x_start_chat)
            || (player.uuid.as_ref() == &game.o_player && !game.o_start_chat)
        {
            player
                .tx
                .send_server_event(XoServerEventData::Error(ErrorData::ChatNotAllowed));
            return Ok(());
        }

        if !game.x_start_chat || !game.o_start_chat {
            player
                .tx
                .send_server_event(XoServerEventData::Error(ErrorData::ChatNotStarted));
            return Ok(());
        }

        versus_player.tx.send_server_event(XoServerEventData::Chat {
            encrypted_message,
            signature,
        });
    } else {
        player
            .1
            .send_server_event(XoServerEventData::Error(ErrorData::NotInGame));
    }
    Ok(())
}

/// The user disconnected handler.
async fn player_disconnected(conn: &sea_orm::DatabaseConnection, player: Player) -> ApiResult<()> {
    log::info!("Player {} disconnected", player.0);

    // If the player disconnected while in a game, then the other player will win.
    if let Some((game_uuid, versus_player)) = ONLINE_GAMES.get_user_game(&player.0).await {
        log::info!("Player {} disconnected while in a game", player.0);

        versus_player
            .1
            .send_server_event(XoServerEventData::GameOver(GameOverData::new(
                game_uuid,
                Some(*versus_player.0),
                GameOverReason::PlayerDisconnected,
            )));
        ONLINE_GAMES
            .remove_game(
                conn,
                &game_uuid,
                Some(*versus_player.0),
                &GameOverReason::PlayerDisconnected,
            )
            .await?;
    } else if SEARCH_FOR_GAME.is_user_in_search(player.0.as_ref()).await {
        // ^ If the player disconnected while searching for a game, then remove him from the search queue.
        log::info!(
            "Player {} disconnected while searching for a game",
            player.0
        );

        SEARCH_FOR_GAME.remove_user(&player.0).await;
    }
    Ok(())
}

/// Auto play handler.
/// ### Note
/// This function will run while there is at least one online game, if not then it will wait 5 seconds and check again.
pub async fn auto_play_handler(conn: sea_orm::DatabaseConnection, move_period: i64) {
    async fn inner(conn: &sea_orm::DatabaseConnection, move_period: i64) -> ApiResult<()> {
        log::info!("Starting auto play handler");

        loop {
            let games = db_utils::get_online_games(conn).await?;
            if !games.is_empty() {
                for game in games {
                    if let Some(auto_play_after) = game.auto_play_after {
                        let board = Board::from_str(&game.board).expect("The board is valid");
                        if chrono::Utc::now().naive_utc() >= auto_play_after {
                            let players = ONLINE_GAMES
                                .get_game_players(&game.uuid)
                                .await
                                .expect("The game should be in the online games");
                            let player = if board.turn() == XoSymbol::X {
                                players.0.clone()
                            } else {
                                players.1.clone()
                            };

                            let place = board
                            .empty_cells()
                            .choose(&mut rand::thread_rng())
                            .expect("There is at least one empty cell, if the board is full then the game should be ended")
                            .to_owned();

                            player
                                .1
                                .send_server_event(XoServerEventData::AutoPlay { place });

                            log::info!(
                                "Playing for player {} in game {} in place {place}",
                                player.0,
                                game.uuid
                            );
                            play(conn, player.clone(), place, move_period).await?;
                        }
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            } else {
                log::info!("There is no online games, waiting 5 seconds");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }

    loop {
        if let Err(err) = inner(&conn, move_period).await {
            log::error!("Auto play handler error: {err}");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}
