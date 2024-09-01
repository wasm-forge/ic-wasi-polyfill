#!/bin/bash

cd tests/canister_initial

dfx canister create canister_initial_backend

pwd

ls

dfx canister install --mode reinstall -y --wasm target/wasm32-wasi/release/canister_initial_backend_nowasi.wasm canister_initial_backend 



