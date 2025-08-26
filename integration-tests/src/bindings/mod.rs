// THIS IS A GENERATED FILE, DO NOT EDIT!
#![allow(dead_code, unused_imports, non_snake_case)]

type CallMode = ic_test::CallMode;
type Caller = ic_test::IcpUser;
type CallBuilder<R> = ic_test::CallBuilder<R, ic_test::IcpUser>;
type DeployMode = ic_test::DeployMode;
type Deployer = ic_test::IcpUser;
type DeployBuilder<C> = ic_test::DeployBuilder<C, Caller>;

// candid: test_canisters/c_tests/src/c_tests_backend.did
pub mod c_tests_backend;

// candid: test_canisters/canister_initial/src/canister_initial_backend/canister_initial_backend.did
pub mod canister_initial_backend;

// candid: test_canisters/canister_upgraded/src/canister_upgraded_backend/canister_upgraded_backend.did
pub mod canister_upgraded_backend;

// candid: test_canisters/fs_tests/src/fs_tests_backend/fs_tests_backend.did
pub mod fs_tests_backend;
