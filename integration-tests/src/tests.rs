//! This is a sample test file, it shows an example of how to create actual tests.
//! The file is only generated once and won't be overwritten.

use ic_test::IcpTest;

use crate::test_setup;

/// This is an example test function. It shows how to create a test environment and use it to call canister methods.
/// Update it to actually do testing.
#[tokio::test]
async fn test_() {
    // Create test environment
    let env = test_setup::setup(IcpTest::new().await).await;

    // let result = env./*canister name*/./*canister method name*/(/*parameters*/).call().await;

    // ...
}
