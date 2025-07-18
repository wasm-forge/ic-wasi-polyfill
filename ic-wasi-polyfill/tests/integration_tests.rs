use candid::Principal;
use pocket_ic::PocketIc;
use std::{cell::RefCell, fs};

const BACKEND_WASM: &str = "../test/canisters/canister_initial/target/wasm32-wasip1/release/canister_initial_backend_nowasi.wasm";
const BACKEND_WASM_UPGRADED: &str = "../test/canisters/canister_upgraded/target/wasm32-wasip1/release/canister_upgraded_backend_nowasi.wasm";

thread_local!(
    static ACTIVE_CANISTER: RefCell<Option<Principal>> = const { RefCell::new(None) };
);

fn set_active_canister(new_canister: Principal) {
    ACTIVE_CANISTER.with(|canister_cell| {
        let mut canister = canister_cell.borrow_mut();
        *canister = Some(new_canister);
    })
}

fn active_canister() -> Principal {
    ACTIVE_CANISTER.with(|canister_cell| {
        let canister = *canister_cell.borrow();
        canister.unwrap()
    })
}

fn setup_test_projects() {
    use std::process::Command;
    let _ = Command::new("bash")
        .arg("build_tests.sh")
        .output()
        .expect("Failed to execute command");
}

fn setup_initial_canister() -> PocketIc {
    setup_test_projects();
    let pic = PocketIc::new();

    let wasm = fs::read(BACKEND_WASM).expect("Wasm file not found, run 'dfx build'.");

    let backend_canister = pic.create_canister();

    pic.add_cycles(backend_canister, 1_000_000_000_000);

    set_active_canister(backend_canister);

    pic.install_canister(backend_canister, wasm, vec![], None);

    pic.tick();

    pic
}

fn upgrade_canister(pic: &PocketIc) {
    setup_test_projects();

    let wasm_upgraded =
        fs::read(BACKEND_WASM_UPGRADED).expect("Wasm file not found, run 'dfx build'.");

    pic.upgrade_canister(active_canister(), wasm_upgraded, vec![], None)
        .unwrap();
}

mod fns {

    use candid::{decode_one, encode_one, Principal};
    use pocket_ic::PocketIc;

    use super::active_canister;

    pub(crate) fn greet(pic: &PocketIc, arg: &str) -> String {
        let response = pic
            .query_call(
                active_canister(),
                Principal::anonymous(),
                "greet",
                encode_one(arg).unwrap(),
            )
            .unwrap();

        let result: String = decode_one(&response).unwrap();
        result
    }

    pub(crate) fn check_new_dir_is_writable(pic: &PocketIc, dirname: &str) -> String {
        let response = pic
            .update_call(
                active_canister(),
                Principal::anonymous(),
                "check_new_dir_is_writable",
                encode_one(dirname).unwrap(),
            )
            .unwrap();

        let result: String = decode_one(&response).unwrap();
        result
    }

    pub(crate) fn check_dir_is_writable(pic: &PocketIc, dirname: &str) -> String {
        let response = pic
            .update_call(
                active_canister(),
                Principal::anonymous(),
                "check_dir_is_writable",
                encode_one(dirname).unwrap(),
            )
            .unwrap();

        let result: String = decode_one(&response).unwrap();
        result
    }

    pub(crate) fn check_new_file_is_writable(pic: &PocketIc, dirname: &str) -> String {
        let response = pic
            .update_call(
                active_canister(),
                Principal::anonymous(),
                "check_new_file_is_writable",
                encode_one(dirname).unwrap(),
            )
            .unwrap();

        let result: String = decode_one(&response).unwrap();
        result
    }

    pub(crate) fn append_text(pic: &PocketIc, filename: &str, content: &str, count: u64) {
        pic.update_call(
            active_canister(),
            Principal::anonymous(),
            "append_text",
            candid::encode_args((filename, content, count)).unwrap(),
        )
        .unwrap();
    }

    pub(crate) fn read_text(pic: &PocketIc, filename: &str, offset: i64, size: u64) -> String {
        let response = pic
            .query_call(
                active_canister(),
                Principal::anonymous(),
                "read_text",
                candid::encode_args((filename, offset, size)).unwrap(),
            )
            .unwrap();

        let result: String = decode_one(&response).unwrap();

        result
    }

