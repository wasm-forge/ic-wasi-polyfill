# The IC Polyfill library

![Tests](https://github.com/wasm-forge/ic-wasi-polyfill/actions/workflows/rust.yml/badge.svg?event=push)

The project provides polyfill implementation of WASI functions using IC System API.

## Usage

The intended use is to add this library as a dependency to your rust project. And then run `wasi2ic` on the produced Wasm binary.
