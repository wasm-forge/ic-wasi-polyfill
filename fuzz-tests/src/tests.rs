use ic_test::IcpTest;

use crate::test_setup;

#[tokio::test]
async fn test_basic_fs_check() {
    let cur = std::env::current_dir().unwrap();
    println!("Current folder: {cur:?}");

    let test_setup::Env {
        icp_test: _,
        fs_tests_backend,
    } = test_setup::setup(IcpTest::new().await).await;

    fs_tests_backend.basic_fs_test().call().await;
}
