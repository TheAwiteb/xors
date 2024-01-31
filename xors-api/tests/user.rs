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

include!("mod.rs");

#[cfg(test)]
mod get_me {
    use super::*;

    #[tokio::test]
    async fn get_me_success() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_get_me".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let mut res = send(
            &service,
            "user/me",
            Method::GET,
            None::<&()>,
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "The response should have a `OK` status code {res:?}"
        );
        let res_json: UserSchema =
            serde_json::from_str(&res.take_string().await.expect("Failed to get body"))
                .expect("Failed to parse body");

        assert_eq!(res_json, user.user, "User should be the same");
    }

    #[tokio::test]
    async fn get_me_with_refresh_token() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_get_me_with_refresh_token".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user/me",
            Method::GET,
            None::<&()>,
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.refresh_token)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn get_me_no_auth() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(&service, "user/me", Method::GET, None::<&()>, vec![]).await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::UNAUTHORIZED),
            "The response should have a `UNAUTHORIZED` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn get_me_invalid_auth() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            "user/me",
            Method::GET,
            None::<&()>,
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str("Bearer invalid").unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::FORBIDDEN),
            "The response should have a `FORBIDDEN` status code {res:?}"
        );
    }
}

#[cfg(test)]
mod get_user {
    use super::*;

    #[tokio::test]
    async fn get_user_success() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_get_user_success".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let mut res = send(
            &service,
            &format!("user?uuid={}", user.user.uuid),
            Method::GET,
            None::<&()>,
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "The response should have a `OK` status code {res:?}"
        );
        let res_json: UserSchema =
            serde_json::from_str(&res.take_string().await.expect("Failed to get body"))
                .expect("Failed to parse body");
        assert_eq!(res_json, user.user, "User should be the same");
    }

    #[tokio::test]
    async fn get_user_without_uuid() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(&service, "user", Method::GET, None::<&()>, vec![]).await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn get_user_invalid_uuid() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            "user?uuid=invalid",
            Method::GET,
            None::<&()>,
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn get_user_not_found() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            &format!("user?uuid={}", Uuid::new_v4()),
            Method::GET,
            None::<&()>,
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::NOT_FOUND),
            "The response should have a `NOT_FOUND` status code {res:?}"
        );
    }
}

#[cfg(test)]
mod update_user {
    use super::*;

    #[tokio::test]
    async fn update_user_first_name() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_update_user_first_name".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let mut res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some("NewFirst".to_string()),
                last_name: user.user.last_name.clone(),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "The response should have a `OK` status code {res:?}"
        );
        let res_json: UserSchema =
            serde_json::from_str(&res.take_string().await.expect("Failed to get body"))
                .expect("Failed to parse body");
        assert_eq!(
            res_json.first_name, "NewFirst",
            "First name should be updated"
        );
        assert_eq!(
            res_json.last_name, user.user.last_name,
            "Last name should not be updated"
        );
    }

    #[tokio::test]
    async fn update_user_last_name() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_update_user_last_name".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let mut res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some(user.user.first_name.clone()),
                last_name: Some("NewLast".to_string()),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "The response should have a `OK` status code {res:?}"
        );
        let res_json: UserSchema =
            serde_json::from_str(&res.take_string().await.expect("Failed to get body"))
                .expect("Failed to parse body");
        assert_eq!(
            res_json.first_name, user.user.first_name,
            "First name should not be updated"
        );
        assert_eq!(
            res_json.last_name.as_deref(),
            Some("NewLast"),
            "Last name should be updated"
        );
    }

    #[tokio::test]
    async fn update_user_with_invalid_first_name() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_update_user_with_invalid_first_name".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some("".to_string()),
                last_name: user.user.last_name.clone(),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some("    ".to_string()),
                last_name: user.user.last_name.clone(),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some("First First".to_string()),
                last_name: user.user.last_name.clone(),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some("Long".repeat(20)),
                last_name: user.user.last_name.clone(),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn update_user_with_invalid_last_name() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_update_user_with_invalid_last_name".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some(user.user.first_name.clone()),
                last_name: Some("".to_string()),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some(user.user.first_name.clone()),
                last_name: Some("    ".to_string()),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;
        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some(user.user.first_name.clone()),
                last_name: Some("Last Last".to_string()),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;
        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some(user.user.first_name.clone()),
                last_name: Some("Long".repeat(20)),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn update_user_with_null_first_name() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_update_user_with_null_first_name".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: None,
                last_name: user.user.last_name.clone(),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn update_user_without_auth() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some("First".to_string()),
                last_name: Some("Last".to_string()),
                profile_image: None,
            }),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::UNAUTHORIZED),
            "The response should have a `UNAUTHORIZED` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn update_user_with_invalid_auth() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some("First".to_string()),
                last_name: Some("Last".to_string()),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str("Bearer invalid").unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::FORBIDDEN),
            "The response should have a `FORBIDDEN` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn update_user_with_refresh_token() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_update_user_with_refresh_token".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user",
            Method::PUT,
            Some(&UpdateUserSchema {
                first_name: Some("First".to_string()),
                last_name: Some("Last".to_string()),
                profile_image: None,
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.refresh_token)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }
}

#[cfg(test)]
mod delete_user {
    use super::*;

    #[tokio::test]
    async fn delete_user_success() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_delete_user_success".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user",
            Method::DELETE,
            Some(&DeleteUserSchema {
                password: "kdfkl(#0()$fkLKJF".to_string(),
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "The response should have a `OK` status code {res:?}"
        );

        assert!(
            xors_api::db_utils::get_user(&conn, user.user.uuid)
                .await
                .is_err(),
            "User should be deleted"
        );
    }

