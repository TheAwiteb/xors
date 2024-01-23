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

pub mod jwt;
pub mod user;

use std::env;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;

use crate::errors::ApiResult;
use crate::schemas::*;
use chrono::Duration;
use entity::prelude::*;
use salvo::conn::SocketAddr;
use salvo::http::ReqBody;
use salvo::hyper::header::HeaderName;
use salvo::prelude::*;
use salvo::test::*;
use salvo::{
    http::HeaderValue,
    hyper::{header, Method},
};
use serde::Serialize;
use uuid::Uuid;

const API_URL: &str = "http://127.0.0.1:5801";

/// Function to send a request to the API.
pub async fn send<T>(
    service: &Service,
    path: &str,
    method: Method,
    body: Option<&T>,
    headers: Vec<(HeaderName, HeaderValue)>,
) -> Response
where
    T: Serialize + ?Sized,
{
    let mut req = RequestBuilder::new(format!("{}/{}", API_URL, path), method).build();
    if let Some(body) = body {
        *req.body_mut() = ReqBody::from(serde_json::to_string(body).unwrap());
        req.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
    };
    for (header_name, header_value) in headers {
        req.headers_mut().insert(header_name, header_value);
    }
    *req.local_addr_mut() = SocketAddr::IPv4(SocketAddrV4::new(Ipv4Addr::new(127, 1, 1, 1), 5802));
    *req.remote_addr_mut() = SocketAddr::IPv4(SocketAddrV4::new(Ipv4Addr::new(127, 1, 1, 2), 5802));
    service.call(req).await
}

/// Returns database connection.
pub async fn get_connection() -> ApiResult<sea_orm::DatabaseConnection> {
    Ok(sea_orm::Database::connect(
        env::var("XORS_API_DATABASE_URL")
            .expect("`XORS_API_DATABASE_URL` environment variable must be set"),
    )
    .await?)
}

/// Returns the secret key.
pub fn get_secret_key() -> String {
    env::var("XORS_API_SECRET_KEY").expect("`XORS_API_SECRET_KEY` environment variable must be set")
}

/// Returns the service.
pub async fn get_service() -> ApiResult<Service> {
    Ok(crate::api::service(
        get_connection().await?,
        100,
        10,
        get_secret_key(),
    ))
}
