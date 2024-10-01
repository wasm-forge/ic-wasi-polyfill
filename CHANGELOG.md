# Changelog

## [unreleased]

- Add changelog.
- Add some exploration tests for the C language canisters.

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
- Support init with predefine environment variables (#7).
- Feature: transient - choose between using transient file system and stable storage.
- Feature: report_wasi_calls - output statistical information on the called polyfill functions.
- Feature: skip_unimplemented_functions - rather than panic, the unimplemented functions are not present in the compiled wasm.
- Fixed error causing uninitialized seed usage (#4).
- Benchmark tests added.


[unreleased]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.6.2...main
[v0.6.2]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.6.1...v0.6.2
[v0.6.1]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.6.0...v0.6.1
[v0.6.0]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.5.0...v0.6.0
[v0.5.0]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/v0.4.3...v0.5.0
[v0.4.3]: https://github.com/wasm-forge/ic-wasi-polyfill/compare/83c82d0bebd0e2fbe09ad5a4acb6f1ab1b3a6e0d...v0.4.3
