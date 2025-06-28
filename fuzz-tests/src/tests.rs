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

#[tokio::test]
async fn test_fs_durability() {
    let cur = std::env::current_dir().unwrap();
    println!("Current folder: {cur:?}");

    let test_setup::Env {
        icp_test: _,
        fs_tests_backend,
    } = test_setup::setup(IcpTest::new().await).await;

    let result = fs_tests_backend.do_fs_test_basic().call().await;

    let expected = std::fs::read_to_string("../target/release/report.txt").unwrap();

    println!("result:\n{result}\n\n\n");
    println!("expected:\n{expected}\n\n\n");

    assert_eq!(result, expected);
}
