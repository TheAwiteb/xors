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
mod signup {
    use super::*;

    #[tokio::test]
    async fn signup_succsess() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "Username".to_owned(),
            password: "fdlkFDLKF#$3213!".to_owned(),
        };

        let mut res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "The response should have a `200 OK` status code {res:?}"
        );

        let user_schema: UserSigninSchema = serde_json::from_str(
            &res.take_string()
                .await
                .expect("Could not get the response body"),
        )
        .expect("Failed to parse response body");

        assert_eq!(
            user_schema.user, signup_schema,
            "The user schema should be the same as the signup schema"
        );
    }

    #[tokio::test]
    async fn already_exist_username() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "Username001".to_owned(),
            password: "fdlkFDLKF#$3213!".to_owned(),
        };

        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "The response should have a `200 OK` status code {res:?}"
        );

        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn invalid_first_name() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "Username".to_owned(),
            password: "fdlkFDLKF#$3213!".to_owned(),
        };

        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn invalid_username_start_with_number() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "1user".to_owned(),
            password: "fdlkFDLKF#$3213!".to_owned(),
        };

        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn invalid_username_start_with_underscore() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "_user".to_owned(),
            password: "fdlkFDLKF#$3213!".to_owned(),
        };

        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn invalid_username_non_english() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "مستخدم".to_owned(),
            password: "fdlkFDLKF#$3213!".to_owned(),
        };

        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn invalid_username_too_short() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "us".to_owned(),
            password: "fdlkFDLKF#$3213!".to_owned(),
        };

        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn invalid_username_too_long() {
        let service = get_service().await.expect("Failed to get service");

        let username = "user".repeat(100);

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username,
            password: "fdlkFDLKF#$3213!".to_owned(),
        };

        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;
        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn invalid_password_too_short() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "user".to_owned(),
            password: "1234567".to_owned(),
        };
        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;
        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}",
        );
    }

    #[tokio::test]
    async fn invalid_password_too_long() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "user".to_owned(),
            password: "123".repeat(10),
        };
        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;
        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}",
        );
    }

    #[tokio::test]
    async fn invalid_password_no_lowercase_letter() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "user".to_owned(),
            password: "KJHD74397$#&KDH".to_owned(),
        };
        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;
        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}",
        );
    }

    #[tokio::test]
    async fn invalid_password_no_uppercase_letter() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "user".to_owned(),
            password: "kjhdf74397$#&kdh".to_owned(),
        };
        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;
        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}",
        );
    }

    #[tokio::test]
    async fn invalid_password_no_number() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "user".to_owned(),
            password: "kjhdfKJHDKH$#&kdh".to_owned(),
        };
        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;
        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}",
        );
    }

    #[tokio::test]
    async fn invalid_password_no_special_character() {
        let service = get_service().await.expect("Failed to get service");

        let signup_schema = NewUserSchema {
            first_name: "First".to_owned(),
            last_name: Some("Last".to_owned()),
            username: "user".to_owned(),
            password: "kjhdfKJHDKH1234".to_owned(),
        };
        let res = send(
            &service,
            "auth/signup",
            Method::POST,
            Some(&signup_schema),
            vec![],
        )
        .await;
        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}",
        );
    }
}

#[cfg(test)]
mod signin {
    use super::*;

    #[tokio::test]
    async fn signin_with_valid_credentials() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");

        let user = crate::db_utils::create_user(
            &conn,
            NewUserSchema {
                first_name: "First".to_owned(),
                last_name: Some("Last".to_owned()),
                username: "Username348939843".to_owned(),
                password: "fdkjhKFHDKH347(#*&".to_owned(),
            },
        )
        .await
        .expect("Failed to create user");

        let signin_schema = SigninSchema {
            username: user.username.clone(),
            password: "fdkjhKFHDKH347(#*&".to_owned(),
        };

        let mut res = send(
            &service,
            "auth/signin",
            Method::POST,
            Some(&signin_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "The response should have a `200 OK` status code {res:?}"
        );

        let user_schema: UserSigninSchema = serde_json::from_str(
            &res.take_string()
                .await
                .expect("Could not get the response body"),
        )
        .expect("Failed to parse response body");

        assert_eq!(
            user_schema.user, user,
            "The user schema should be the same as the signin schema"
        );
    }

    #[tokio::test]
    async fn signin_with_invalid_username() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");

        crate::db_utils::create_user(
            &conn,
            NewUserSchema {
                first_name: "First".to_owned(),
                last_name: Some("Last".to_owned()),
                username: "Username3489398423".to_owned(),
                password: "fdkjhKFHDKH347(#*&".to_owned(),
            },
        )
        .await
        .expect("Failed to create user");

        let signin_schema = SigninSchema {
            username: "InvalidUsername".to_owned(),
            password: "fdkjhKFHDKH347(#*&".to_owned(),
        };

        let res = send(
            &service,
            "auth/signin",
            Method::POST,
            Some(&signin_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::FORBIDDEN),
            "The response should have a `403 FORBIDDEN` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn signin_with_invalid_password() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");

        let user = crate::db_utils::create_user(
            &conn,
            NewUserSchema {
                first_name: "First".to_owned(),
                last_name: Some("Last".to_owned()),
                username: "Username3489398431".to_owned(),
                password: "fdkjhKFHDKH347(#*&".to_owned(),
            },
        )
        .await
        .expect("Failed to create user");

        let signin_schema = SigninSchema {
            username: user.username,
            password: "InvalidPassword".to_owned(),
        };

