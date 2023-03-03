# List available recipes
default:
    @just --list --unsorted

# Run a lox script
run:
    cargo run

# Compile and lint checking
check:
    cargo check
    cargo clippy

# Build the binary
build:
    cargo build --release

# Run all tests
test:
    cargo test

# Run all tests with nextest
nextest:
    cargo nextest run

# Continuously test
ctest:
    cargo watch -x test

# Continuously run nextest
cntest:
    cargo watch -x 'nextest run'

# Continuously test with nvim integration
ltest:
    cargo watch -x 'ltest -- --nocapture'

# aliases
alias c := check
alias b := build
alias t := test
alias nt := nextest
