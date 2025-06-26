#![allow(dead_code)]

use ic_test::IcpTest;

use crate::bindings::fs_tests_backend::{self, FsTestsBackendCanister};

pub(crate) struct Env {
    pub icp_test: IcpTest,
    pub fs_tests_backend: FsTestsBackendCanister,
}

pub(crate) async fn setup(icp_test: IcpTest) -> Env {
    let icp_user = icp_test.icp.test_user(0);

    // initialize canisters

    let fs_tests_backend = fs_tests_backend::deploy(&icp_user).call().await;

    // Additional setup steps
    // ...

    Env {
        icp_test,
        fs_tests_backend,
    }
}
