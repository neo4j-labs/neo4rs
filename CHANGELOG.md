<!-- markdownlint-disable MD024 -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](http://semver.org).

## [0.9.0-rc.3](https://github.com/neo4j-labs/neo4rs/tree/0.9.0-rc.3) - 2025-01-15

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.9.0-rc.2...0.9.0-rc.3)

### Other

- Add query! macro providing a more ergonomic way to create parmeterized queries [#214](https://github.com/neo4j-labs/neo4rs/pull/214) ([knutwalker](https://github.com/knutwalker))
- Change query entrypoints to use `Into<Query>` instead of just `Query` [#213](https://github.com/neo4j-labs/neo4rs/pull/213) ([knutwalker](https://github.com/knutwalker))
- release: neo4rs v0.9.0-rc.2 [#212](https://github.com/neo4j-labs/neo4rs/pull/212) ([github-actions](https://github.com/github-actions))

## [v0.9.0-rc.2](https://github.com/neo4j-labs/neo4rs/tree/v0.9.0-rc.2) - 2025-01-13

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.8.0...v0.9.0-rc.2)

### Other

- release: neo4rs v0.9.0-rc.2 [#212](https://github.com/neo4j-labs/neo4rs/pull/212) ([github-actions](https://github.com/github-actions))
- Fix extracting point properties from BoltType [#211](https://github.com/neo4j-labs/neo4rs/pull/211) ([knutwalker](https://github.com/knutwalker))
- Fix property parsing when a property contains a struct [#210](https://github.com/neo4j-labs/neo4rs/pull/210) ([knutwalker](https://github.com/knutwalker))
- Bump some dependencies [#208](https://github.com/neo4j-labs/neo4rs/pull/208) ([knutwalker](https://github.com/knutwalker))
- Guard against incomplete list/maps [#207](https://github.com/neo4j-labs/neo4rs/pull/207) ([knutwalker](https://github.com/knutwalker))
- Add missing property name to the "property missing" error message [#206](https://github.com/neo4j-labs/neo4rs/pull/206) ([knutwalker](https://github.com/knutwalker))
- Client side routing implementation [#205](https://github.com/neo4j-labs/neo4rs/pull/205) ([madchicken](https://github.com/madchicken))
- Add Ignore to current Bolt implementation [#204](https://github.com/neo4j-labs/neo4rs/pull/204) ([madchicken](https://github.com/madchicken))
- Add skip_ssl_validation flag and refactor TLS config [#201](https://github.com/neo4j-labs/neo4rs/pull/201) ([madchicken](https://github.com/madchicken))
- Make result summary available behind a feature flag [#199](https://github.com/neo4j-labs/neo4rs/pull/199) ([knutwalker](https://github.com/knutwalker))
- release: neo4rs v0.9.0-rc.1 [#198](https://github.com/neo4j-labs/neo4rs/pull/198) ([github-actions](https://github.com/github-actions))
- Use server default db over hardcoded default db [#197](https://github.com/neo4j-labs/neo4rs/pull/197) ([knutwalker](https://github.com/knutwalker))
- Use new Pull message behind the feature flag [#196](https://github.com/neo4j-labs/neo4rs/pull/196) ([knutwalker](https://github.com/knutwalker))
- Update README.md [#195](https://github.com/neo4j-labs/neo4rs/pull/195) ([knutwalker](https://github.com/knutwalker))
- Make it a bit simpler to test against an Aura instance [#193](https://github.com/neo4j-labs/neo4rs/pull/193) ([knutwalker](https://github.com/knutwalker))
- Refactor connection creation [#192](https://github.com/neo4j-labs/neo4rs/pull/192) ([knutwalker](https://github.com/knutwalker))
- Return server errors as Neo4jError, not unexpected [#191](https://github.com/neo4j-labs/neo4rs/pull/191) ([knutwalker](https://github.com/knutwalker))
- Retry on certain query failures for managed transactions [#190](https://github.com/neo4j-labs/neo4rs/pull/190) ([knutwalker](https://github.com/knutwalker))
- Implement a few messages for the new bolt protocol implementation [#181](https://github.com/neo4j-labs/neo4rs/pull/181) ([knutwalker](https://github.com/knutwalker))

## [v0.8.0](https://github.com/neo4j-labs/neo4rs/tree/v0.8.0) - 2024-08-07

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.7.3...v0.8.0)

### Other

- release: neo4rs v0.8.0 [#189](https://github.com/neo4j-labs/neo4rs/pull/189) ([github-actions](https://github.com/github-actions))
- Return server errors as Neo4jError, not unexpected [#187](https://github.com/neo4j-labs/neo4rs/pull/187) ([knutwalker](https://github.com/knutwalker))
- Retry on certain query failures for managed transactions [#186](https://github.com/neo4j-labs/neo4rs/pull/186) ([knutwalker](https://github.com/knutwalker))

## [v0.7.3](https://github.com/neo4j-labs/neo4rs/tree/v0.7.3) - 2024-07-31

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.7.2...v0.7.3)

### Other

- release: neo4rs v0.7.3 [#185](https://github.com/neo4j-labs/neo4rs/pull/185) ([github-actions](https://github.com/github-actions))
- Enable servers to use SSR [#183](https://github.com/neo4j-labs/neo4rs/pull/183) ([knutwalker](https://github.com/knutwalker))
- Use system installed certificates by default and accept neo4j+ssc connections [#180](https://github.com/neo4j-labs/neo4rs/pull/180) ([madchicken](https://github.com/madchicken))
- release: neo4rs v0.7.2 [#179](https://github.com/neo4j-labs/neo4rs/pull/179) ([github-actions](https://github.com/github-actions))

## [v0.7.2](https://github.com/neo4j-labs/neo4rs/tree/v0.7.2) - 2024-07-22

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.7.1...v0.7.2)

### Other

- release: neo4rs v0.7.2 [#179](https://github.com/neo4j-labs/neo4rs/pull/179) ([github-actions](https://github.com/github-actions))
- Remove an unsafe block [#178](https://github.com/neo4j-labs/neo4rs/pull/178) ([ChayimFriedman2](https://github.com/ChayimFriedman2))
- Bump MSRV to 1.75.0 [#177](https://github.com/neo4j-labs/neo4rs/pull/177) ([knutwalker](https://github.com/knutwalker))
- Implement new version of the Pull command [#176](https://github.com/neo4j-labs/neo4rs/pull/176) ([knutwalker](https://github.com/knutwalker))
- Refactor current stream API in preparation for summary stats [#175](https://github.com/neo4j-labs/neo4rs/pull/175) ([knutwalker](https://github.com/knutwalker))
- Use GHA service container for integration tests [#174](https://github.com/neo4j-labs/neo4rs/pull/174) ([knutwalker](https://github.com/knutwalker))
- Update CI jobs to include feature flags [#173](https://github.com/neo4j-labs/neo4rs/pull/173) ([knutwalker](https://github.com/knutwalker))
- Add feature flags for new bolt protocol implementation [#172](https://github.com/neo4j-labs/neo4rs/pull/172) ([knutwalker](https://github.com/knutwalker))
- Add more re-implementations of the bolt protocol [#171](https://github.com/neo4j-labs/neo4rs/pull/171) ([knutwalker](https://github.com/knutwalker))
- Implement TryInto<serde_json::Value>, with optional `json` feature [#166](https://github.com/neo4j-labs/neo4rs/pull/166) ([elimirks](https://github.com/elimirks))
- Simplify extractors deserializer [#165](https://github.com/neo4j-labs/neo4rs/pull/165) ([knutwalker](https://github.com/knutwalker))
- Add some general types and untilities for more serde processing [#163](https://github.com/neo4j-labs/neo4rs/pull/163) ([knutwalker](https://github.com/knutwalker))
- Add serde serializer and deserializer for packstream [#162](https://github.com/neo4j-labs/neo4rs/pull/162) ([knutwalker](https://github.com/knutwalker))
- Release 0.7.1 [#160](https://github.com/neo4j-labs/neo4rs/pull/160) ([knutwalker](https://github.com/knutwalker))

## [v0.7.1](https://github.com/neo4j-labs/neo4rs/tree/v0.7.1) - 2023-12-28

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.7.0...v0.7.1)

### Other

- Deserialize missing properties as none [#159](https://github.com/neo4j-labs/neo4rs/pull/159) ([knutwalker](https://github.com/knutwalker))
- release: neo4rs v0.8.0-dev.1 [#158](https://github.com/neo4j-labs/neo4rs/pull/158) ([github-actions](https://github.com/github-actions))
- release: neo4rs v0.7.0 [#157](https://github.com/neo4j-labs/neo4rs/pull/157) ([github-actions](https://github.com/github-actions))

## [v0.7.0](https://github.com/neo4j-labs/neo4rs/tree/v0.7.0) - 2023-12-11

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.7.0-rc.3...v0.7.0)

### Other

- release: neo4rs v0.7.0 [#157](https://github.com/neo4j-labs/neo4rs/pull/157) ([github-actions](https://github.com/github-actions))
- Switch to the testcontainers_modules testcontainer [#155](https://github.com/neo4j-labs/neo4rs/pull/155) ([knutwalker](https://github.com/knutwalker))
- release: neo4rs v0.7.0-rc.3 [#154](https://github.com/neo4j-labs/neo4rs/pull/154) ([github-actions](https://github.com/github-actions))

## [v0.7.0-rc.3](https://github.com/neo4j-labs/neo4rs/tree/v0.7.0-rc.3) - 2023-11-25

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.7.0-rc.2...v0.7.0-rc.3)

### Other

- Assert impl Send+Sync for Graph and Txn [#153](https://github.com/neo4j-labs/neo4rs/pull/153) ([knutwalker](https://github.com/knutwalker))
- Remove string allocation from Display impl for BoltType [#152](https://github.com/neo4j-labs/neo4rs/pull/152) ([knutwalker](https://github.com/knutwalker))
- fix #150. read_chunk had a buffer read bug before [#151](https://github.com/neo4j-labs/neo4rs/pull/151) ([elimirks](https://github.com/elimirks))
- release: neo4rs v0.7.0-rc.2 [#146](https://github.com/neo4j-labs/neo4rs/pull/146) ([github-actions](https://github.com/github-actions))

## [v0.7.0-rc.2](https://github.com/neo4j-labs/neo4rs/tree/v0.7.0-rc.2) - 2023-11-14

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.7.0-rc.1...v0.7.0-rc.2)

### Other

- Expose all the bolt types [#145](https://github.com/neo4j-labs/neo4rs/pull/145) ([knutwalker](https://github.com/knutwalker))
- Support deserialization into serde_json::Value and similar deserialize_any kinda types [#142](https://github.com/neo4j-labs/neo4rs/pull/142) ([knutwalker](https://github.com/knutwalker))
- Support deserialization into newtype structs [#141](https://github.com/neo4j-labs/neo4rs/pull/141) ([knutwalker](https://github.com/knutwalker))
- Support deserialization of C-style enums [#140](https://github.com/neo4j-labs/neo4rs/pull/140) ([knutwalker](https://github.com/knutwalker))
- Support deserializing SocketAddr and maybe some other enums [#138](https://github.com/neo4j-labs/neo4rs/pull/138) ([knutwalker](https://github.com/knutwalker))
- Only store cheap/rc cloneable fields inside of Graph [#136](https://github.com/neo4j-labs/neo4rs/pull/136) ([knutwalker](https://github.com/knutwalker))
- Be more flexible in what is accepted for run_queries [#135](https://github.com/neo4j-labs/neo4rs/pull/135) ([knutwalker](https://github.com/knutwalker))
- breaking! Remove internal mutex boxing of connections [#134](https://github.com/neo4j-labs/neo4rs/pull/134) ([knutwalker](https://github.com/knutwalker))
- Improve API for RowStream [#133](https://github.com/neo4j-labs/neo4rs/pull/133) ([knutwalker](https://github.com/knutwalker))
- Handle `null` value when deserializing Option [#132](https://github.com/neo4j-labs/neo4rs/pull/132) ([s1ck](https://github.com/s1ck))
- Improve parser/macro code to remove Rc<RefCell<>> wrappers [#129](https://github.com/neo4j-labs/neo4rs/pull/129) ([knutwalker](https://github.com/knutwalker))
- Update lockfiles with xtask and when initiating a release [#126](https://github.com/neo4j-labs/neo4rs/pull/126) ([knutwalker](https://github.com/knutwalker))
- Add test for the example in GH issue #108 [#125](https://github.com/neo4j-labs/neo4rs/pull/125) ([knutwalker](https://github.com/knutwalker))

## [v0.7.0-rc.1](https://github.com/neo4j-labs/neo4rs/tree/v0.7.0-rc.1) - 2023-10-20

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.6.2...v0.7.0-rc.1)

### Other

- release: neo4rs v0.7.0-rc.1 [#124](https://github.com/neo4j-labs/neo4rs/pull/124) ([github-actions](https://github.com/github-actions))
- Update Neo4j versions used in the integration tests [#123](https://github.com/neo4j-labs/neo4rs/pull/123) ([knutwalker](https://github.com/knutwalker))
- Set MSRV to 1.63 for all crates [#122](https://github.com/neo4j-labs/neo4rs/pull/122) ([knutwalker](https://github.com/knutwalker))
- Mark RowStream as `must_use` [#121](https://github.com/neo4j-labs/neo4rs/pull/121) ([knutwalker](https://github.com/knutwalker))
- Breaking! Move a lot of `get::<T>` functions to use `serde` instead of `TryFrom<BoltType>` [#120](https://github.com/neo4j-labs/neo4rs/pull/120) ([knutwalker](https://github.com/knutwalker))
- Make some bolt types `pub use` [#119](https://github.com/neo4j-labs/neo4rs/pull/119) ([knutwalker](https://github.com/knutwalker))
- Add `_on` variants to tx/run/execute that allow selecting the db per call [#117](https://github.com/neo4j-labs/neo4rs/pull/117) ([knutwalker](https://github.com/knutwalker))
- Use the MSRV lockfile for MSRV tests [#115](https://github.com/neo4j-labs/neo4rs/pull/115) ([knutwalker](https://github.com/knutwalker))
- Allow direct IP uris [#114](https://github.com/neo4j-labs/neo4rs/pull/114) ([knutwalker](https://github.com/knutwalker))
- Allow HashMap to/from BoltType conversion  [#109](https://github.com/neo4j-labs/neo4rs/pull/109) ([caamartin35](https://github.com/caamartin35))
- Allow use of deprecated `add_server_trust_anchors` [#105](https://github.com/neo4j-labs/neo4rs/pull/105) ([knutwalker](https://github.com/knutwalker))
- Share test code between integration tests and doc examples [#104](https://github.com/neo4j-labs/neo4rs/pull/104) ([knutwalker](https://github.com/knutwalker))
- Breaking! Change `labels` and `type` to return `&str` instead of `String` [#103](https://github.com/neo4j-labs/neo4rs/pull/103) ([knutwalker](https://github.com/knutwalker))
- release: neo4rs v0.7.0-alpha.1 [#102](https://github.com/neo4j-labs/neo4rs/pull/102) ([github-actions](https://github.com/github-actions))
- Add keys method to nodes/rels [#101](https://github.com/neo4j-labs/neo4rs/pull/101) ([knutwalker](https://github.com/knutwalker))
- Extract more additional data via serde [#100](https://github.com/neo4j-labs/neo4rs/pull/100) ([knutwalker](https://github.com/knutwalker))
- Implement From<Option> for BoltType [#99](https://github.com/neo4j-labs/neo4rs/pull/99) ([jifalops](https://github.com/jifalops))
- add Query::has_param_key [#98](https://github.com/neo4j-labs/neo4rs/pull/98) ([jifalops](https://github.com/jifalops))
- Initial serde integration [#96](https://github.com/neo4j-labs/neo4rs/pull/96) ([knutwalker](https://github.com/knutwalker))
- release: neo4rs v0.6.2 [#95](https://github.com/neo4j-labs/neo4rs/pull/95) ([github-actions](https://github.com/github-actions))

## [v0.6.2](https://github.com/neo4j-labs/neo4rs/tree/v0.6.2) - 2023-06-30

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.6.1...v0.6.2)

### Other

- Prepend `bolt` scheme to URI if no scheme is present [#94](https://github.com/neo4j-labs/neo4rs/pull/94) ([s1ck](https://github.com/s1ck))
- release: neo4rs v0.6.1 [#92](https://github.com/neo4j-labs/neo4rs/pull/92) ([github-actions](https://github.com/github-actions))

## [v0.6.1](https://github.com/neo4j-labs/neo4rs/tree/v0.6.1) - 2023-06-12

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.6.0...v0.6.1)

### Other

- release: neo4rs v0.6.1 [#92](https://github.com/neo4j-labs/neo4rs/pull/92) ([github-actions](https://github.com/github-actions))
- Warn when connecting against neo4j schema [#91](https://github.com/neo4j-labs/neo4rs/pull/91) ([knutwalker](https://github.com/knutwalker))
- Support encrypted connections [#88](https://github.com/neo4j-labs/neo4rs/pull/88) ([knutwalker](https://github.com/knutwalker))
- release: neo4rs v0.6.0 [#77](https://github.com/neo4j-labs/neo4rs/pull/77) ([github-actions](https://github.com/github-actions))
- Refactor Config and ConfigBuilder [#71](https://github.com/neo4j-labs/neo4rs/pull/71) ([s1ck](https://github.com/s1ck))
- Pass multiple params as a hashmap [#68](https://github.com/neo4j-labs/neo4rs/pull/68) ([0xDjole](https://github.com/0xDjole))

## [v0.6.0](https://github.com/neo4j-labs/neo4rs/tree/v0.6.0) - 2023-03-24

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.5.9...v0.6.0)

### Other

- release: neo4rs v0.6.0 [#77](https://github.com/neo4j-labs/neo4rs/pull/77) ([github-actions](https://github.com/github-actions))
- release: neo4rs v0.6.1 [#76](https://github.com/neo4j-labs/neo4rs/pull/76) ([github-actions](https://github.com/github-actions))
- release: neo4rs v0.6.0 [#74](https://github.com/neo4j-labs/neo4rs/pull/74) ([github-actions](https://github.com/github-actions))
- Update CI configuration [#72](https://github.com/neo4j-labs/neo4rs/pull/72) ([knutwalker](https://github.com/knutwalker))
- Support Neo4j 5.x alongside Neo4j 4.4 [#70](https://github.com/neo4j-labs/neo4rs/pull/70) ([knutwalker](https://github.com/knutwalker))
- bool type in params [#67](https://github.com/neo4j-labs/neo4rs/pull/67) ([0xDjole](https://github.com/0xDjole))
- Implement Error trait using thiserror. [#65](https://github.com/neo4j-labs/neo4rs/pull/65) ([SolidTux](https://github.com/SolidTux))
- Compatibility with Neo4j version 5 and 4.4. [#64](https://github.com/neo4j-labs/neo4rs/pull/64) ([SolidTux](https://github.com/SolidTux))
- Update dependencies. [#63](https://github.com/neo4j-labs/neo4rs/pull/63) ([SolidTux](https://github.com/SolidTux))
- Setup Github Actions CI [#60](https://github.com/neo4j-labs/neo4rs/pull/60) ([knutwalker](https://github.com/knutwalker))
- Eliminate warnings and deprecations from docs [#59](https://github.com/neo4j-labs/neo4rs/pull/59) ([knutwalker](https://github.com/knutwalker))
- Add more From impls for numbers [#58](https://github.com/neo4j-labs/neo4rs/pull/58) ([knutwalker](https://github.com/knutwalker))
- Fix clippy warnings [#56](https://github.com/neo4j-labs/neo4rs/pull/56) ([s1ck](https://github.com/s1ck))
- Replace Into impls with From [#55](https://github.com/neo4j-labs/neo4rs/pull/55) ([s1ck](https://github.com/s1ck))
- Fix deprecation warnings [#53](https://github.com/neo4j-labs/neo4rs/pull/53) ([s1ck](https://github.com/s1ck))
- Implement Into/TryFrom BoltType for Vec and Into for f64 [#11](https://github.com/neo4j-labs/neo4rs/pull/11) ([Peikos](https://github.com/Peikos))

## [v0.5.9](https://github.com/neo4j-labs/neo4rs/tree/v0.5.9) - 2021-09-09

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.5.8...v0.5.9)

### Other

- Parse nulls [#10](https://github.com/neo4j-labs/neo4rs/pull/10) ([titanous](https://github.com/titanous))

## [v0.5.8](https://github.com/neo4j-labs/neo4rs/tree/v0.5.8) - 2021-01-10

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/v0.5.7...v0.5.8)

## [v0.5.7](https://github.com/neo4j-labs/neo4rs/tree/v0.5.7) - 2021-01-08

[Full Changelog](https://github.com/neo4j-labs/neo4rs/compare/f951f8e2c1e7b01916bfa199d803406ca3f84433...v0.5.7)
