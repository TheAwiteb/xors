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


_default:
    @{{JUST_EXECUTABLE}} --list-heading "{{header}}" --list

# Run the API
run:
    RUST_LOG=debug cargo dotenv -e .env.dev -- run

# Run the CI (Local use only)
@ci:
    cargo +stable dotenv -e .env.dev -- build -q
    cargo +stable dotenv -e .env.dev -- fmt -- --check
    cargo +stable dotenv -e .env.dev -- clippy -- -D warnings --no-deps
    {{JUST_EXECUTABLE}} msrv
    {{JUST_EXECUTABLE}} tests

# Check that the current MSRV is correct
msrv:
    @rustup toolchain install {{msrv}}
    echo "Checking MSRV ({{msrv}})"
    cargo +{{msrv}} dotenv -e .env.dev -- build -q
    echo "MSRV is correct"

# Run the tests
@tests:
    docker-compose rm -f -s db
    docker-compose up -d db
    sleep 2
    cargo dotenv -e .env.dev -- run --manifest-path ./migration/Cargo.toml -- up -u postgres://myuser:mypassword@localhost:8246/xors_api_db

    cargo dotenv -e .env.dev -- test 

    docker-compose rm -f -s db
