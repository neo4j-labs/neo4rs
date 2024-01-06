#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

cargo release version --execute --no-confirm --verbose --package neo4rs "$1"
cargo release replace --execute --no-confirm --verbose --package neo4rs
cargo release hook    --execute --no-confirm --verbose --package neo4rs
cargo release commit  --execute --no-confirm --verbose

