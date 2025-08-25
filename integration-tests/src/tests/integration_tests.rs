use ic_test::IcpTest;

use crate::{
    bindings::canister_upgraded_backend,
    test_setup::{self, Env},
};

async fn upgrade_canister(env: &Env) {
    let wasm = canister_upgraded_backend::wasm().expect("Wasm not found for the upgrade_canister");

    let user = env.icp_test.icp.default_user().principal;

    env.icp_test
        .icp
        .pic
        .upgrade_canister(
            env.canister_initial_backend.canister_id,
            wasm,
            vec![],
            Some(user),
        )
        .await
        .expect("Failed to upgrade canister!");
}

#[tokio::test]
async fn greet_after_upgrade() {
    let env = test_setup::setup(IcpTest::new().await).await;

    let result = env
        .canister_initial_backend
        .greet("ICP".to_string())
        .call()
        .await;

    assert_eq!(result, "Hello, ICP!");
    upgrade_canister(&env).await;

    let result = env
        .canister_initial_backend
        .greet("ICP".to_string())
        .call()
        .await;

    assert_eq!(result, "Greetings, ICP!");
}

#[tokio::test]
async fn writing_10mib() {
    let env = test_setup::setup(IcpTest::new().await).await;

    let _res = env
        .canister_initial_backend
        .write_mib_text("test.txt".to_string(), 10)
        .call()
        .await;
}

#[tokio::test]
async fn reading_file_after_upgrade() {
    let env = test_setup::setup(IcpTest::new().await).await;

    env.canister_initial_backend
        .append_text("d1/d2/d3/test1.txt".to_string(), "test1".to_string(), 10u64)
        .call()
        .await;

    env.canister_initial_backend
        .append_text("d1/d2/test2.txt".to_string(), "test2".to_string(), 10u64)
        .call()
        .await;

    env.canister_initial_backend
        .append_text("test3.txt".to_string(), "test3".to_string(), 10u64)
        .call()
        .await;

    env.canister_initial_backend
        .append_text("d1/d2/test2.txt".to_string(), "abc".to_string(), 10u64)
        .call()
        .await;

    let result = env
        .canister_initial_backend
        .read_text("d1/d2/test2.txt".to_string(), 45, 100)
        .call()
        .await;

    assert_eq!(result, "test2abcabcabcabcabcabcabcabcabcabc");

    // do upgrade
    upgrade_canister(&env).await;

    let result = env
        .canister_initial_backend
        .read_text("d1/d2/test2.txt".to_string(), 40, 15)
        .call()
        .await;

    assert_eq!(result, "test2test2abcab");
}

#[tokio::test]
async fn list_folders_after_upgrade() {
    let env = test_setup::setup(IcpTest::new().await).await;

    env.canister_initial_backend
        .create_files("files".to_string(), 10)
        .call()
        .await;

    env.canister_initial_backend
        .create_files("files/f2".to_string(), 10)
        .call()
        .await;

    assert_eq!(
        vec! {"0.txt", "1.txt", "2.txt", "3.txt", "4.txt", "5.txt", "6.txt", "7.txt", "8.txt", "9.txt", "f2"},
        env.canister_initial_backend
            .list_files("files".to_string())
            .call()
            .await
    );

    assert_eq!(
        vec! {"0.txt", "1.txt", "2.txt", "3.txt", "4.txt", "5.txt", "6.txt", "7.txt", "8.txt", "9.txt"},
        env.canister_initial_backend
            .list_files("files/f2".to_string())
            .call()
            .await
    );

    // do upgrade
    upgrade_canister(&env).await;

    assert_eq!(
        vec! {"0.txt", "1.txt", "2.txt", "3.txt", "4.txt", "5.txt", "6.txt", "7.txt", "8.txt", "9.txt", "f2"},
        env.canister_initial_backend
            .list_files("files".to_string())
            .call()
            .await
    );

    assert_eq!(
        vec! {"0.txt", "1.txt", "2.txt", "3.txt", "4.txt", "5.txt", "6.txt", "7.txt", "8.txt", "9.txt"},
        env.canister_initial_backend
            .list_files("files/f2".to_string())
            .call()
            .await
    );
}

#[tokio::test]
async fn create_1000_files() {
    let env = test_setup::setup(IcpTest::new().await).await;

    // TODO: create more files
    let file_count = 250;
    let path1 = "files1";
    let path2 = "files2//";
    let path3 = "files3";
    let path4 = "./files4";

    env.canister_initial_backend
        .create_files(path1.to_string(), file_count)
        .call()
        .await;
    env.canister_initial_backend
        .create_files(path2.to_string(), file_count)
        .call()
        .await;
    env.canister_initial_backend
        .create_files(path3.to_string(), file_count)
        .call()
        .await;
    env.canister_initial_backend
        .create_files(path4.to_string(), file_count)
        .call()
        .await;

    let result = env
        .canister_initial_backend
        .list_files(path2.to_string())
        .call()
        .await;

    let mut filenames = vec![];

    for i in 0..file_count {
        filenames.push(format!("{i}.txt"))
    }
    assert_eq!(result, filenames);

    let result = env
        .canister_initial_backend
        .list_files("".to_string())
        .call()
        .await;

    let filenames = vec!["files1", "files2", "files3", "files4"];

    assert_eq!(result, filenames);
}

