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

# Run the CI
@ci: && msrv
    cargo +stable build -q
    cargo +stable fmt -- --check
    cargo +stable clippy -- -D warnings

# Check that the current MSRV is correct
@msrv:
    rustup toolchain install {{msrv}}
    echo "Checking MSRV ({{msrv}})"
    cargo +{{msrv}} check -q
    echo "MSRV is correct"

# Rebuild the API docker image and run it
docker-dev:
    cargo +stable build -r -q
    docker rm -f $(docker ps -a -f NAME=xors-api-1 -q)
    docker rmi $(docker images -a | rg "xors-api\s*latest\s*(\w*)" -o --trim | xargs python3 -c "import sys;print(sys.argv[-1])")
    docker compose up

# Rebuild the API docker image and the database docker image and run them
fdocker-dev: && docker-dev
    docker rm $(docker ps -a -f NAME=xors-db-1 -q)

alias d := docker-dev
alias fd := fdocker-dev

