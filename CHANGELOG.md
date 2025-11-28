# Changelog

## [v0.12.0]
- Update to ic-cdk v0.19
- Update depencenties

## [v0.11.1]
- WASI call counting it enabled by an explicit feature
- Update dependencies

## [v0.11.0]
- API change, introduce MountedFileSizePolicy
- Update dependencies

## [v0.10.0]
- Make static RNG and ENV variables public
- Add WASI cumulative instruction counter
- Integrate testing of C hello world
- Add flame graph estimations for folders
- Refactor Metadata and DirEntry structures
- Remove generating a vector on listing directory entries -> faster folder iteration
- Add testing of upgrading from v0.9.0 to v0.10.0
- Use with_direntries instead of get_direntries
- Remove get_direntries


## [v0.9.0]
- Update stable-structures to v0.7.0
- Update dependecies
- Introduce ic-test
- Add durability tests

## [v0.8.2]
- Update version ic-cdk v0.18.3
- Update pocket-ic to v9.0
- Update dependecies

## [v0.8.1]
- Refine unsafe blocks in the 2024 edition

## [v0.8.0]

- Pocket-ic 7.0/8.0
- Update to rust edition 2024
- Update dependencies
- Update documentation 
- Additional tests for wasi compliance
- Refactor test structure, 
- Wasi_mock updated to be used with wasmtime tests

## [v0.7.0]

- More supported functions
- Update dependencies

## [v0.6.4]

- Update Pocket-ic client version to V0.5.
- Update dependencies to the latest versions.
- Add default implementation for poll_oneoff to avoid panic by default.

## [v0.6.3]

- Add changelog.
- Add some exploration tests for the C language canisters.
- Fix file access status error.

## [v0.6.2]

- Improved debug messages.


## [v0.6.1]

- More benchmark tests.

## [v0.6.0]

- API change: add mounted memory files support.
- improved project structure.

## [v0.5.0]

- API change: to initialize with memory index range rather than starting memoryId index.

## [v0.4.3]

- Support init with memory manager (requires start memory index).
- Support init with predefined environment variables (#7).
- Benchmark tests added.


[v0.11.1]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.11.0...v0.11.1
[v0.11.0]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.10.0...v0.11.0
[v0.10.0]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.9.0...v0.10.0
[v0.9.0]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.8.2...v0.9.0
[v0.8.2]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.8.1...v0.8.2
[v0.8.1]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.8.0...v0.8.1
[v0.8.0]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.7.0...v0.8.0
[v0.7.0]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.6.4...v0.7.0
[v0.6.4]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.6.3...v0.6.4
[v0.6.3]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.6.2...v0.6.3
[v0.6.2]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.6.1...v0.6.2
[v0.6.1]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.6.0...v0.6.1
[v0.6.0]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.5.0...v0.6.0
[v0.5.0]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.4.3...v0.5.0
[v0.4.3]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/83c82d0bebd0e2fbe09ad5a4acb6f1ab1b3a6e0d...v0.4.3
