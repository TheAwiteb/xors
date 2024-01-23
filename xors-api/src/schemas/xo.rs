// A API for xors (Xo game)
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

pub use self::api::*;
pub use self::websocket::*;

mod websocket {
    use salvo::websocket::Message;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    /// The Xo websocket server event.
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct XoServerEvent {
        /// The event name.
        pub event: XoServerEventKind,
        /// The event data.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<XoServerEventData>,
    }

    /// The Xo server events kind.
    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum XoServerEventKind {
        /// The game found event. Means that the server found a match for the player.
        GameFound,
        /// The turn event. Means that it's the player's turn.
        YourTurn,
        /// The round start event with the round number of 3 rounds. starting from 1.
        RoundStart,
        /// The round end event.
        RoundEnd,
        /// The play event. Means that the player played.
        Play,
        /// The auto play event. Means that the server played for the player.
        AutoPlay,
        /// The game over event with the game over data.
        GameOver,
        /// The error event with the error data.
        Error,
    }

    /// The Xo websocket server events.
    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    #[serde(untagged)]
    pub enum XoServerEventData {
        /// The game found event. Means that the server found a match for the player.
        GameFound { x_player: Uuid, o_player: Uuid },
        /// The turn event. Means that it's the player's turn.
        YourTurn { auto_play_after: i64 },
        /// The round start event with the round number of 3 rounds. starting from 1.
        RoundStart { round: i16 },
        /// The round end event.
        RoundEnd(RoundData),
        /// The play event. Means that the player played.
        Play(PlayData),
        /// The auto play event. Means that the server played for the player.
        AutoPlay { place: u8 },
        /// The game over event with the game over data.
        GameOver(GameOverData),
        /// The error event with the error data.
        Error(ErrorData),
    }

    /// The Xo websocket client event.
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct XoClientEvent {
        /// The event name.
        pub event: XoClientEventKind,
        /// The event data.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<XoClientEventsData>,
    }

    /// The Xo client events kind.
    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum XoClientEventKind {
        /// The search event. Means that the player want to search for a game.
        Search,
        /// The play event. Means that the player want to play.
        Play,
    }

    /// The Xo client events data.
    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    #[serde(untagged)]
    pub enum XoClientEventsData {
        /// The play data.
        Play { place: u8 },
    }

    /// The Xo play data.
    #[derive(Serialize, Deserialize, Clone, Debug, derive_new::new)]
    pub struct PlayData {
        /// The place that the player want to play in.
        /// The place is a number between 0 and 8.
        /// | 0 | 1 | 2 |
        /// | 3 | 4 | 5 |
        /// | 6 | 7 | 8 |
        pub place: u8,
        /// The player's uuid.
        pub player: Uuid,
    }

    /// The Xo game over data.
    #[derive(Serialize, Deserialize, Clone, Debug, derive_new::new)]
    pub struct GameOverData {
        /// The winner. if the winner is None, then the game is a draw.
        pub winner: Option<Uuid>,
        /// The game over reason.
        pub reason: GameOverReason,
    }

    /// The Xo round data.
    #[derive(Serialize, Deserialize, Clone, Debug, derive_new::new)]
    pub struct RoundData {
        /// The round number.
        pub round: i16,
        /// The winner. if the winner is None, then the round is a draw.
        pub winner: Option<Uuid>,
    }

    /// The Xo game found data.
    #[derive(Serialize, Deserialize, Clone, Debug, derive_new::new)]
    pub struct GameFoundData {
        /// The X player's uuid.
        pub x_player: Uuid,
        /// The O player's uuid.
        pub o_player: Uuid,
    }

    /// The Xo game over reason.
    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum GameOverReason {
        /// The game is over because the player won.
        PlayerWon,
        /// The game is over because is a draw.
        Draw,
        /// The game is over because the player left.
        PlayerDisconnected,
    }

    /// The Xo error reasons
    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum ErrorData {
        /// Invalid body. (The body must be json)
        InvalidBody,
        /// Unknown event. (The event is not supported)
        UnknownEvent,
        /// The event data is not valid for the event kind.
        InvalidEventDataForEvent,
        /// Already in search. (You can't search for a game while you are in search)
        AlreadyInSearch,
        /// Already in game. (You can't search for a game while you are in a game)
        AlreadyInGame,
        /// Not in game. (You can't play while you are not in a game)
        NotInGame,
        /// Not your turn. (You can't play now)
        NotYourTurn,
        /// Invalid place. (The place is already taken)
        InvalidPlace,
        /// Maximum online games reached. (Depends on the server configuration)
        MaxGamesReached,
        /// Other error.
        #[serde(untagged)]
        Other(String),
    }

