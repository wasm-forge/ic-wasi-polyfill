use std::path::PathBuf;

use candid::Principal;
use ic_test::{IcpTest, IcpUser};

use crate::bindings::fs_tests_backend::{self, FsTestsBackendCanister};

use crate::test_setup;

#[tokio::test]
async fn test_() {
    let test_setup::Env {
        icp_test,
        fs_tests_backend,
    } = test_setup::setup(IcpTest::new().await).await;

    // Your test code
    // ...

    // example calls
    // let result = fs_tests_backend./*canister method name*/().call().await;
}
