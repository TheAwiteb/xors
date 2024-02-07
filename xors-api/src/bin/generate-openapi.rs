// A RESTful tic tac toy API for XORS project
// Copyright (C) 2024  Awiteb <Awiteb@pm.me>
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

// Include the test module, to get the connection ext...
include!("../../tests/mod.rs");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(openapi_path) = env::args().nth(1) else {
        eprintln!("Usage: generate_openapi <openapi_path>");
        std::process::exit(1);
    };

    let openapi = xors_api::api::service(get_connection().await?, 100, 10, get_secret_key()).1;
    std::fs::write(openapi_path, openapi.to_pretty_json()?)?;

    Ok(())
}