    impl XoServerEventData {
        /// Returns the event data as XoServerEvent `[Message]` of json.
        pub fn to_message(&self) -> Message {
            Message::text(serde_json::to_string(&XoServerEvent::from(self)).unwrap())
        }
    }

    impl XoServerEvent {
        /// Returns the event as `[Message]` of json.
        pub fn to_message(&self) -> Message {
            Message::text(serde_json::to_string(self).unwrap())
        }
    }

    impl XoServerEventData {
        /// Returns the kind of the event.
        pub fn kind(&self) -> XoServerEventKind {
            match self {
                Self::GameFound { .. } => XoServerEventKind::GameFound,
                Self::YourTurn { .. } => XoServerEventKind::YourTurn,
                Self::RoundStart { .. } => XoServerEventKind::RoundStart,
                Self::RoundEnd(_) => XoServerEventKind::RoundEnd,
                Self::Play(_) => XoServerEventKind::Play,
                Self::AutoPlay { .. } => XoServerEventKind::AutoPlay,
                Self::GameOver(_) => XoServerEventKind::GameOver,
                Self::Error(_) => XoServerEventKind::Error,
            }
        }
    }

    impl From<XoServerEventData> for Result<Message, salvo::Error> {
        fn from(value: XoServerEventData) -> Self {
            Ok(XoServerEvent::from(&value).to_message())
        }
    }

    impl ToString for GameOverReason {
        fn to_string(&self) -> String {
            match self {
                GameOverReason::PlayerWon => "Player Won".to_owned(),
                GameOverReason::Draw => "Draw".to_owned(),
                GameOverReason::PlayerDisconnected => "Player Disconnected".to_owned(),
            }
        }
    }

    impl From<&XoServerEventData> for XoServerEvent {
        fn from(value: &XoServerEventData) -> Self {
            Self {
                event: value.kind(),
                data: Some(value.clone()),
            }
        }
    }
}

/// API schemas.
mod api {
    use std::str::FromStr;

    use salvo::prelude::*;
    use serde::Serialize;

    /// The Xo symbol.
    #[derive(Serialize, PartialEq, Eq, Clone, Copy, Debug, ToSchema)]
    #[salvo(symbol = "XoSymbolSchema", example = XoSymbol::X)]
    pub enum XoSymbol {
        /// The X symbol.
        X,
        /// The O symbol.
        O,
    }

    /// The XO game board.
    #[derive(Serialize, Clone, Debug, Default, ToSchema)]
    #[salvo(symbol = "BoardSchema", example = json!(Board::default()))]
    pub struct Board {
        /// The board cells.
        #[salvo(madx = 9)]
        cells: [Option<XoSymbol>; 9],
        /// The sequence of the played cells.
        /// It's shows the played cells in order by the index.
        played_cells: Vec<u8>,
    }

    /// The XO rounds result.
    #[derive(Serialize, Clone, Debug, Default, ToSchema)]
    #[salvo(symbol = "RoundsResultSchema")]
    pub struct RoundsResult {
        /// The X player rounds wins.
        pub x_player: usize,
        /// The O player rounds wins.
        pub o_player: usize,
        /// The rounds draws.
        pub draws: usize,
        /// All the rounds boards.
        boards: Vec<Board>,
    }

    impl RoundsResult {
        /// Returns the wins of the symbol.
        pub fn wins(&self, symbol: XoSymbol) -> usize {
            match symbol {
                XoSymbol::X => self.x_player,
                XoSymbol::O => self.o_player,
            }
        }

        /// Add a win to the symbol.
        pub fn add_win(&mut self, symbol: XoSymbol) {
            match symbol {
                XoSymbol::X => self.x_player += 1,
                XoSymbol::O => self.o_player += 1,
            }
        }

        /// Add board to the rounds result.
        /// The board must be of an ended round.
        ///
        /// ### Panics
        /// - Panics if the total rounds is more than 3, which is impossible because the game max rounds is 3.
        /// - Panics if the board is not end. will check if the board is end by calling `is_end` method.
        pub fn add_board(&mut self, board: Board) {
            let index = self.x_player + self.o_player + self.draws;
            assert!(index < 3, "The total rounds is more than 3");
            assert!(board.is_end(), "The board is not end");
            self.boards.push(board);
        }
    }

    impl Board {
        /// Set the board cell.
        ///
        /// ### Panics
        /// - Panics if the index is not between 0 and 8.
        pub fn set_cell(&mut self, index: u8, symbol: XoSymbol) {
            assert!(index <= 8, "The index must be between 0 and 8");

            self.cells[index as usize] = Some(symbol);
            self.played_cells.push(index);
        }

