# Changelog

## [0.6.3]

- Add changelog.

## [0.6.2]

- Improved debug messages.


## [0.6.1]

- More benchmark tests.

## [0.6.0]

- API change: add mounted memory files support.
- improved project structure.

## [0.5.0]

- API change: to initialize with memory index range rather than starting memoryId index.

## [0.4.3]

- Support init with memory manager (requires start memory index).
- Support init with predefine environment variables #7.
- Feature: transient - choose between using transient file system and stable storage.
- Feature: report_wasi_calls - output statistical information on the called polyfill functions.
- Feature: skip_unimplemented_functions - rather than panic, the unimplemented functions are not present in the compiled wasm.
- Fixed error causing uninitialized seed usage #4.
- Benchmark tests added.