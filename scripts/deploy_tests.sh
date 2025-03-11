#!/bin/bash

cd test/canister_initial

dfx canister create canister_initial_backend

pwd

ls

dfx canister install --mode reinstall -y --wasm target/wasm32-wasip1/release/canister_initial_backend_nowasi.wasm canister_initial_backend 



