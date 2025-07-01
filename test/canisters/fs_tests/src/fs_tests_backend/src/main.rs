use std::{env, fs};

mod canister;
fn main() {
    let _ = fs::remove_dir_all("playground");
    fs::create_dir_all("playground").unwrap();

    let path = env::current_dir().unwrap();

    env::set_current_dir("playground").unwrap();

    let scan = canister::do_fs_test_basic(42);

    println!("{scan}");

    let _ = env::set_current_dir(path);
}
