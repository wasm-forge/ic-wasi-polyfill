use std::{fs::File, io::Write};

use crate::canister::{generate_random_fs, new_log, scan_directory};

pub fn do_fs_test_interal() -> String {
    new_log("log.txt");
    generate_random_fs(5, 200, 20);
    //failing_test();

    scan_directory(".".to_string())
}

// we don't have dangling file support yet
pub fn failing_dangling_file_test() {
    let mut file_a = File::create("a.txt").unwrap();
    file_a.flush().unwrap();

    {
        // this operation fails if there is no dangling file support
        let _ = std::fs::remove_file("a.txt");
    }
}

pub fn open_file_test() {
    //truncate=false,write=false,append=false,create=true
    // special combination fails
    let res = std::fs::OpenOptions::new()
        .write(false)
        .read(true)
        .append(true)
        .truncate(false)
        .create(true)
        .open("some_file.txt")
        .unwrap();
}
