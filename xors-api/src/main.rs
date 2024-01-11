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

use std::env;

use base64::engine::GeneralPurpose;
use migration::{Migrator, MigratorTrait};
use salvo::prelude::*;

pub mod api;
pub mod db_utils;
pub mod errors;
pub mod schemas;
pub mod utils;

pub const BASE_64_ENGINE: GeneralPurpose = GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    base64::engine::general_purpose::PAD,
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let host = env::var("XORS_API_HOST").expect("`XORS_API_HOST` environment variable must be set");
    let port = env::var("XORS_API_PORT").expect("`XORS_API_PORT` environment variable must be set");
    let connection = sea_orm::Database::connect(
        env::var("XORS_API_DATABASE_URL")
            .expect("`XORS_API_DATABASE_URL` environment variable must be set"),
    )
    .await?;
    let secret_key = env::var("XORS_API_SECRET_KEY")
        .expect("`XORS_API_SECRET_KEY` environment variable must be set");

    log::debug!("Connected to the database");
    Migrator::up(&connection, None).await?;

    log::info!("Starting API on http://{host}:{port}");
    log::info!("The OpenAPI spec is available at http://{host}:{port}/api-doc/openapi.json");
    log::info!("The ReDoc documentation is available at http://{host}:{port}/api-doc/swagger-ui");
    log::info!("Press Ctrl+C to stop the API");

    let acceptor = salvo::conn::TcpListener::new(format!("{host}:{port}"))
        .bind()
        .await;
    let server_handler = tokio::spawn(async move {
        Server::new(acceptor)
            .serve(api::service(connection, secret_key))
            .await
    });

    server_handler.await?;
    log::info!("API is shutting down");

    Ok(())
}
