#!/bin/bash

rm *.wasm

set -e


cd ../../../ic-wasi-polyfill
pwd

cargo build --release --target wasm32-wasip1 --features "report_wasi_calls"

cd ../test_canisters/c_tests/src

$WASI_SDK_PATH/bin/clang++ -mexec-model=reactor -fno-exceptions main.cpp -L../../../../ic-wasi-polyfill/target/wasm32-wasip1/release -lic_wasi_polyfill -o main.wasm
wasi2ic main.wasm nowasi.wasm

dfx canister create c_tests_backend
dfx canister install -y --mode reinstall --wasm nowasi.wasm c_tests_backend

#dfx canister call c_tests_backend greet --output raw --type raw `echo "world" | xxd -p` | xxd -p -r
#dfx canister call c_tests_backend test_access --output raw 
dfx canister call c_tests_backend test_stat --output raw 

