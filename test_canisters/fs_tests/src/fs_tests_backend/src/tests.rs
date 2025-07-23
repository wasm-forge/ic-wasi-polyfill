use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
};

use crate::canister::{generate_random_fs, new_log, scan_directory};

pub fn do_fs_test_interal() -> String {
    new_log("log.txt");
    generate_random_fs(42, 3500, 20);
    //failing_dangling_file_test();
    //open_file_test();
    //cache_failure_test();

    scan_directory(".".to_string())
}

// we don't have dangling file support yet
#[allow(dead_code)]
pub fn failing_dangling_file_test() {
    let mut file_a = File::create("a.txt").unwrap();
    file_a.flush().unwrap();

    {
        // this operation fails if there is no dangling file support
        let _ = std::fs::remove_file("a.txt");
    }
}

#[allow(dead_code)]
pub fn open_file_test() {
    // special combination fails

    for i in 0..32 {
        let r = i % 2;
        let w = (i >> 1) % 2;
        let a = (i >> 2) % 2;
        let t = (i >> 3) % 2;
        let c = (i >> 4) % 2;

        let fname = format!("r{r}w{w}a{a}t{t}c{c}.txt");

        let _r = std::fs::OpenOptions::new()
            .read(r == 1)
            .write(w == 1)
            .append(a == 1)
            .truncate(t == 1)
            .create(c == 1)
            .open(&fname);
    }
}

#[allow(dead_code)]
pub fn cache_failure_test() {
    let filename = "cache.txt";
    let offset_step = 1024 * 64; // 64K - guarantees different chunks of every setup

    {
        // initial write
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)
            .unwrap();

        let _ = file.seek(SeekFrom::Start(offset_step));
        let _ = file.write_all(&[2]);
    }

    {
        // truncate file
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)
            .unwrap();

        let _ = file.seek(SeekFrom::Start(2 * offset_step));
        let _ = file.write_all(&[7]);
        let _ = file.seek(SeekFrom::Start(offset_step));
        let _ = file.write_all(&[6]);
    }
}
