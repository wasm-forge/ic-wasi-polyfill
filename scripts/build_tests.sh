#!/bin/bash
set -e


# build fs_tests for wasi
cargo build --release --target wasm32-wasip1

wasi2ic target/wasm32-wasip1/release/fs_tests_backend.wasm target/wasm32-wasip1/release/fs_tests_backend_nowasi.wasm

# build fuzz-tests 
cargo build --release 

# build other test canisters
cd test/canisters/canister_initial

cargo build --release --target wasm32-wasip1

wasi2ic target/wasm32-wasip1/release/canister_initial_backend.wasm target/wasm32-wasip1/release/canister_initial_backend_nowasi.wasm

cd ../canister_upgraded

cargo build --release --target wasm32-wasip1

wasi2ic target/wasm32-wasip1/release/canister_upgraded_backend.wasm target/wasm32-wasip1/release/canister_upgraded_backend_nowasi.wasm


