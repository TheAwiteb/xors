# xors api

Xors API is a REST API for the [xors](https://github.com/TheAwiteb/xors) project.

## Features
- [X] Full documentation using Swagger UI
- [X] JWT authentication with refresh tokens
- [X] Captcha support
- [X] Username and password validation
- [ ] Password reset
- [ ] Multiplayer support with websockets

## Requirements
The API can be run only using Docker and docker-compose.


## Installation
> [!NOTE]
> Update the `JWT_SECRET` in `docker-compose.yml` file.
> You can use `openssl rand -hex 32` to generate a random secret.

```bash
git clone https://github.com/TheAwiteb/xors
cd xors
# After updating the JWT_SECRET
docker-compose up -d
```

<!-- ## Log file

> [!warning]
> The log file will be rewritten every time you restart the API.

The log file is located at `/app/logs/xors.log` inside the container, you can access it using the following command:
```bash
docker cp xors_api_1:/app/logs/xors_api.log xors_api.log
``` -->

### Database
The PostgreSQL database is in a separate container, and doesn't have any connection to the host machine.

#### Backup
To backup the database, you can use the following command:
```bash
docker-compose exec db bash -c "export PGPASSWORD=mypassword && pg_dump -U myuser xors_api_db" | gzip -9 > "xors_api_db-postgres-backup-$(date +%d-%m-%Y"_"%H_%M_%S).sql.gz"
```

#### Restore
To restore the database, you can use the following command:

> [!NOTE]
> Replace `xors_api_db-postgres-backup-17-01-2024_20_46_15.sql.gz` with the backup file name.
> And replace `xors_api_db-postgres-backup-17-01-2024_20_46_15.sql` with the backup file name without `.gz` extension.

```bash
# Stop the API
docker-compose stop api
# Restore the database
gunzip -k xors_api_db-postgres-backup-17-01-2024_20_46_15.sql.gz && \
        docker cp xors_api_db-postgres-backup-17-01-2024_20_46_15.sql xors_db_1:/pg-backup.sql && \
        docker-compose exec db bash -c "export PGPASSWORD=mypassword && dropdb -U myuser xors_api_db --force && createdb -U myuser xors_api_db && psql -U myuser xors_api_db < pg-backup.sql"
# Start the API
docker-compose start api
```

## API
After running the server, you can access the API documentation at `http://0.0.0.0:8000/api-doc/swagger-ui/`

### Development
For development, you need to have this requirements:
- [cargo (Rust)](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [just](https://crates.io/crates/just)
- [cargo-dotenv](https://crates.io/crates/cargo-dotenv)
- [docker-compose](https://docs.docker.com/engine/install/)

#### Run the database
For the database, we will run it in a docker container, you can run it using the following command:
```bash
docker-compose up -d db
```

#### Run the API
To run the API, you need to run the following command:
```bash
just run
```

#### Run the CI
To run the CI, you need to run the following command:
```bash
just ci
```

#### Close the database
To close the database, you need to run the following command:
```bash
docker-compose stop db
```

### Flowchart
You can find the flowchart of the API at [`flowchart.mermaid`](./flowchart.mermaid) file. And this is how it looks like:

```mermaid
---
title: Xors API
---

flowchart TB
    start([Start])

    %% Start the auth endpoints
    start --> /auth
    start --> /user

    %% The auth endpoints
    subgraph /auth
        auth_start -- GET --> captcha
        auth_start -- POST --> signup
        auth_start -- POST --> signin
        auth_start -- GET --> refresh

        subgraph captcha endpoint
            captcha["/captcha"]
            captcha_END([End])

            captcha --> captcha_T1(["Send captcha"]) -- 200 --> captcha_END
        end

        subgraph signup endpoint
            signup["/signup"]
            signup_END([End])

            signup --> signup_Q1{"Is signin schema\nentered correctly?"}
            signup_Q1 -- "Yes" --> signup_Q2{"Is the captcha\ncorrect?"}
            signup_Q1 -- "No" --> signup_T1(["Send 400\nerror"]) --> signup_END

            signup_Q2 -- "Yes" --> signup_Q3{"Are all user data\nvalide?"}
            signup_Q2 -- "No" --> signup_T2(["Send 403\nerror"]) --> signup_END

            signup_Q3 -- "Yes" --> signup_T3(["Create the\nuser and send\nit's data\nwith JWT"]) -- 200 --> signup_END
            signup_Q3 -- "No" --> signup_T4(["Send 400\nerror"]) --> signup_END
        end

        subgraph signin endpoint
            signin["/signin"]
            signin_END([End])

            signin --> signin_Q1{"Is signin schema\nentered correctly?"}
            signin_Q1 -- "Yes" --> signin_Q2{"Do the username and\npassword exist?"}
            signin_Q1 -- "No" --> signin_T1(["Send 400\nerror"]) --> signin_END

            signin_Q2 -- "Yes" --> signin_T2(["Send the\nuser data\nwith JWT"]) -- 200 --> signin_END
            signin_Q2 -- "No" --> signin_T3(["Send 403\nerror"]) --> signin_END
        end

        subgraph refresh endpoint
            refresh["/refresh"]
            refresh_END([End])

            refresh --> refresh_Q1{"Is there Authorization header?"}
            refresh_Q1 -- "Yes" --> refresh_Q2{"Is valid\nBearer JWT token?"}
            refresh_Q1 -- "No" --> refresh_T1(["Send 403\nerror"]) --> refresh_END

            refresh_Q2 -- "Yes" --> refresh_Q3{"Is it a\nrefresh token?"}
            refresh_Q2 -- "No" --> refresh_T2(["Send 403\nerror"]) --> refresh_END

            refresh_Q3 -- "Yes" --> refresh_Q4{"Is in the active time?\nand it's not expired?"}
            refresh_Q3 -- "No" --> refresh_T3(["Send 400\nerror"]) --> refresh_END

            refresh_Q4 -- "Yes" --> refresh_T4(["Send the\nuser data\nwith JWT"]) -- 200 --> refresh_END
            refresh_Q4 -- "No" --> refresh_T5(["Send 403\nerror"]) --> refresh_END
        end
    end

    %% The user endpoints
    subgraph /user
        user_start -- GET --> me
        user_start -- GET --> get_user
        user_start -- DELETE --> delete_user
        user_start -- PUT --> put_user

        subgraph me_endpint
            me["/me"]
            me_END([End])

            me --> me_Q1{"Is there Authorization header?"}
            me_Q1 -- "Yes" --> me_Q2{"Is valid\nBearer JWT token?"}
            me_Q1 -- "No" --> me_T1(["Send 403\nerror"]) --> me_END

            me_Q2 -- "Yes"  --> me_T4(["Send the\nuser data"]) -- 200 --> me_END
            me_Q2 -- "No" --> me_T2(["Send 403\nerror"]) --> me_END
        end

        subgraph get_user
            get_user_END([End])

            get_user_Q1{"Is there Authorization header?"}
            get_user_Q1 -- "Yes" --> get_user_Q2{"Is valid\nBearer JWT token?"}
            get_user_Q1 -- "No" --> get_user_T1(["Send 403\nerror"]) --> get_user_END

            get_user_Q2 -- "Yes" --> get_user_Q3{"Is there is `uuid`\nin the query?"}
            get_user_Q2 -- "No" --> get_user_T2(["Send 403\nerror"]) --> get_user_END

            get_user_Q3 -- "Yes" --> get_user_Q4{"Is the user `uuid`\nexist?"}
            get_user_Q3 -- "No" --> get_user_T3(["Send 400\nerror"]) --> get_user_END

            get_user_Q4 -- "Yes" --> get_user_T4(["Send the\nuser data"]) -- 200 --> get_user_END
            get_user_Q4 -- "No" --> get_user_T5(["Send 400\nerror"]) --> get_user_END
        end

        subgraph delete_user
            delete_user_END([End])

            delete_user_Q1{"Is there Authorization header?"}
            delete_user_Q1 -- "Yes" --> delete_user_Q2{"Is valid\nBearer JWT token?"}
            delete_user_Q1 -- "No" --> delete_user_T1(["Send 403\nerror"]) --> delete_user_END

            delete_user_Q2 -- "Yes" --> delete_user_Q3{"Is delete schema\nentered correctly?\nusername,password"}
            delete_user_Q2 -- "No" --> delete_user_T2(["Send 403\nerror"]) --> delete_user_END

            delete_user_Q3 -- "Yes" --> delete_user_Q4{"Do the username and\npassword valid?"}
            delete_user_Q3 -- "No" --> delete_user_T3(["Send 400\nerror"]) --> delete_user_END

            delete_user_Q4 -- "Yes" --> delete_user_T4(["Delete the\nuser"]) -- 200 --> delete_user_END
            delete_user_Q4 -- "No" --> delete_user_T5(["Send 403\nerror"]) --> delete_user_END
        end

        subgraph put_user
            put_user_END([End])

            put_user_Q1{"Is there Authorization header?"}
            put_user_Q1 -- "Yes" --> put_user_Q2{"Is valid\nBearer JWT token?"}
            put_user_Q1 -- "No" --> put_user_T1(["Send 403\nerror"]) --> put_user_END

            put_user_Q2 -- "Yes" --> put_user_Q3{"Is delete schema\nentered correctly?\first_name,last_name"}
            put_user_Q2 -- "No" --> put_user_T2(["Send 403\nerror"]) --> put_user_END

            put_user_Q3 -- "Yes" --> put_user_T3(["Delete the\nuser"]) -- 200 --> put_user_END
            put_user_Q3 -- "No" --> put_user_T4(["Send 400\nerror"]) --> put_user_END
        end
    end
```

## License
This project is licensed under the AGPL-3.0 License - see the [LICENSE](LICENSE) file for details
