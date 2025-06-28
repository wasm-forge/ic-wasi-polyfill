use ic_cdk::export_candid;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl, Memory,
};

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

use std::{
    cell::RefCell,
    fs::{self},
};

pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn seed(&mut self, seed: u64) {
        self.state = seed;
    }

    pub fn next_rand(&mut self) -> u64 {
        const A: u64 = 6364136223846793005;
        const C: u64 = 1;

        self.state = self.state.wrapping_mul(A).wrapping_add(C);
        self.state
    }
}

const PROFILING: MemoryId = MemoryId::new(100);
const WASI_MEMORY_ID: MemoryId = MemoryId::new(11);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static RND: RefCell<SimpleRng> =
        const { RefCell::new(SimpleRng { state: 42 }) };
}

fn next_rand() -> u64 {
    RND.with(|rng| rng.borrow_mut().next_rand())
}

fn seed_rand(seed: u64) {
    RND.with(|rng| rng.borrow_mut().seed(seed));
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

    format!("{hash:x}")
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
    println!("Creating directory: {dir_name}");

    fs::create_dir_all(dir_name).unwrap();
}

#[ic_cdk::update]
fn test_read_dir(dir_name: String) -> Vec<String> {
    println!("Reading directory: {dir_name}");

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
    println!("Reading file: {file_name:?}");

    let data: Vec<u8> = std::fs::read(file_name).unwrap();

    String::from_utf8(data).unwrap()
}

#[ic_cdk::update]
fn write_file(file_name: String, content: String) {
    println!("Writing file: {file_name:?}");

    std::fs::write(file_name, content).unwrap();
}

fn do_scan_dir_entry(
    current_path: &std::path::Path,
    collected_paths: &mut Vec<String>,
) -> anyhow::Result<()> {
    let metadata = fs::metadata(current_path)?;

    if metadata.is_dir() {
        // count number of cur dir elements
        let count = fs::read_dir(current_path)?.count();

        // Add current directory itself and the number of elements
        let entry_info = format!("{} {}/.", current_path.display(), count);
        collected_paths.push(entry_info);

        for entry in fs::read_dir(current_path)? {
            let entry = entry?;
            let path = entry.path();

            do_scan_dir_entry(&path, collected_paths)?;
        }
    } else if metadata.is_file() {
        // compute hash of a file
        let hash = compute_file_hash(current_path.to_string_lossy().to_string());

        let entry_info = format!("{} {}", current_path.display(), hash);
        collected_paths.push(entry_info);
    } else if metadata.file_type().is_symlink() {
        // TODO: Handle symlinks
    }

    Ok(())
}

#[ic_cdk::update]
pub fn scan_directory(path: String) -> String {
    let path = std::path::Path::new(&path);

    let mut paths = Vec::new();

    do_scan_dir_entry(path, &mut paths).expect("Error scanning directory!");

    paths.sort();
    paths.join("\n")
}

#[ic_cdk::update]
pub fn generate_random_fs(seed: u64, steps: u64, max_depth: u64) {
    seed_rand(seed);

    let count =
        generate_random_file_structure(steps, max_depth, std::path::Path::new(".")).unwrap();

    println!("Used {count} operations");
}

#[ic_cdk::update]
pub fn do_fs_test_basic() -> String {
    generate_random_fs(42, 100, 20);

    scan_directory(".".to_string())
}

fn generate_random_file_structure(
    mut op_count: u64,
    depth: u64,
    parent_path: &std::path::Path,
) -> anyhow::Result<u64> {
    let depth = depth - 1;

    if depth == 0 {
        return Ok(op_count);
    }

    while op_count > 0 {
        op_count -= 1;
        let action = next_rand() % 16;

        match action {
            0 => {
                // Create a new file
                let path = parent_path.join(format!("file{op_count}.txt"));
                let _ = File::create(path)?;
            }
            1 => {
                // Write to file
                let path = parent_path.join(format!("file{op_count}.txt"));
                if let Ok(mut file) = fs::OpenOptions::new()
                    .write(true)
                    .truncate(next_rand() % 2 == 1)
                    .create(true)
                    .open(&path)
                {
                    writeln!(file, "Hello world {op_count}")?;
                }
            }
            2 => {
                // Read from file
                let path = parent_path.join(format!("file{op_count}.txt"));
                if let Ok(mut file) = File::open(&path) {
                    let mut buffer = String::new();
                    let _ = file.read_to_string(&mut buffer);
                }
            }
            3 => {
                // Truncate file
                let path = parent_path.join(format!("file{op_count}.txt"));
                let _ = fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&path);
            }
            4 => {
                // Rename file
                let from = parent_path.join(format!("file{op_count}.txt"));
                let to = parent_path.join(format!("file{op_count}_renamed.txt"));
                let _ = fs::rename(&from, &to);
            }
            5 => {
                // Copy file
                let from = parent_path.join(format!("file{op_count}.txt"));
                let to = parent_path.join(format!("file{op_count}_copy.txt"));
                let _ = fs::copy(&from, &to);
            }
            6 => {
                // Delete file
                let path = parent_path.join(format!("file{op_count}.txt"));
                let _ = fs::remove_file(&path);
            }
            7 => {
                // Create directory
                let path = parent_path.join(format!("dir{op_count}"));
                let _ = fs::create_dir(&path);
            }
            8 => {
                // Remove directory
                let path = parent_path.join(format!("dir{op_count}"));
                let _ = fs::remove_dir_all(&path);
            }
            9 => {
                // List directory contents
                let _ = fs::read_dir(parent_path)?
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .collect::<Vec<_>>();
            }
            10 => {
                // Get metadata
                let path = parent_path.join(format!("file{op_count}.txt"));
                let _ = fs::metadata(&path);
            }
            11 => {
                // Get file permissions
                let path = parent_path.join(format!("file{op_count}.txt"));
                if let Ok(meta) = fs::metadata(&path) {
                    let perms = meta.permissions();
                }
            }
            12 => {
                // TODO: test symlink creation

                /*
                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    let target = parent_path.join(format!("file{op_count}.txt"));
                    let link = parent_path.join(format!("symlink{op_count}.txt"));
                    let _ = symlink(&target, &link);
                }

                 */
            }
            13 => {
                // Recursively generate inside subfolder
                let dirs: Vec<fs::DirEntry> = fs::read_dir(parent_path)?
                    .filter_map(Result::ok)
                    .filter(|e| e.path().is_dir())
                    .collect();

                if !dirs.is_empty() {
                    let rnd = next_rand() as usize % dirs.len();
                    let sub = dirs[rnd].path();
                    let res = generate_random_file_structure(op_count, depth, &sub)?;
                    op_count = res;
                }
            }
            14 => {
                // Simulate access/modification time updates (requires `filetime` crate)
                /*
                #[cfg(feature = "filetime")]
                {
                    use filetime::FileTime;
                    let path = parent_path.join(format!("file{op_count}.txt"));
                    let now = FileTime::now();
                    let _ = filetime::set_file_times(&path, now, now);
                }

                 */
            }
            15 => {
                // Move file into subdirectory
                let from = parent_path.join(format!("file{op_count}.txt"));
                let to_dir = parent_path.join(format!("dir{op_count}"));
                fs::create_dir_all(&to_dir)?;
                let to = to_dir.join(format!("file{op_count}.txt"));
                let _ = fs::rename(from, to);
            }
            _ => unreachable!(),
        }
    }

    Ok(op_count)
}

export_candid!();
