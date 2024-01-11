FROM rust:latest

WORKDIR /app

EXPOSE 8000

COPY Cargo.toml Cargo.lock ./

COPY src ./src
COPY openapi_doc.md openapi_doc.md
COPY entity ./entity
COPY migration ./migration

RUN cargo build --release

CMD ["cargo", "run", "--release"]

# FOR dev
# COPY target/release/xors-api ./api
# CMD ["./api"]