    #[tokio::test]
    async fn delete_user_with_invalid_password() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_delete_user_with_invalid_password".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user",
            Method::DELETE,
            Some(&DeleteUserSchema {
                password: "Invalid".to_string(),
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn delete_user_without_auth() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(&service, "user", Method::DELETE, None::<&()>, vec![]).await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::UNAUTHORIZED),
            "The response should have a `UNAUTHORIZED` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn delete_user_with_invalid_auth() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            "user",
            Method::DELETE,
            Some(&DeleteUserSchema {
                password: "kdfkl(#0()$fkLKJF".to_string(),
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str("Bearer invalid").unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::FORBIDDEN),
            "The response should have a `FORBIDDEN` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn delete_user_with_refresh_token() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = xors_api::db_utils::signin_user(
            xors_api::db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_delete_user_with_refresh_token".to_string(),
                    password: "kdfkl(#0()$fkLKJF".to_string(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user",
            Method::DELETE,
            Some(&DeleteUserSchema {
                password: "kdfkl(#0()$fkLKJF".to_string(),
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.refresh_token)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }
}

#[cfg(test)]
mod reset_password {
    use super::*;

    #[tokio::test]
    async fn reset_password_success() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        const OLD_PASSWORD: &str = "kdfkl(#0()$fkLKJF";
        const NEW_PASSWORD: &str = "kdfkl(#0()$fkLKJf";

        let user = db_utils::signin_user(
            db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "reset_password_success".to_string(),
                    password: OLD_PASSWORD.to_owned(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user/reset_password",
            Method::POST,
            Some(&ResetPasswordSchema {
                old_password: OLD_PASSWORD.to_owned(),
                new_password: NEW_PASSWORD.to_owned(),
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "The response should have a `OK` status code {res:?}"
        );

        let user_with_new_password = db_utils::get_user(&conn, user.user.uuid)
            .await
            .expect("Failed to get user");
        assert!(
            bcrypt::verify(NEW_PASSWORD, user_with_new_password.password_hash.as_ref())
                .expect("Failed to verify password"),
            "Password should be updated"
        );
    }

    #[tokio::test]
    async fn reset_password_with_invalid_old_password() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        const OLD_PASSWORD: &str = "kdfkl(#0()$fkLKJF";
        const NEW_PASSWORD: &str = "kdfkl(#0()$fkLKJf";

        let user = db_utils::signin_user(
            db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "reset_password_with_invalid_old".to_string(),
                    password: OLD_PASSWORD.to_owned(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user/reset_password",
            Method::POST,
            Some(&ResetPasswordSchema {
                old_password: "InvalidOldPassword".to_owned(),
                new_password: NEW_PASSWORD.to_owned(),
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn reset_password_without_auth() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            "user/reset_password",
            Method::POST,
            Some(&ResetPasswordSchema {
                old_password: "kdfkl(#0()$fkLKJF".to_string(),
                new_password: "kdfkl(#0()$fkLKJf".to_string(),
            }),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::UNAUTHORIZED),
            "The response should have a `UNAUTHORIZED` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn reset_password_with_invalid_auth() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            "user/reset_password",
            Method::POST,
            Some(&ResetPasswordSchema {
                old_password: "kdfkl(#0()$fkLKJF".to_string(),
                new_password: "kdfkl(#0()$fkLKJf".to_string(),
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str("Bearer invalid").unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::FORBIDDEN),
            "The response should have a `FORBIDDEN` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn reset_password_with_same_password() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        const OLD_PASSWORD: &str = "kdfkl(#0()$fkLKJF";

        let user = db_utils::signin_user(
            db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "username_reset_password_with_same_password".to_string(),
                    password: OLD_PASSWORD.to_owned(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user/reset_password",
            Method::POST,
            Some(&ResetPasswordSchema {
                old_password: OLD_PASSWORD.to_owned(),
                new_password: OLD_PASSWORD.to_owned(),
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn reset_password_with_invalid_new_password() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        const OLD_PASSWORD: &str = "kdfkl(#0()$fkLKJF";

        let user = db_utils::signin_user(
            db_utils::create_user(
                &conn,
                NewUserSchema {
                    first_name: "First".to_string(),
                    last_name: Some("Last".to_string()),
                    username: "reset_password_with_invalid_new".to_string(),
                    password: OLD_PASSWORD.to_owned(),
                },
            )
            .await
            .expect("Failed to create user"),
            &secret_key,
        )
        .await
        .expect("Failed to signin user");

        let res = send(
            &service,
            "user/reset_password",
            Method::POST,
            Some(&ResetPasswordSchema {
                old_password: "kdfkl(#0()$fkLKJF".to_string(),
                new_password: "".to_string(),
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );

        let res = send(
            &service,
            "user/reset_password",
            Method::POST,
            Some(&ResetPasswordSchema {
                old_password: "kdfkl(#0()$fkLKJF".to_string(),
                new_password: "    ".to_string(),
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );

        let res = send(
            &service,
            "user/reset_password",
            Method::POST,
            Some(&ResetPasswordSchema {
                old_password: "kdfkl(#0()$fkLKJF".to_string(),
                new_password: "kdfkl(#0()$fkLKJF".repeat(20),
            }),
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", user.jwt)).unwrap(),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `BAD_REQUEST` status code {res:?}"
        );
    }
}
