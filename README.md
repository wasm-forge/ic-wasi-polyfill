# The IC Polyfill library

![Tests](https://github.com/wasm-forge/ic-wasi-polyfill/actions/workflows/rust.yml/badge.svg?event=push)

The project provides polyfill implementation of WASI functions using IC System API.

## Usage

It uses a file system provided by stable-fs. The intended use is you add this library as a dependency to your project.

And then run `wasi2ic` on the produced Wasm binary.