        let res = send(
            &service,
            "auth/signin",
            Method::POST,
            Some(&signin_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn signin_with_invalid_username_and_password() {
        let service = get_service().await.expect("Failed to get service");

        let signin_schema = SigninSchema {
            username: "InvalidUsername".to_owned(),
            password: "InvalidPassword".to_owned(),
        };

        let res = send(
            &service,
            "auth/signin",
            Method::POST,
            Some(&signin_schema),
            vec![],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}"
        );
    }
}

mod refresh {
    use super::*;

    #[tokio::test]
    async fn refresh_with_valid_refresh_token() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = crate::db_utils::create_user(
            &conn,
            NewUserSchema {
                first_name: "First".to_owned(),
                last_name: Some("Last".to_owned()),
                username: "Username3489239".to_owned(),
                password: "fdkjhKFHDKH347(#*&".to_owned(),
            },
        )
        .await
        .expect("Failed to create user");

        let (jwt, refresh_token) = crate::db_utils::signin_user(user.clone(), &secret_key)
            .await
            .map(|user| (user.jwt, user.refresh_token))
            .expect("Failed to signin user");

        // Sleep until the refresh token available
        tokio::time::sleep(Duration::seconds(3).to_std().unwrap()).await;

        let mut res = send(
            &service,
            "auth/refresh",
            Method::GET,
            None::<&str>,
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", refresh_token))
                    .expect("Failed to create header value"),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::OK),
            "The response should have a `200 OK` status code {res:?}"
        );

        let user_schema: UserSigninSchema = serde_json::from_str(
            &res.take_string()
                .await
                .expect("Could not get the response body"),
        )
        .expect("Failed to parse response body");

        assert_eq!(
            user_schema.user, user,
            "The user schema should be the same as the signin schema"
        );

        assert_ne!(
            user_schema.jwt, jwt,
            "The JWT token should be different from the old one"
        );

        assert_ne!(
            user_schema.refresh_token, refresh_token,
            "The refresh token should be different from the old one"
        );
    }

    #[tokio::test]
    async fn unavailable_refresh_token() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = crate::db_utils::create_user(
            &conn,
            NewUserSchema {
                first_name: "First".to_owned(),
                last_name: Some("Last".to_owned()),
                username: "Username3489238".to_owned(),
                password: "fdkjhKFHDKH347(#*&".to_owned(),
            },
        )
        .await
        .expect("Failed to create user");

        let refresh_token = crate::db_utils::signin_user(user.clone(), &secret_key)
            .await
            .map(|user| user.refresh_token)
            .expect("Failed to signin user");

        let res = send(
            &service,
            "auth/refresh",
            Method::GET,
            None::<&str>,
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", refresh_token))
                    .expect("Failed to create header value"),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::FORBIDDEN),
            "The response should have a `403 FORBIDDEN` status code {res:?}"
        )
    }

    #[tokio::test]
    async fn expired_refresh_token() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = crate::db_utils::create_user(
            &conn,
            NewUserSchema {
                first_name: "First".to_owned(),
                last_name: Some("Last".to_owned()),
                username: "Username3489237".to_owned(),
                password: "fdkjhKFHDKH347(#*&".to_owned(),
            },
        )
        .await
        .expect("Failed to create user");

        let refresh_token = crate::db_utils::signin_user(user.clone(), &secret_key)
            .await
            .map(|user| user.refresh_token)
            .expect("Failed to signin user");

        // Sleep until the refresh token expired
        tokio::time::sleep(Duration::seconds(5).to_std().unwrap()).await;

        let res = send(
            &service,
            "auth/refresh",
            Method::GET,
            None::<&str>,
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", refresh_token))
                    .expect("Failed to create header value"),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::UNAUTHORIZED),
            "The response should have a `401 UNAUTHORIZED` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn invalid_refresh_token() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            "auth/refresh",
            Method::GET,
            None::<&str>,
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", Uuid::new_v4()))
                    .expect("Failed to create header value"),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::FORBIDDEN),
            "The response should have a `403 FORBIDDEN` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn without_token() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(&service, "auth/refresh", Method::GET, None::<&str>, vec![]).await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::UNAUTHORIZED),
            "The response should have a `401 UNAUTHORIZED` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn invalid_token_type() {
        let service = get_service().await.expect("Failed to get service");

        let res = send(
            &service,
            "auth/refresh",
            Method::GET,
            None::<&str>,
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Basic {}", Uuid::new_v4()))
                    .expect("Failed to create header value"),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::UNAUTHORIZED),
            "The response should have a `401 UNAUTHORIZED` status code {res:?}"
        );
    }

    #[tokio::test]
    async fn refresh_with_jwt() {
        let service = get_service().await.expect("Failed to get service");
        let conn = get_connection().await.expect("Failed to get connection");
        let secret_key = get_secret_key();

        let user = crate::db_utils::create_user(
            &conn,
            NewUserSchema {
                first_name: "First".to_owned(),
                last_name: Some("Last".to_owned()),
                username: "Username3489236".to_owned(),
                password: "fdkjhKFHDKH347(#*&".to_owned(),
            },
        )
        .await
        .expect("Failed to create user");

        let jwt = crate::db_utils::signin_user(user.clone(), &secret_key)
            .await
            .map(|user| user.jwt)
            .expect("Failed to signin user");

        // Sleep until the refresh token available
        tokio::time::sleep(Duration::seconds(3).to_std().unwrap()).await;

        let res = send(
            &service,
            "auth/refresh",
            Method::GET,
            None::<&str>,
            vec![(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", jwt))
                    .expect("Failed to create header value"),
            )],
        )
        .await;

        assert_eq!(
            res.status_code,
            Some(StatusCode::BAD_REQUEST),
            "The response should have a `400 BAD_REQUEST` status code {res:?}"
        );
    }
}
