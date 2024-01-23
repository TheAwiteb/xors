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

use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    str::FromStr,
    sync::Arc,
};

use chrono::Duration;
use entity::prelude::*;
use futures_util::{FutureExt, StreamExt};
use once_cell::sync::Lazy;
use rand::prelude::SliceRandom;
use salvo::{prelude::*, websocket::Message};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

use crate::{errors::ApiResult, schemas::*};

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
                            println!(
                                "{}",
                                serde_json::to_string(&XoClientEvent {
                                    event: XoClientEventKind::Play,
                                    data: Some(XoClientEventsData::Play { place: 0 })
                                })
                                .unwrap()
                            );
                            if let Err(err) = handle_event(
                                event,
                                &conn,
                                max_online_games.as_ref(),
                                *move_period.as_ref(),
                                tx.clone(),
                                user_uuid.clone(),
                            )
                            .await
                            {
                                let _ = tx.send(
                                    XoServerEventData::Error(ErrorData::Other(err.to_string()))
                                        .into(),
                                );
                            }
                        } else {
                            let _ =
                                tx.send(XoServerEventData::Error(ErrorData::UnknownEvent).into());
                        }
                    } else {
                        let _ = tx.send(XoServerEventData::Error(ErrorData::InvalidBody).into());
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
        _ => {
            let _ = tx.send(XoServerEventData::Error(ErrorData::InvalidBody).into());
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
        let _ = player
            .1
            .send(XoServerEventData::Error(ErrorData::AlreadyInGame).into());
    } else if &ONLINE_GAMES.online_games_count().await >= max_online_games {
        log::error!(
            "Player {} can't join the game because the max online games is reached",
            player.0
        );
        let _ = player
            .1
            .send(XoServerEventData::Error(ErrorData::MaxGamesReached).into());
    } else if SEARCH_FOR_GAME.is_user_in_search(player.0.as_ref()).await {
        log::error!("Player {} is already in the search queue", player.0);
        let _ = player
            .1
            .send(XoServerEventData::Error(ErrorData::AlreadyInSearch).into());
    } else if SEARCH_FOR_GAME.search_users_count().await > 0 {
        let Some(other_player) = SEARCH_FOR_GAME.pop_front().await else {
            unreachable!("The search queue is not empty")
        };

        log::info!(
            "Player {} found a game with player {}",
            player.0,
            other_player.0
        );

        let now = chrono::Utc::now().naive_utc();
        let game = GameActiveModel {
            uuid: Set(Uuid::new_v4()),
            round: Set(1i16),
            auto_play_after: Set(Some(now + Duration::seconds(move_period))),
            rounds_result: Set(RoundsResult::default().to_string()),
            x_player: Set(*other_player.0.as_ref()),
            o_player: Set(*player.0.as_ref()),
            board: Set(Board::default().to_string()),
            winner: Set(None),
            reason: Set(None),
            created_at: Set(now),
            ..Default::default()
        }
        .save(conn)
        .await?;

        ONLINE_GAMES
            .add_game(*game.uuid.as_ref(), other_player.clone(), player.clone())
            .await;

        ONLINE_GAMES
            .broadcast_message(
                *game.uuid.as_ref(),
                XoServerEventData::GameFound {
                    x_player: *other_player.0.as_ref(),
                    o_player: *player.0.as_ref(),
                }
                .to_message(),
            )
            .await;

        ONLINE_GAMES
            .broadcast_message(
                *game.uuid.as_ref(),
                XoServerEventData::RoundStart { round: 1 }.to_message(),
            )
            .await;

        let _ = other_player.1.send(
            XoServerEventData::YourTurn {
                auto_play_after: game.auto_play_after.as_ref().unwrap().timestamp(),
            }
            .into(),
        );
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

        let mut game = GameEntity::find()
            .filter(GameColumn::Uuid.eq(game_uuid))
            .one(conn)
            .await?
            .expect("The game is in the online games so it should be here");
        let mut board = Board::from_str(&game.board).expect("The board is valid");

        let (player_symbol, versus_symbol) = if game.x_player == *player.0.as_ref() {
            (XoSymbol::X, XoSymbol::O)
        } else {
            (XoSymbol::O, XoSymbol::X)
        };

        if board.turn() != player_symbol {
            log::error!("Player {} is playing while it's not his turn", player.0);

            let _ = player.1.send(Ok(
                XoServerEventData::Error(ErrorData::NotYourTurn).to_message()
            ));
        } else if place > 8 || !board.is_empty_cell(place) {
            log::error!(
                "Player {} is playing in an non empty cell or invalid place",
                player.0
            );

            let _ = player.1.send(Ok(
                XoServerEventData::Error(ErrorData::InvalidPlace).to_message()
            ));
        } else if board.is_draw() || board.is_win(XoSymbol::X) || board.is_win(XoSymbol::O) {
            log::error!("Player {} is playing while the round is over", player.0);

            let _ = player.1.send(Ok(
                XoServerEventData::Error(ErrorData::NotYourTurn).to_message()
            ));
        } else {
            let _ = versus_player
                .1
                .send(XoServerEventData::Play(PlayData::new(place, *player.0.as_ref())).into());
            game.auto_play_after =
                Some((chrono::Utc::now() + Duration::seconds(move_period)).naive_utc());
            board.set_cell(place, player_symbol);
            let mut rounds_result =
                RoundsResult::from_str(&game.rounds_result).expect("The rounds result is valid");

            if board.is_win(player_symbol) {
                log::info!("Player {} won the round {}", player.0, game.round);
                rounds_result.add_win(player_symbol);
            } else if board.is_draw() {
                log::info!("The round {} is a draw", game.round);
                rounds_result.draws += 1;
            }

            // Check if the game is over
            if game.round == 3 && board.is_end() {
                let game_over_data = match rounds_result
                    .wins(player_symbol)
                    .cmp(&rounds_result.wins(versus_symbol))
                {
                    Ordering::Greater => {
                        GameOverData::new(Some(*player.0.as_ref()), GameOverReason::PlayerWon)
                    }
                    Ordering::Less => GameOverData::new(
                        Some(*versus_player.0.as_ref()),
                        GameOverReason::PlayerWon,
                    ),
                    Ordering::Equal => GameOverData::new(None, GameOverReason::Draw),
                };

                ONLINE_GAMES
                    .broadcast_message(
                        game.uuid,
                        XoServerEventData::GameOver(game_over_data.clone()).to_message(),
                    )
                    .await;

                rounds_result.add_board(board.clone());
                game.winner = game_over_data.winner;
                game.reason = Some(game_over_data.reason.to_string());
                ONLINE_GAMES.remove_game(conn, &game_uuid).await?;
            } else if game.round == 2
                && (rounds_result.x_player == 2 || rounds_result.o_player == 2)
            {
                let game_over_data = GameOverData::new(
                    Some(if rounds_result.wins(player_symbol) == 2 {
                        *player.0.as_ref()
                    } else {
                        *versus_player.0.clone().as_ref()
                    }),
                    GameOverReason::PlayerWon,
                );
                ONLINE_GAMES
                    .broadcast_message(
                        game.uuid,
                        XoServerEventData::GameOver(game_over_data.clone()).to_message(),
                    )
                    .await;

                rounds_result.add_board(board.clone());
                game.winner = game_over_data.winner;
                game.reason = Some(game_over_data.reason.to_string());
                ONLINE_GAMES.remove_game(conn, &game_uuid).await?;
            } else if board.is_end() {
                // ^^ Check if the round is over
                rounds_result.add_board(board.clone());
                ONLINE_GAMES
                    .broadcast_message(
                        game.uuid,
                        XoServerEventData::RoundEnd(RoundData::new(
                            game.round,
                            if board.is_win(player_symbol) {
                                Some(*player.0.as_ref())
                            } else {
                                None
                            },
                        ))
                        .to_message(),
                    )
                    .await;
                ONLINE_GAMES
                    .broadcast_message(
                        game.uuid,
                        XoServerEventData::RoundStart {
                            round: game.round + 1,
                        }
                        .to_message(),
                    )
                    .await;

                board = Board::default();
                game.round += 1;
                let data = Ok((XoServerEventData::YourTurn {
                    auto_play_after: game.auto_play_after.unwrap().timestamp(),
                })
                .to_message());
                if player_symbol == XoSymbol::X {
                    let _ = player.1.send(data);
                } else {
                    let _ = versus_player.1.send(data);
                }
            } else {
                let _ = versus_player.1.send(
                    XoServerEventData::YourTurn {
                        auto_play_after: game.auto_play_after.unwrap().timestamp(),
                    }
                    .into(),
                );
            }

            let mut game = game.into_active_model();
            game.board = Set(board.to_string());
            game.rounds_result = Set(rounds_result.to_string());
            game.round = Set(game.round.unwrap());
            game.auto_play_after = Set(game.auto_play_after.unwrap());
            game.winner = Set(game.winner.unwrap());
            game.reason = Set(game.reason.unwrap());
            game.save(conn).await?;
        }
    } else {
        let _ = player.1.send(Ok(
            XoServerEventData::Error(ErrorData::NotInGame).to_message()
        ));
    }
    Ok(())
}

