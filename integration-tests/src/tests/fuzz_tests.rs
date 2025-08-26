use std::{fs::OpenOptions, io::Write};

use ic_test::IcpTest;

use crate::{
    bindings::fs_tests_backend,
    test_setup::{self},
};

#[tokio::test]
async fn test_basic_fs_check() {
    let cur = std::env::current_dir().unwrap();
    println!("Current folder: {cur:?}");

    let env = test_setup::setup(IcpTest::new().await).await;

    env.fs_tests_backend.basic_fs_test().call().await;
}

#[tokio::test]
async fn test_fs_durability() {
    let cur = std::env::current_dir().unwrap();
    println!("Current folder: {cur:?}");

    let env = test_setup::setup(IcpTest::new().await).await;

    let backend = env.fs_tests_backend;

    let _c = backend.do_fs_test().call().await;

    // re-deploy
    let wasm = fs_tests_backend::wasm().expect("Wasm not found for the upgrade_canister");
    let user = env.icp_test.icp.default_user().principal;
    env.icp_test
        .icp
        .pic
        .upgrade_canister(backend.canister_id, wasm, vec![], Some(user))
        .await
        .expect("Failed to upgrade canister!");

    let c = backend.do_fs_test().call().await;

    let computed = c.trim();

    let e = std::fs::read_to_string("../target/release/report.txt").unwrap();
    let expected = e.trim();

    let expected_log_ =
        std::fs::read_to_string("../target/release/playground/playground/log.txt").unwrap();
    let expected_log = expected_log_.trim();

    if computed != expected {
        let computed_log = backend.get_log().call().await;

        // write scans and logs into a separate files for comparisons
        let mut a = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("a_computed.txt")
            .unwrap();

        a.write_all(computed.trim().as_bytes()).unwrap();
        a.write_all("\n".as_bytes()).unwrap();
        a.write_all(computed_log.trim().as_bytes()).unwrap();

        let mut b = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("b_expected.txt")
            .unwrap();

        b.write_all(expected.trim().as_bytes()).unwrap();
        b.write_all("\n".as_bytes()).unwrap();
        b.write_all(expected_log.trim().as_bytes()).unwrap();

        assert_eq!(computed, expected);
    }
}
