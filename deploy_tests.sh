#!/bin/bash

cd src/tests/benchmark_test

dfx canister create benchmark_test_backend

pwd

ls

dfx canister install --mode reinstall -y --wasm target/wasm32-wasi/release/benchmark_test_backend_nowasi.wasm benchmark_test_backend 