        /// Check if the cell is empty.
        pub fn is_empty_cell(&self, index: u8) -> bool {
            self.cells[index as usize].is_none()
        }

        /// Returns the empty cells.
        pub fn empty_cells(&self) -> Vec<u8> {
            (0..=8)
                .filter(|&index| self.is_empty_cell(index))
                .collect::<Vec<_>>()
        }

        /// Check if the board is full.
        pub fn is_full(&self) -> bool {
            self.cells.iter().all(|cell| cell.is_some())
        }

        /// Check if the board is end.
        /// The board is draw or someone won.
        pub fn is_end(&self) -> bool {
            self.is_draw() || self.is_win(XoSymbol::X) || self.is_win(XoSymbol::O)
        }

        /// Returns the symbol turn.
        ///
        /// ## Explanation
        /// In the XO game the X symbol always starts first, so the X will play 5 times and the O will play 4 times.
        /// So the O can't play more than the X. The O plays must be (X plays - 1), like that we can know which symbol turn is it.
        /// In another words, if the count of played cells is even then it's the X turn, else it's the O turn.
        pub fn turn(&self) -> XoSymbol {
            if self.cells.iter().filter(|cell| cell.is_some()).count() % 2 == 0 {
                XoSymbol::X
            } else {
                XoSymbol::O
            }
        }

        /// Check if the board is a draw.
        pub fn is_draw(&self) -> bool {
            self.is_full() && !self.is_win(XoSymbol::X) && !self.is_win(XoSymbol::O)
        }

        /// Check if the symbol is win.
        pub fn is_win(&self, symbol: XoSymbol) -> bool {
            crate::api::xo::WINNING_COMBINATIONS
                .iter()
                .any(|&(a, b, c)| {
                    self.cells[a] == Some(symbol)
                        && self.cells[b] == Some(symbol)
                        && self.cells[c] == Some(symbol)
                })
        }
    }

    impl ToString for XoSymbol {
        fn to_string(&self) -> String {
            match self {
                XoSymbol::X => "X".to_owned(),
                XoSymbol::O => "O".to_owned(),
            }
        }
    }

    impl ToString for Board {
        fn to_string(&self) -> String {
            format!(
                "{}:{}",
                self.cells
                    .iter()
                    .map(|cell| match cell {
                        Some(symbol) => symbol.to_string(),
                        None => "-".to_owned(),
                    })
                    .collect::<Vec<_>>()
                    .join(""),
                self.played_cells
                    .iter()
                    .map(|cell| cell.to_string())
                    .collect::<Vec<_>>()
                    .join("")
            )
        }
    }

    impl ToString for RoundsResult {
        fn to_string(&self) -> String {
            format!(
                "{}{}{} {}",
                "X".repeat(self.x_player),
                "O".repeat(self.o_player),
                "-".repeat(self.draws),
                self.boards
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
    }

    /// The XO symbol. The symbol is either "X" or "O" and "-" for empty cell.
    ///
    /// # Errors
    /// - Will return `Err(())`  if there is an invalid symbol or the string length is not 9.
    /// - Will return `Err(())` if the played cells is invalid
    ///
    /// # Examples
    /// - "XOXOXOXOX:03214658" is valid.
    impl FromStr for Board {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (str_board, played_cells) = s.split_once(':').ok_or(())?;
            let board_chars = str_board.chars().collect::<Vec<_>>();

            if board_chars.len() != 9 {
                return Err(());
            }
            if played_cells.chars().count() > 9 || played_cells.chars().any(|c| !c.is_ascii_digit())
            {
                return Err(());
            }

            let mut board = Self::default();
            for index in played_cells
                .chars()
                .map(|c| c.to_digit(10).expect("The digit is valid") as usize)
            {
                if index > 8 {
                    return Err(());
                }

                let symbol = match board_chars
                    .get(index)
                    .expect("The index is valid because the played cells is valid")
                {
                    'X' => XoSymbol::X,
                    'O' => XoSymbol::O,
                    _ => return Err(()),
                };
                board.set_cell(index as u8, symbol);
            }
            Ok(board)
        }
    }

    impl FromStr for RoundsResult {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (rounds_result, boards) = s.split_once(' ').ok_or(())?;
            Ok(Self {
                x_player: rounds_result.chars().filter(|c| c == &'X').count(),
                o_player: rounds_result.chars().filter(|c| c == &'O').count(),
                draws: rounds_result.chars().filter(|c| c == &'-').count(),
                boards: if boards.is_empty() {
                    Vec::new()
                } else {
                    boards
                        .trim()
                        .split(',')
                        .map(|board| Board::from_str(board).expect("The board is valid"))
                        .collect()
                },
            })
        }
    }
}
