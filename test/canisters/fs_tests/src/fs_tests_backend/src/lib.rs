use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl, Memory,
};

use std::{
    cell::RefCell,
    fs::{self},
};

const PROFILING: MemoryId = MemoryId::new(100);
const WASI_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub fn profiling_init() {
    let memory = MEMORY_MANAGER.with(|m| m.borrow().get(PROFILING));
    memory.grow(4096);
}

#[ic_cdk::init]
fn init() {
    profiling_init();

    let wasi_memory = MEMORY_MANAGER.with(|m| m.borrow().get(WASI_MEMORY_ID));
    ic_wasi_polyfill::init_with_memory(&[0u8; 32], &[], wasi_memory);

    ic_wasi_polyfill::FS.with(|fs| {
        let mut fs = fs.borrow_mut();
        fs.storage
            .set_chunk_size(ic_wasi_polyfill::ChunkSize::CHUNK16K)
            .unwrap();
    });
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    profiling_init();

    let wasi_memory = MEMORY_MANAGER.with(|m| m.borrow().get(WASI_MEMORY_ID));
    ic_wasi_polyfill::init_with_memory(&[0u8; 32], &[], wasi_memory);
}

#[ic_cdk::update]
fn test_create_dir_all(dir_name: String) {
    println!("Creating directory: {}", dir_name);

    fs::create_dir_all(dir_name).unwrap();
}

#[ic_cdk::update]
fn test_read_dir(dir_name: String) -> Vec<String> {
    println!("Reading directory: {}", dir_name);

    let mut res = vec![];
    let entries = fs::read_dir(dir_name).unwrap();

    for entry in entries {
        let entry = entry.unwrap();

        res.push(entry.path().into_os_string().into_string().unwrap());
    }

    res
}

#[ic_cdk::update]
fn read_file(file_name: String) -> String {
    println!("Reading file: {:?}", file_name);

    let data: Vec<u8> = std::fs::read(file_name).unwrap();

    let res = String::from_utf8(data).unwrap();

    res
}

#[ic_cdk::update]
fn write_file(file_name: String, content: String) {
    println!("Writing file: {:?}", file_name);

    std::fs::write(file_name, content).unwrap();
}
