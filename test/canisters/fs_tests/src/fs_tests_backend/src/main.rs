use std::{env, fs};

use crate::canister::seed_rand;

mod canister;
fn main() {
    seed_rand(42);

    let _ = fs::remove_dir_all("playground");
    fs::create_dir("playground").unwrap();

    let path = env::current_dir().unwrap();

    env::set_current_dir("playground").unwrap();

    let scan = canister::do_fs_test_basic();

    println!("{scan}");

    let _ = env::set_current_dir(path);
}
