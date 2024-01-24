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

use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

mod game;
mod jwt;
mod user;
mod xo;

pub use {game::*, jwt::*, user::*, xo::*};

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, derive_new::new)]
#[salvo(schema(symbol = "MessageSchema", example = json!(MessageSchema::new("Message".to_owned()))))]
pub struct MessageSchema {
    #[salvo(schema(example = "Message"))]
    message: String,
}
