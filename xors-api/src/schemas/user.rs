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

/// The delete user schema.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[salvo(schema(symbol = "DeleteUserSchema", example = json!(DeleteUserSchema::default())))]
pub struct DeleteUserSchema {
    /// The user's password.
    pub password: String,
}

/// The update user schema.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[salvo(schema(symbol = "UpdateUserSchema", example = json!(UpdateUserSchema::default())))]
pub struct UpdateUserSchema {
    /// The user's first name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// The user's last name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
}

impl Default for DeleteUserSchema {
    fn default() -> Self {
        Self {
            password: "password".to_owned(),
        }
    }
}

impl Default for UpdateUserSchema {
    fn default() -> Self {
        Self {
            first_name: Some("first_name".to_owned()),
            last_name: Some("last_name".to_owned()),
        }
    }
}
