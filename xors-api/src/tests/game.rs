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

use super::*;

#[cfg(test)]
mod get_game {
    use super::*;
    use crate::db_utils;

    #[tokio::test]
    async fn get_game_success() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");

        let player_x = db_utils::create_user(
            &conn,
            NewUserSchema {
                username: "get_game_succses_x_player".to_owned(),
                first_name: "Player".to_owned(),
                password: "fdkDFLKJL4859#$&".to_owned(),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create player x");
        let player_o = db_utils::create_user(
            &conn,
            NewUserSchema {
                username: "get_game_succses_o_player".to_owned(),
                first_name: "Player".to_owned(),
                password: "fdkDFLKJL4859#$&".to_owned(),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create player x");

        let game = db_utils::create_game(&conn, player_x.uuid, player_o.uuid, 10)
            .await
            .expect("Failed to create game");
        db_utils::end_game(&conn, game.uuid.as_ref())
            .await
            .expect("Failed to end game");

        let mut res = send(
            &service,
            &format!("game/{}", game.uuid.as_ref()),
            Method::GET,
            None::<&()>,
            vec![],
        )
        .await;

        let res_game: GameSchema = serde_json::from_str(&res.take_string().await.unwrap())
            .expect("Failed to parse game schema");

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "Status code should be 200 {res:?}"
        );
        assert_eq!(game.x_player.as_ref(), &res_game.x_player.uuid);
        assert_eq!(game.o_player.as_ref(), &res_game.o_player.uuid);
    }

    #[tokio::test]
    async fn get_game_not_found() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            &format!("game/{}", Uuid::new_v4()),
            Method::GET,
            None::<&()>,
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::NOT_FOUND),
            "Status code should be 404 {res:?}"
        );
    }

    #[tokio::test]
    async fn get_game_invalid_uuid() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            "game/invalid-uuid",
            Method::GET,
            None::<&()>,
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "Status code should be 400 {res:?}"
        );
    }

    /// If the the user uuid not found in the database, deleted user should be returned instead
    #[tokio::test]
    async fn deleted_user() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");

        let player_x = db_utils::create_user(
            &conn,
            NewUserSchema {
                username: "deleted_user_x_player".to_owned(),
                first_name: "Player".to_owned(),
                password: "fdkDFLKJL4859#$&".to_owned(),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create player x");

        let game = db_utils::create_game(&conn, player_x.uuid, Uuid::new_v4(), 10)
            .await
            .expect("Failed to create game");
        db_utils::end_game(&conn, game.uuid.as_ref())
            .await
            .expect("Failed to end game");

        let mut res = send(
            &service,
            &format!("game/{}", game.uuid.as_ref()),
            Method::GET,
            None::<&()>,
            vec![],
        )
        .await;

        let res_game: GameSchema = serde_json::from_str(&res.take_string().await.unwrap())
            .expect("Failed to parse game schema");

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "Status code should be 200 {res:?}"
        );
        assert_eq!(game.x_player.as_ref(), &res_game.x_player.uuid);
        assert_eq!(res_game.o_player.uuid, Uuid::nil());
        assert_eq!(res_game.o_player.username, "Deleted");
    }

    #[tokio::test]
    async fn unend_game() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");

        let player_x = db_utils::create_user(
            &conn,
            NewUserSchema {
                username: "unend_game_x_player".to_owned(),
                first_name: "Player".to_owned(),
                password: "fdkDFLKJL4859#$&".to_owned(),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create player x");
        let player_o = db_utils::create_user(
            &conn,
            NewUserSchema {
                username: "unend_game_o_player".to_owned(),
                first_name: "Player".to_owned(),
                password: "fdkDFLKJL4859#$&".to_owned(),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create player x");

        let game = db_utils::create_game(&conn, player_x.uuid, player_o.uuid, 10)
            .await
            .expect("Failed to create game");

        let res = send(
            &service,
            &format!("game/{}", game.uuid.as_ref()),
            Method::GET,
            None::<&()>,
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::NOT_FOUND),
            "Status code should be 404 {res:?}"
        );
    }
}

#[cfg(test)]
mod lastest_games {
    use super::*;
    use crate::db_utils;

    #[tokio::test]
    async fn lastest_games_success() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");

        let game1 = db_utils::create_game(&conn, Uuid::new_v4(), Uuid::new_v4(), 10)
            .await
            .expect("Failed to create game");
        let game2 = db_utils::create_game(&conn, Uuid::new_v4(), Uuid::new_v4(), 10)
            .await
            .expect("Failed to create game");
        db_utils::end_game(&conn, game1.uuid.as_ref())
            .await
            .expect("Failed to end game");
        db_utils::end_game(&conn, game2.uuid.as_ref())
            .await
            .expect("Failed to end game");

        let mut res = send(&service, "games", Method::GET, None::<&()>, vec![]).await;

        let res_games: Vec<GameSchema> = serde_json::from_str(&res.take_string().await.unwrap())
            .expect("Failed to parse game schema");

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "Status code should be 200 {res:?}"
        );

        assert!(
            res_games.iter().any(|g| &g.uuid == game1.uuid.as_ref()),
            "Game 1 should be in the response {res:?}"
        );
        assert!(
            res_games.iter().any(|g| &g.uuid == game2.uuid.as_ref()),
            "Game 2 should be in the response {res:?}"
        );
    }

    #[tokio::test]
    async fn unend_games() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");

        let game1 = db_utils::create_game(&conn, Uuid::new_v4(), Uuid::new_v4(), 10)
            .await
            .expect("Failed to create game");
        let game2 = db_utils::create_game(&conn, Uuid::new_v4(), Uuid::new_v4(), 10)
            .await
            .expect("Failed to create game");
        db_utils::end_game(&conn, game1.uuid.as_ref())
            .await
            .expect("Failed to end game");

        let mut res = send(&service, "games", Method::GET, None::<&()>, vec![]).await;

        let res_games: Vec<GameSchema> = serde_json::from_str(&res.take_string().await.unwrap())
            .expect("Failed to parse game schema");

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "Status code should be 200 {res:?}"
        );

        assert!(
            res_games.iter().any(|g| &g.uuid == game1.uuid.as_ref()),
            "Game 1 should be in the response {res:?}"
        );
        assert!(
            !res_games.iter().any(|g| &g.uuid == game2.uuid.as_ref()),
            "Game 2 should not be in the response {res:?}"
        );
    }
}
