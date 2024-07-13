#!/bin/bash
rustup target add wasm32-wasi

mkdir src/tests/svg

cd src/tests/fs_tests

cargo build --release --target wasm32-wasi

wasi2ic target/wasm32-wasi/release/fs_tests_backend.wasm target/wasm32-wasi/release/fs_tests_backend_nowasi.wasm

cd ../benchmark_test

cargo build --release --target wasm32-wasi

wasi2ic target/wasm32-wasi/release/benchmark_test_backend.wasm target/wasm32-wasi/release/benchmark_test_backend_nowasi.wasm

cd ../benchmark_test_upgraded

cargo build --release --target wasm32-wasi

wasi2ic target/wasm32-wasi/release/benchmark_test_upgraded_backend.wasm target/wasm32-wasi/release/benchmark_test_upgraded_backend_nowasi.wasm

