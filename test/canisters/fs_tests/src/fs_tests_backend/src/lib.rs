use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl, Memory,
};

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        // Constants from Numerical Recipes
        const A: u64 = 6364136223846793005;
        const C: u64 = 1;

        self.state = self.state.wrapping_mul(A).wrapping_add(C);
        self.state
    }
}

use std::{
    cell::RefCell,
    fs::{self},
};

const PROFILING: MemoryId = MemoryId::new(100);
const WASI_MEMORY_ID: MemoryId = MemoryId::new(11);

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

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

#[ic_cdk::update]
fn compute_file_hash(path: String) -> String {
    let file = File::open(path).expect("I/O error");
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 4096];

    loop {
        let n = reader.read(&mut buffer).expect("I/O error");

        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let hash = hasher.finalize();

    format!("{:x}", hash)
}

#[ic_cdk::update]
pub fn basic_fs_test() {
    let file_path = "test_file.txt";
    let test_content = "Hello, Rust!";

    let mut file = File::create(file_path).expect("Failed to create file");
    file.write_all(test_content.as_bytes())
        .expect("Failed to write to file");

    let mut read_content = String::new();
    let mut file = File::open(file_path).expect("Failed to open file");
    file.read_to_string(&mut read_content)
        .expect("Failed to read file");

    assert_eq!(read_content, test_content);
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

    String::from_utf8(data).unwrap()
}

#[ic_cdk::update]
fn write_file(file_name: String, content: String) {
    println!("Writing file: {:?}", file_name);

    std::fs::write(file_name, content).unwrap();
}
