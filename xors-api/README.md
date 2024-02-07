# xors api

Xors API is a REST API for the [xors](https://github.com/TheAwiteb/xors) project.

<details>
<summary>Table of Contents</summary>

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Database](#database)
    - [Backup](#backup)
    - [Restore](#restore)
- [API](#api)
    - [OpenAPI](#openapi)
    - [Development](#development)
        - [Run the API](#run-the-api)
        - [Run the CI](#run-the-ci)
- [Multiplayer WebSocket API](#multiplayer-websocket-api)
    - [Chat protocol](#chat-protocol)
    - [Client Events](#client-events)
        - [`search` event](#search-event)
        - [`play` event](#play-event)
        - [`wellcome` event](#wellcome-event)
        - [`chat` event](#chat-event)
    - [Server Events](#server-events)
        - [`game_found` event](#game_found-event)
        - [`wellcome` event](#wellcome-event-1)
        - [`chat` event](#chat-event-1)
        - [`your_turn` event](#your_turn-event)
        - [`round_start` event](#round_start-event)
        - [`round_end` event](#round_end-event)
        - [`play` event](#play-event)
        - [`auto_play` event](#auto_play-event)
        - [`game_over` event](#game_over-event)
        - [`error` event](#error-event)
        - [Game Over Reasons](#game-over-reasons)
        - [Errors](#errors)
- [License](#license)

</details>

## Features
- [X] Full documentation using Swagger UI
- [X] JWT authentication with refresh tokens
- [X] Captcha support
- [X] Rate limiting
- [ ] File logging (currently using stdout)
- [X] Username and password validation
- [X] Ability to change profile image
- [X] Password reset
- [X] Game history
- [X] Multiplayer support with websockets
- [X] Game replay
- [X] Auto play when the other take too long to play
- [X] In-game chat
- [ ] Private games
- [ ] Friends system (add, remove, block, unblock, invite to game (if online))
- [ ] [Elo rating system](https://en.wikipedia.org/wiki/Elo_rating_system) (with leaderboard)


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



## Database
The PostgreSQL database is in a separate container, and doesn't have any connection to the host machine.

### Backup
To backup the database, you can use the following command:
```bash
docker-compose exec db bash -c "export PGPASSWORD=mypassword && pg_dump -U myuser xors_api_db" | gzip -9 > "xors_api_db-postgres-backup-$(date +%d-%m-%Y"_"%H_%M_%S).sql.gz"
```

### Restore
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

### OpenAPI
You can find the OpenAPI file at [`openapi.json`](./openapi.json) file. And you can view it using [Swagger Editor](https://editor.swagger.io/?url=https://raw.githubusercontent.com/TheAwiteb/xors/master/xors-api/openapi.json).

### Development
For development, you need to have this requirements:
- [cargo (Rust)](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [just](https://crates.io/crates/just)
- [cargo-dotenv](https://crates.io/crates/cargo-dotenv)
- [docker-compose](https://docs.docker.com/engine/install/)

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

## Multiplayer WebSocket API
Our WebSocket API is easy to use, and it's based on [JSON](https://www.json.org/json-en.html) messages. The WebSocket API is located at `ws://<HOST>:<POST>/xo/` and it's only available for authenticated users, meaning that you need to send the `Authorization` header with the `Bearer <TOKEN>` value in the [WebSocket handshake request](https://en.wikipedia.org/wiki/WebSocket#Protocol_handshake).

### Chat protocol
The chat protocol is based on [PGP](https://en.wikipedia.org/wiki/Pretty_Good_Privacy) encryption and signing. The client should send the [`wellcome` event](#wellcome-event) to the server to send the PGP public key to the other player, and after that, the client can send the [`chat` event](#chat-event) to the server to send a message to the other player (Sould reiceve the [`wellcome` event](#wellcome-event-1) from the other player before sending the [`chat` event](#chat-event)). The server will not check the signature, it's only check that the message is a valid PGP message and siginature is a valid PGP signature. Also the server doesn't save anythig about the chat messages, public keys, or signatures. Also the server doesn't save any metadata about the chat messages, it's only relay the messages between the players.


### Client Events
The client can send the following events to the server:

#### `search` event
The client can send the `search` event to the server to search for a new game. The event has the following structure:
```json
{
    "event":"search",
}
```
#### `play` event
The client can send the `play` event to the server to play a move. The event has the following structure:
```json
{
    "event":"play",
    "data":{"place":5}
}
```
- `place` is the place number, and it's a number between 0 and 8, and it's mapped to the following board:

```
0 | 1 | 2
---------
3 | 4 | 5
---------
6 | 7 | 8
```

#### `wellcome` event
The client can send the `wellcome` event to the server to send the PGP public key to the other player. The event should be sent after the [game found event](#game_found-event).
The client can choise to not send the `wellcome` event, but in this case, the other player will not be able to send the `chat` event to the client, so you can make it optional to the client to start chatting or not. The event has the following structure:

```json
{
    "event":"wellcome",
    "data":{"public_key":"<PGP-PUBLIC-KEY>"}
}
```
- `public_key` is the PGP public key.

#### `chat` event
The client can send the `chat` event to the server to send a message to the other player. The event has the following structure:
```json
{
    "event":"chat",
    "data":{"encrypted_message":"<PGP-ENCRYPTED-MESSAGE>", "signature":"<PGP-SIGNATURE>"}
}
```
- `message` is the PGP encrypted message by the other player public key of course after receiving it from [wellcome event](#wellcome-event).
- `signature` is the PGP signature of the hash sha2-256 of the **encrypted message**. You should make sure that the signature is valid using the other player public key the server will not check the signature and it's only check that the signature is a valid PGP signature.


### Server Events
The server can send the following events to the client:

#### `game_found` event
The server sends the `game_found` event to the client when finding a game for the client after sending the [`search` event](#search-event). The event has the following structure:
```json
{
    "event":"game_found",
    "data":{"x_player":"<PLAYER_UUID>","o_player":"<PLAYER_UUID>"}
}
```
- `x_player` is the UUID of the player who will play with the `X` symbol.
- `o_player` is the UUID of the player who will play with the `O` symbol.

#### `wellcome` event
Resend of the [`wellcome` event](#wellcome-event) from the other player. 

#### `chat` event
A chat message from the other player, with valid PGP message and signature. The event has the following structure:
```json
{
    "event":"chat",
    "data":{"encrypted_message":"<PGP-ENCRYPTED-MESSAGE>", "signature":"<PGP-SIGNATURE>"}
}

#### `your_turn` event
The `your_turn` event is sent to the client when it's their turn to play. The event has the following structure:
```json
{
    "event":"your_turn",
    "data":{"auto_play_after":<TIMESTAMP>}
}
```
- `auto_play_after` is the timestamp of when the server will play automatically if the client didn't play before that time.

#### `round_start` event
The `round_start` event is sent to the client when a new round starts, the round starts when the game found and when the before round ends (if the game is not over). The event has the following structure:
```json
{
    "event":"round_start",
    "data":{"round":2}
}
```
- `round` is the round number.

#### `round_end` event
The `round_end` event is sent to the client when a round ends, when the round end and the game is not over. If the game is over, the `game_over` event will be sent instead. The event has the following structure:
```json
{
    "event":"round_end",
    "data":{"round":1,"winner":"<PLAYER_UUID>"}
}
```
- `round` is the round number.
- `winner` is the UUID of the winner, if the round is a draw, the value will be `null`.

#### `play` event
The `play` event is sent to the client when the other player plays. The event has the following structure:
```json
{
    "event":"play",
    "data":{"place":4,"player":"<PLAYER_UUID>"}
}
```
- `place` is the place number, and it's a number between 0 and 8, and it's mapped to the following board:

```
0 | 1 | 2
---------
3 | 4 | 5
---------
6 | 7 | 8
```
- `player` is the UUID of the player who played.

#### `auto_play` event
The `auto_play` event is sent to the client when the server plays for the client because the client didn't play before the `auto_play_after` time in the [`your_turn` event](#your_turn-event). The event has the following structure:
```json
{
    "event":"auto_play",
    "data":{"place":4}
}
```
- `place` is the place number, and it's a number between 0 and 8, and it's mapped to the following board:

```
0 | 1 | 2
---------
3 | 4 | 5
---------
6 | 7 | 8
```

#### `game_over` event
The `game_over` event is sent to the client when the game is over. The event has the following structure:
```json
{
    "uuid":"<GAME_UUID>",
    "event":"game_over",
    "data":{"winner":"<PLAYER_UUID>","reason":"<REASON>"}
}
```
- `uuid` is the UUID of the game.
- `winner` is the UUID of the winner, if the game is a draw, the value will be `null`.
- `reason` is the reason of the game over, checkout the [Game Over Reasons](#game-over-reasons) section for more information.

#### `error` event
The `error` event is sent to the client when an error occurs. The event has the following structure:
```json
{
    "event":"error",
    "data":"<MESSAGE>"
}
```
- `message` is the error message, checkout the [Errors](#errors) section for more information.

### Game Over Reasons
The game over reason is sent to the client when the game is over, and it's sent in the [`game_over` event](#game_over-event). The following are the game over reasons:
| Reason | Description |
| --- | --- |
| `player_won` | The player won the game. |
| `draw` | The game is a draw. |
| `player_disconnected` | The other player disconnected. |

### Errors
The error message is sent to the client when an error occurs, and it's sent in the [`error` event](#error-event). The following are the error messages:
| Message | Description | Reason |
| --- | --- | --- |
| `invalid_body` | The event body is invalid | When the event body is not a valid JSON |
| `unknown_event` | The event is unknown | When the event is not client event |
| `invalid_event_data_for_event` | The event data is invalid for the event | When the event data is invalid for the event |
| `already_in_search` | The player is already in search | When the player tries to search for a game while they are already in search |
| `already_wellcomed` | The player is already wellcomed | When the player tries to send the `wellcome` event after sending it before |
| `chat_not_allowed` | Chat is not allowed | When the player tries to send a chat message before sending the `wellcome` event |
| `chat_not_started` | Chat is not started | When the player tries to send a chat message before receiving the `wellcome` event |
| `invalid_public_key` | The public key is invalid | When the player tries to send the `wellcome` event with an invalid PGP public key |
| `invalid_chat_message` | The chat message is invalid | When the player tries to send a chat message with an invalid PGP message |
| `invalid_chat_signature` | The chat signature is invalid | When the player tries to send a chat message with an invalid PGP signature |
| `already_in_game` | The player is already in a game | When the player tries to search for a game while they are already in a game |
| `not_in_game` | The player is not in a game | When the player tries to play a move while they are not in a game |
| `not_your_turn` | It's not the player turn | When the player tries to play a move while it's not their turn |
| `invalid_place` | The place is invalid | When the player tries to play a move with an invalid place, like already played place or place out of the board |
| `max_games_reached` | The server reached the maximum games limit | When the server reached the maximum games limit |
| `other` | Other errors | Usually when an unexpected error occurs, like a database error, if you get this error, try to resend the last event and report the error to the server owner |


## License
This project is licensed under the AGPL-3.0 License - see the [LICENSE](LICENSE) file for details