/// The user disconnected handler.
async fn player_disconnected(conn: &sea_orm::DatabaseConnection, player: Player) -> ApiResult<()> {
    log::info!("Player {} disconnected", player.0);

    // If the player disconnected while in a game, then the other player will win.
    if let Some((game_uuid, versus_player)) = ONLINE_GAMES.get_user_game(&player.0).await {
        log::info!("Player {} disconnected while in a game", player.0);

        let game = GameEntity::find()
            .filter(GameColumn::Uuid.eq(game_uuid))
            .one(conn)
            .await?
            .expect("The game is in the online games so it should be here");
        let mut game = game.into_active_model();
        game.winner = Set(Some(*versus_player.0.as_ref()));
        game.reason = Set(Some(GameOverReason::PlayerDisconnected.to_string()));
        game.save(conn).await?;
        let _ = versus_player.1.send(
            XoServerEventData::GameOver(GameOverData::new(
                Some(*versus_player.0.as_ref()),
                GameOverReason::PlayerDisconnected,
            ))
            .into(),
        );
        ONLINE_GAMES.remove_game(conn, &game_uuid).await?;
    } else {
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
            let games = GameEntity::find()
                .filter(
                    GameColumn::EndedAt
                        .is_null()
                        .and(GameColumn::AutoPlayAfter.is_not_null()),
                )
                .all(conn)
                .await?;
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

                            let _ = player.1.send(XoServerEventData::AutoPlay { place }.into());

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