#[tokio::test]
async fn long_paths_and_file_names() {
    let env = test_setup::setup(IcpTest::new().await).await;

    let file_count = 20;

    // Wasi max path length limit is 512 bytes, have to reduce test limits accordingly
    let long_name = "1234567890ABCD7890ABCDEFABCDEF1234567890ABCDE";
    let long_name2 = "1234567890ABCFABCDEF12345678904567890ABCDEFä";
    let long_name3 = "1234567890ABC34567890ABCDEF💖567890ABCDEFA💖";

    let mut path = "".to_string();
    for _ in 0..3 {
        path.push_str(long_name);
        path.push('/');
        path.push_str(long_name2);
        path.push('/');
        path.push_str(long_name3);
        path.push('/');
    }

    env.canister_initial_backend
        .create_files(path.clone(), file_count)
        .call()
        .await;

    let result = env
        .canister_initial_backend
        .list_files(path.clone())
        .call()
        .await;

    let mut filenames = vec![];

    for i in 0..file_count {
        filenames.push(format!("{i}.txt"))
    }
    assert_eq!(result, filenames);

    let filenames = vec![long_name];

    let result = env
        .canister_initial_backend
        .list_files("".to_string())
        .call()
        .await;
    assert_eq!(result, filenames);

    // try reading one of the files

    let file_content_start = "0123456789012345678901234567890123456789012345678901234567890123:";
    let file_name = "13.txt";
    let expected_content = format!("{file_content_start}{path}/{file_name}");
    let content_length = expected_content.len();

    let content = env
        .canister_initial_backend
        .read_text(format!("{path}/{file_name}"), 0, 100000)
        .call()
        .await;

    assert_eq!(expected_content, content);

    let expected_content = "0123:123";

    let content = env
        .canister_initial_backend
        .read_text(format!("{path}/3.txt"), 60, expected_content.len() as u64)
        .call()
        .await;

    assert_eq!(expected_content, content);

    let expected_content = "A💖//13.txt";

    let content = env
        .canister_initial_backend
        .read_text(
            format!("{path}/13.txt"),
            content_length as i64 - expected_content.len() as i64,
            100,
        )
        .call()
        .await;

    assert_eq!(expected_content, content);
}

#[tokio::test]
async fn deep_subfolder_structure() {
    let env = test_setup::setup(IcpTest::new().await).await;

    let file_count = 20;

    // wasi max path length limit is 512 bytes, have to reduce test limits accordingly
    let long_name = "A";
    let long_name2 = "B";
    let long_name3 = "C";

    let mut path = "".to_string();
    for _ in 0..83 {
        path.push_str(long_name);
        path.push('/');
        path.push_str(long_name2);
        path.push('/');
        path.push_str(long_name3);
        path.push('/');
    }

    env.canister_initial_backend
        .create_files(path.clone(), file_count)
        .call()
        .await;

    let result = env
        .canister_initial_backend
        .list_files(path.clone())
        .call()
        .await;

    let mut filenames = vec![];

    for i in 0..file_count {
        filenames.push(format!("{i}.txt"))
    }
    assert_eq!(result, filenames);

    let filenames = vec![long_name];

    let result = env
        .canister_initial_backend
        .list_files("".to_string())
        .call()
        .await;

    assert_eq!(result, filenames);

    // try reading one of the files

    let file_content_start = "0123456789012345678901234567890123456789012345678901234567890123:";
    let file_name = "13.txt";
    let expected_content = format!("{file_content_start}{path}/{file_name}");
    let content_length = expected_content.len();

    let content = env
        .canister_initial_backend
        .read_text(format!("{path}/{file_name}"), 0, 100000)
        .call()
        .await;
    assert_eq!(expected_content, content);

    let expected_content = "0123:A/B";

    let content = env
        .canister_initial_backend
        .read_text(format!("{path}/3.txt"), 60, expected_content.len() as u64)
        .call()
        .await;

    assert_eq!(expected_content, content);

    let expected_content = "C//13.txt";

    let content = env
        .canister_initial_backend
        .read_text(
            format!("{path}/13.txt"),
            content_length as i64 - expected_content.len() as i64,
            100,
        )
        .call()
        .await;

    assert_eq!(expected_content, content);
}

/* TODO: ic-test has a broken function, when multy-value tuple is returned
#[tokio::test]
async fn long_chunk() {
    let env = test_setup::setup(IcpTest::new().await).await;

    let (time, size) = env
        .canister_initial_backend
        .append_chunk("abc1234567".to_string(), 10_000_000)
        .call()
        .await;

    println!("creating the chunk time={time} size={size}");

    let (time, size) = env
        .canister_initial_backend
        .store_chunk_map_4_k(2131)
        .call()
        .await;

    println!("store_chunk_map time={time} size={size}");
}
    */

#[tokio::test]
async fn created_dir_is_writable() {
    let env = test_setup::setup(IcpTest::new().await).await;

    let result = env
        .canister_initial_backend
        .check_new_dir_is_writable("/usr/tmp".to_string())
        .call()
        .await;

    assert_eq!(result, "Is writable");
}

#[tokio::test]
async fn created_file_is_writable() {
    let env = test_setup::setup(IcpTest::new().await).await;

    let result = env
        .canister_initial_backend
        .check_new_file_is_writable("text_file.txt".to_string())
        .call()
        .await;

    assert_eq!(result, "Is writable");
}

#[tokio::test]
async fn root_dir_is_writable() {
    let env = test_setup::setup(IcpTest::new().await).await;

    let result = env
        .canister_initial_backend
        .check_dir_is_writable(".".to_string())
        .call()
        .await;

    assert_eq!(result, "Is writable");
}
