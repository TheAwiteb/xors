# This justfile is for the contrbutors of this project, not for the end user.
#
# Requirements for this justfile:
# - Linux distribution, the real programer does not program on garbage OS like Windows or MacOS
# - just (Of course) <https://github.com/casey/just>
# - cargo (For the build and tests) <https://doc.rust-lang.org/cargo/getting-started/installation.html>

set shell := ["/usr/bin/bash", "-c"]

JUST_EXECUTABLE := "just -u -f " + justfile()
header := "Available tasks:\n"
# Get the MSRV from the Cargo.toml
msrv := `cat Cargo.toml | grep "rust-version" | sed 's/.*"\(.*\)".*/\1/'`
# Docker exec
de := "docker-compose exec -T api"


_default:
    @{{JUST_EXECUTABLE}} --list-heading "{{header}}" --list

# Run the CI (Local use only)
ci:
    cargo +stable build -q
    cargo +stable fmt -- --all --check
    cargo +stable clippy -- -D warnings --no-deps
    {{JUST_EXECUTABLE}} msrv
    {{JUST_EXECUTABLE}} docker-tests

# Check that the current MSRV is correct
msrv:
    @rustup toolchain install {{msrv}}
    echo "Checking MSRV ({{msrv}})"
    cargo +{{msrv}} build -q
    echo "MSRV is correct"

# Rebuild the API docker image and run it
docker-dev:
    cargo +stable build -r -q
    docker-compose rm -f -s api
    docker compose up

# Rebuild the API docker image and the database docker image and run them
fdocker-dev: && docker-dev
    docker rm -f $(docker ps -a -f NAME=xors-db-1 -q)

# Run the tests in a docker container
docker-tests:
    docker compose up -d

    {{de}} cargo test 

    @{{JUST_EXECUTABLE}} docker-clean

# Clean docker images (Panics if the images are not present)
docker-clean:
    @docker-compose rm -f -s api db

alias d := docker-dev
alias fd := fdocker-dev

