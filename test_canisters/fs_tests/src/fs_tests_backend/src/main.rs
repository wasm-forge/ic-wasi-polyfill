use std::{env, fs};

mod canister;
mod tests;

fn main() {
    let _ = fs::remove_dir_all("playground");
    fs::create_dir_all("playground").unwrap();

    let path = env::current_dir().unwrap();

    env::set_current_dir("playground").unwrap();

    // double fs_test call (for the canister upgrade imitation)
    let _scan = canister::do_fs_test();
    //
    let scan = canister::do_fs_test();

    println!("{scan}");

    let _ = env::set_current_dir(path);
}
