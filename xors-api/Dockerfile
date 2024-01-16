FROM rust:latest

WORKDIR /app

EXPOSE 8000

COPY Cargo.toml Cargo.lock ./

COPY src ./src
COPY api_des.md api_des.md
COPY entity ./entity
COPY migration ./migration

RUN cargo build --release

CMD ["cargo", "run", "--release"]

# FOR dev
# COPY target/release/xors-api ./api
# CMD ["./api"]
