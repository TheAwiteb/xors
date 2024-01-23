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

use std::env;

use base64::engine::GeneralPurpose;
use migration::{Migrator, MigratorTrait};
use salvo::prelude::*;

pub mod api;
pub mod db_utils;
pub mod errors;
pub mod schemas;
pub mod utils;

#[cfg(test)]
mod tests;

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
    let max_online_games = env::var("XORS_API_MAX_ONLINE_GAMES")
        .expect("`XORS_API_MAX_ONLINE_GAMES` environment variable must be set")
        .parse::<usize>()
        .expect("`XORS_API_MAX_ONLINE_GAMES` environment variable must be a number");
    let move_period = env::var("XORS_API_MOVE_PERIOD")
        .expect("`XORS_API_MOVE_PERIOD` environment variable must be set")
        .parse::<i64>()
        .expect("`XORS_API_MOVE_PERIOD` environment variable must be a number");
    if move_period < 0 {
        panic!("`XORS_API_MOVE_PERIOD` environment variable must be a positive number");
    }

    log::debug!("Connected to the database");
    Migrator::up(&connection, None).await?;

    log::info!("Starting API on http://{host}:{port}");
    log::info!("XO websocket is available at ws://{host}:{port}/xo");
    log::info!("The OpenAPI spec is available at http://{host}:{port}/api-doc/openapi.json");
    log::info!("The ReDoc documentation is available at http://{host}:{port}/api-doc/swagger-ui");
    log::info!("Press Ctrl+C to stop the API");

    let server_connection = connection.clone();
    let acceptor = salvo::conn::TcpListener::new(format!("{host}:{port}"))
        .bind()
        .await;
    let server_handler = tokio::spawn(async move {
        Server::new(acceptor)
            .serve(api::service(
                server_connection,
                max_online_games,
                move_period,
                secret_key,
            ))
            .await
    });
    let auto_play_handler = tokio::spawn(async move {
        api::xo::auto_play_handler(connection, move_period).await;
    });

    server_handler.await?;
    auto_play_handler.await?;
    log::info!("API is shutting down");

    Ok(())
}
