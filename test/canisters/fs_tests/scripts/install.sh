#!/bin/sh

export RELEASE_DIR=target/wasm32-wasip1/release

cargo clean

cargo build --release --target wasm32-wasip1

ic-wasm $RELEASE_DIR/fs_tests_backend.wasm -o $RELEASE_DIR/fs_tests_backend_meta.wasm metadata candid:service -f ./src/fs_tests_backend/fs_tests_backend.did -v public

wasi2ic $RELEASE_DIR/fs_tests_backend_meta.wasm $RELEASE_DIR/fs_tests_backend_nowasi.wasm


dfx canister create fs_tests_backend

dfx canister install --mode reinstall fs_tests_backend --wasm $RELEASE_DIR/fs_tests_backend_nowasi.wasm -y

