use std::path::PathBuf;

use candid::Principal;
use ic_test::{IcpTest, IcpUser};

use crate::bindings::{
    canister_initial_backend::{self, CanisterInitialBackendCanister},
    canister_upgraded_backend::{self, CanisterUpgradedBackendCanister},
    fs_tests_backend::{self, FsTestsBackendCanister},
};

pub(crate) struct Env {
    pub icp_test: IcpTest,
    pub canister_initial_backend: CanisterInitialBackendCanister,
    pub canister_upgraded_backend: CanisterUpgradedBackendCanister,
    pub fs_tests_backend: FsTestsBackendCanister,
}

pub(crate) async fn setup(icp_test: IcpTest) -> Env {
    let icp_user = icp_test.icp.test_user(0);

    // initialize canisters

    let canister_initial_backend = canister_initial_backend::deploy(&icp_user).call().await;

    let canister_upgraded_backend = canister_upgraded_backend::deploy(&icp_user).call().await;

    let fs_tests_backend = fs_tests_backend::deploy(&icp_user).call().await;

    // Additional setup steps
    // ...

    Env {
        icp_test,
        canister_initial_backend,
        canister_upgraded_backend,
        fs_tests_backend,
    }
}
