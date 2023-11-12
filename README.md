# Neo4rs [![CI Status][ci-badge]][ci-url]  [![Crates.io][crates-badge]][crates-url]

[ci-badge]: https://github.com/neo4j-labs/neo4rs/actions/workflows/checks.yml/badge.svg
[ci-url]: https://github.com/neo4j-labs/neo4rs
[crates-badge]: https://img.shields.io/crates/v/neo4rs.svg?style=shield
[crates-url]: https://crates.io/crates/neo4rs
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg?style=shield
[docs-url]: https://docs.rs/neo4rs

`neo4rs` is a driver for the [Neo4j](https://neo4j.com/) graph database, written in Rust.

`neo4rs` implements the [bolt specification](https://7687.org/bolt/bolt-protocol-message-specification-4.html#version-41)

This driver is compatible with Neo4j version 5.x and 4.4.
Only the latest 5.x version is supported, following the [Neo4j Version support policy](https://neo4j.com/developer/kb/neo4j-supported-versions/).

## API Documentation: [![Docs.rs][docs-badge]][docs-url]

## Example

```rust
    // concurrent queries
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Graph::new(&uri, user, pass).await.unwrap();
    for _ in 1..=42 {
        let graph = graph.clone();
        tokio::spawn(async move {
            let mut result = graph.execute(
           query("MATCH (p:Person {name: $name}) RETURN p").param("name", "Mark")
        ).await.unwrap();
            while let Ok(Some(row)) = result.next().await {
            let node: Node = row.get("p").unwrap();
            let name: String = node.get("name").unwrap();
                println!("{}", name);
            }
        });
    }

    //Transactions
    let mut txn = graph.start_txn().await.unwrap();
    txn.run_queries([
        "CREATE (p:Person {name: 'mark'})",
        "CREATE (p:Person {name: 'jake'})",
        "CREATE (p:Person {name: 'luke'})",
    ])
    .await
    .unwrap();
    txn.commit().await.unwrap(); //or txn.rollback().await.unwrap();
```

## MSRV

The crate has a minimum supported Rust version (MSRV) of `1.63.0`.

A change in the MSRV in *not* considered a breaking change.
For versions past 1.0.0, a change in the MSRV can be done in a minor version increment (1.1.3 -> 1.2.0)
for versions before 1.0.0, a change in the MSRV can be done in a patch version increment (0.1.3 -> 0.1.4).


## Development

### Testing

This crate contains unit tests and integration tests.
The unit tests are run with `cargo test --lib` and do not require a running Neo4j instance.
The integration tests are run with `cargo test` and require a running Neo4j instance.

#### Running the integration tests

To run the tests, you need to have either docker or an existing Neo4j instance running.
Docker is recommended since the tests don't necessarily clean up after themselves.

##### Using Docker

To run the tests with docker, you need to have docker installed and running.
You can control the version of Neo4j that is used by setting the `NEO4J_VERSION_TAG` environment variable.
The default version is `4.2`.
The tests will use the official `neo4j` docker image, with the provided version as tag.

You might run into panics or test failures with the message 'failed to start container'.
In that case, try to pull the image first before running the tests with `docker pull neo4j:$NEO4J_VERSION_TAG`.

This could happen if you are on a machine with an architecture that is not supported by the image, e.g. `arm64` like the Apple silicon Macs.
In that case, pulling the image will fail with a message like 'no matching manifest for linux/arm64/v8'.
You need to use the `--platform` flag to pull the image for a different architecture, e.g. `docker pull --platform linux/amd64 neo4j:$NEO4J_VERSION_TAG`.
There is an experimental option in docker to use Rosetta to run those images, so that tests don't take forever to run (please check the docker documentation).

You could also use a newer neo4j version like `4.4` instead, which has support for `arm64` architecture.

##### Using an existing Neo4j instance

To run the tests with an existing Neo4j instance, you need to have the `NEO4J_TEST_URI` environment variable set to the connection string, e.g. `neo4j+s://42421337thisisnotarealinstance.databases.neo4j.io`.
The default user is `neo4j`, but it can be changed with the `NEO4J_TEST_USER` environment variable.
The default password is `neo`, but it can be changed with the `NEO4J_TEST_PASS` environment variable.

Some tests might run different queries depending on the Neo4j version.
You can use the `NEO4J_VERSION_TAG` environment variable to set the version of the Neo4j instance.

It is recommended to only run a single integration test and manually clean up the database after the test.

```sh
env NEO4J_TEST_URI=neo4j+s://42421337thisisnotarealinstance.databases.neo4j.io NEO4J_TEST_USER=neo4j NEO4J_TEST_PASS=supersecret NEO4J_VERSION_TAG=5.8 cargo test --test <name of the integration test, see the file names in lib/tests/>
```

### Updating `Cargo.lock` files for CI

We have CI tests that verify the MSRV as well as the minimal version of the dependencies.
The minimal versions are the lowest version that still satisfies the `Cargo.toml` entries, instead of the default of the highest version.

If you change anything in the `Cargo.toml`, you need to update the `Cargo.lock` files for CI.

This project uses [xtask](https://github.com/matklad/cargo-xtask#cargo-xtask)s to help with updating the lock files.

It is recommended to close all editors, or more specifically, all rust-analyzer instances for this project before running the commands below.

#### Update `ci/Cargo.lock.msrv`

```bash
# If there are errors, update Cargo.toml to fix and try again from the top.
# You might have to downgrade or remove certain crates to hit the MSRV.
# A number of such downgrades are already defined in the `update_msrv_lock` function
# in the xtask script.
# Alternatively, you might suggest an increase of the MSRV.
cargo xtask msrv
```

Using `xtask` requires that `curl` and `jq` are available on the system.


#### Update `ci/Cargo.lock.min`

```bash
# If there are errors, update Cargo.toml to fix and try again from the top.
cargo xtask min
```

Using `xtask` requires that `curl` and `jq` are available on the system.


## License

Neo4rs is licensed under either of the following, at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
