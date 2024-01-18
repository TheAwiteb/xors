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

use entity::prelude::*;
use salvo::prelude::*;
use sea_orm::DatabaseConnection;

use crate::errors::{ApiError, ApiResult};

use super::jwt::JwtClaims;

pub trait UserExt {
    /// Returns the user that make the request.
    fn user(
        &self,
        conn: &DatabaseConnection,
    ) -> impl std::future::Future<Output = ApiResult<UserModel>> + Send;

    /// Returns the JWT claims of the user that make the request.
    fn jwt_claims(&self) -> &JwtClaims;
}

impl UserExt for Depot {
    async fn user(&self, conn: &DatabaseConnection) -> ApiResult<UserModel> {
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

    fn jwt_claims(&self) -> &JwtClaims {
        &self
            .jwt_auth_data::<JwtClaims>()
            .expect("The user is authorized so it should be here")
            .claims
    }
}
