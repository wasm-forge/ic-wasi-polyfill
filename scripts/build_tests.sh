#!/bin/bash
set -e


# build fs_tests for wasi
cargo build --release --target wasm32-wasip1

candid-extractor target/wasm32-wasip1/release/fs_tests_backend.wasm > test/canisters/fs_tests/src/fs_tests_backend/fs_tests_backend.did

wasi2ic target/wasm32-wasip1/release/fs_tests_backend.wasm target/wasm32-wasip1/release/fs_tests_backend_nowasi.wasm

# build fs_tests for local launch
cargo build --release

# remove old dir and report
rm -rf target/release/report.txt
rm -rf target/release/playground

# launch in local drive
mkdir -p target/release/playground

pushd .

# launch fs_test in the local playground
cd target/release/playground

../fs_test > ../report.txt

cat ../report.txt

popd

pushd .

# prepare other canisters
candid-extractor target/wasm32-wasip1/release/canister_initial_backend.wasm > test/canisters/canister_initial/src/canister_initial_backend/canister_initial_backend.did

wasi2ic target/wasm32-wasip1/release/canister_initial_backend.wasm target/wasm32-wasip1/release/canister_initial_backend_nowasi.wasm

candid-extractor target/wasm32-wasip1/release/canister_upgraded_backend.wasm > test/canisters/canister_upgraded/src/canister_upgraded_backend/canister_upgraded_backend.did

wasi2ic target/wasm32-wasip1/release/canister_upgraded_backend.wasm target/wasm32-wasip1/release/canister_upgraded_backend_nowasi.wasm

popd

# regenerate APIs
ic-test update --force