    pub(crate) fn create_files(pic: &PocketIc, path: &str, count: u64) {
        pic.update_call(
            active_canister(),
            Principal::anonymous(),
            "create_files",
            candid::encode_args((path, count)).unwrap(),
        )
        .unwrap();
    }

    pub(crate) fn list_files(pic: &PocketIc, path: &str) -> Vec<String> {
        let response = pic
            .query_call(
                active_canister(),
                Principal::anonymous(),
                "list_files",
                encode_one(path).unwrap(),
            )
            .unwrap();

        let result: Vec<String> = decode_one(&response).unwrap();

        result
    }

    pub(crate) fn append_chunk(pic: &PocketIc, text: &str, times: usize) -> (u64, usize) {
        let response = pic
            .update_call(
                active_canister(),
                Principal::anonymous(),
                "append_chunk",
                candid::encode_args((text, times)).unwrap(),
            )
            .unwrap();

        let (time, size): (u64, usize) = candid::decode_args(&response).unwrap();

        (time, size)
    }

    //pub fn store_chunk_map(key: u64) -> (u64, usize)
    pub(crate) fn store_chunk_map4k(pic: &PocketIc, key: u64) -> (u64, usize) {
        let response = pic
            .update_call(
                active_canister(),
                Principal::anonymous(),
                "store_chunk_map4k",
                candid::encode_one(key).unwrap(),
            )
            .unwrap();

        let (time, size): (u64, usize) = candid::decode_args(&response).unwrap();

        (time, size)
    }
}

#[test]
fn greet_after_upgrade() {
    let pic = setup_initial_canister();

    let result = fns::greet(&pic, "ICP");

    assert_eq!(result, "Hello, ICP!");

    upgrade_canister(&pic);

    let result = fns::greet(&pic, "ICP");

    assert_eq!(result, "Greetings, ICP!");
}

#[test]
fn writing_10mib() {
    let pic = setup_initial_canister();

    let args = candid::encode_args(("test.txt", 10u64)).unwrap();

    pic.add_cycles(active_canister(), 1_000_000_000_000_000);

    pic.tick();

    let _response = pic
        .update_call(
            active_canister(),
            Principal::anonymous(),
            "write_mib_text",
            args,
        )
        .unwrap();
}

#[test]
fn reading_file_after_upgrade() {
    let pic = setup_initial_canister();

    fns::append_text(&pic, "d1/d2/d3/test1.txt", "test1", 10u64);
    fns::append_text(&pic, "d1/d2/test2.txt", "test2", 10u64);
    fns::append_text(&pic, "test3.txt", "test3", 10u64);
    fns::append_text(&pic, "d1/d2/test2.txt", "abc", 10u64);

    let result = fns::read_text(&pic, "d1/d2/test2.txt", 45i64, 100u64);
    assert_eq!(result, "test2abcabcabcabcabcabcabcabcabcabc");

    // do upgrade
    upgrade_canister(&pic);

    let result = fns::read_text(&pic, "d1/d2/test2.txt", 40i64, 15u64);
    assert_eq!(result, "test2test2abcab");
}

#[test]
fn list_folders_after_upgrade() {
    let pic = setup_initial_canister();

    fns::create_files(&pic, "files", 10);
    fns::create_files(&pic, "files/f2", 10);

    assert_eq!(
        vec! {"0.txt", "1.txt", "2.txt", "3.txt", "4.txt", "5.txt", "6.txt", "7.txt", "8.txt", "9.txt", "f2"},
        fns::list_files(&pic, "files")
    );

    assert_eq!(
        vec! {"0.txt", "1.txt", "2.txt", "3.txt", "4.txt", "5.txt", "6.txt", "7.txt", "8.txt", "9.txt"},
        fns::list_files(&pic, "files/f2")
    );

    // do upgrade
    upgrade_canister(&pic);

    assert_eq!(
        vec! {"0.txt", "1.txt", "2.txt", "3.txt", "4.txt", "5.txt", "6.txt", "7.txt", "8.txt", "9.txt", "f2"},
        fns::list_files(&pic, "files")
    );

    assert_eq!(
        vec! {"0.txt", "1.txt", "2.txt", "3.txt", "4.txt", "5.txt", "6.txt", "7.txt", "8.txt", "9.txt"},
        fns::list_files(&pic, "files/f2")
    );
}

#[test]
fn create_1000_files() {
    let pic = setup_initial_canister();

    // TODO: create more files
    let file_count = 250;
    let path1 = "files1";
    let path2 = "files2//";
    let path3 = "files3";
    let path4 = "./files4";

    fns::create_files(&pic, path1, file_count);
    fns::create_files(&pic, path2, file_count);
    fns::create_files(&pic, path3, file_count);
    fns::create_files(&pic, path4, file_count);

    let result = fns::list_files(&pic, path2);

    let mut filenames = vec![];

    for i in 0..file_count {
        filenames.push(format!("{i}.txt"))
    }
    assert_eq!(result, filenames);

    let result = fns::list_files(&pic, "");

    let filenames = vec!["files1", "files2", "files3", "files4"];

    assert_eq!(result, filenames);
}

#[test]
fn long_paths_and_file_names() {
    let pic = setup_initial_canister();

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

    fns::create_files(&pic, &path, file_count);

    let result = fns::list_files(&pic, &path);

    let mut filenames = vec![];

    for i in 0..file_count {
        filenames.push(format!("{i}.txt"))
    }
    assert_eq!(result, filenames);

    let filenames = vec![long_name];

    let result = fns::list_files(&pic, "");
    assert_eq!(result, filenames);

    // try reading one of the files

    let file_content_start = "0123456789012345678901234567890123456789012345678901234567890123:";
    let file_name = "13.txt";
    let expected_content = format!("{file_content_start}{path}/{file_name}");
    let content_length = expected_content.len();

    let content = fns::read_text(&pic, &format!("{path}/{file_name}"), 0, 100000);
    assert_eq!(expected_content, content);

    let expected_content = "0123:123";
    let content = fns::read_text(
        &pic,
        &format!("{path}/3.txt"),
        60,
        expected_content.len() as u64,
    );
    assert_eq!(expected_content, content);

    let expected_content = "A💖//13.txt";
    let content = fns::read_text(
        &pic,
        &format!("{path}/13.txt"),
        content_length as i64 - expected_content.len() as i64,
        100,
    );

    assert_eq!(expected_content, content);
}

#[test]
fn deep_subfolder_structure() {
    let pic = setup_initial_canister();

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

    fns::create_files(&pic, &path, file_count);

    let result = fns::list_files(&pic, &path);

    let mut filenames = vec![];

    for i in 0..file_count {
        filenames.push(format!("{i}.txt"))
    }
    assert_eq!(result, filenames);

    let filenames = vec![long_name];

    let result = fns::list_files(&pic, "");
    assert_eq!(result, filenames);

    // try reading one of the files

    let file_content_start = "0123456789012345678901234567890123456789012345678901234567890123:";
    let file_name = "13.txt";
    let expected_content = format!("{file_content_start}{path}/{file_name}");
    let content_length = expected_content.len();

    let content = fns::read_text(&pic, &format!("{path}/{file_name}"), 0, 100000);
    assert_eq!(expected_content, content);

    let expected_content = "0123:A/B";
    let content = fns::read_text(
        &pic,
        &format!("{path}/3.txt"),
        60,
        expected_content.len() as u64,
    );
    assert_eq!(expected_content, content);

    let expected_content = "C//13.txt";
    let content = fns::read_text(
        &pic,
        &format!("{path}/13.txt"),
        content_length as i64 - expected_content.len() as i64,
        100,
    );

    assert_eq!(expected_content, content);
}

#[test]
fn long_chunk() {
    let pic = setup_initial_canister();

    let (time, size) = fns::append_chunk(&pic, "abc1234567", 10_000_000);

    println!("creating the chunk time={time} size={size}");

    //    let (time, size) = fns::store_chunk(&pic, "text.txt");
    let (time, size) = fns::store_chunk_map4k(&pic, 2131);

    println!("store_chunk_map time={time} size={size}");
}

#[test]
fn created_dir_is_writable() {
    let pic = setup_initial_canister();

    let result = fns::check_new_dir_is_writable(&pic, "/usr/tmp");

    assert_eq!(result, "Is writable");
}

#[test]
fn created_file_is_writable() {
    let pic = setup_initial_canister();

    let result = fns::check_new_file_is_writable(&pic, "text_file.txt");

    assert_eq!(result, "Is writable");
}

#[test]
fn root_dir_is_writable() {
    let pic = setup_initial_canister();

    let result = fns::check_dir_is_writable(&pic, ".");

    assert_eq!(result, "Is writable");
}
