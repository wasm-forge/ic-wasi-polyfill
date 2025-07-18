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
    collections::BTreeMap,
    fs::{self, FileType, Permissions},
    io::{Seek, SeekFrom},
    path::PathBuf,
};

pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn seed(&mut self, seed: u64) {
        self.state = seed;
    }

    pub fn get_seed(&mut self) -> u64 {
        self.state
    }

    pub fn next_rand(&mut self) -> u64 {
        const A: u64 = 6364136223846793005;
        const C: u64 = 1013904223;

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

    static LOG: RefCell<Option<File>> =
        const { RefCell::new(None) };

}

pub fn next_rand(max: u64) -> u64 {
    let rnd = RND.with(|rng| rng.borrow_mut().next_rand());

    ((rnd as f64 / u64::MAX as f64) * max as f64) as u64
}

pub fn seed_rand(seed: u64) {
    RND.with(|rng| rng.borrow_mut().seed(seed));
}

pub fn get_seed() -> u64 {
    RND.with(|rng| rng.borrow_mut().get_seed())
}

pub(crate) fn new_log(path: &str) {
    let p = std::path::Path::new(path);

    let file = if p.exists() {
        fs::OpenOptions::new()
            .create(false)
            .append(true)
            .open(path)
            .unwrap()
    } else {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap()
    };

    LOG.with(|log| {
        *log.borrow_mut() = Some(file);
    });
}

fn log(msg: &str) {
    LOG.with(|log| {
        if let Some(file) = log.borrow_mut().as_mut() {
            use std::io::Write;
            writeln!(file, "{msg}").unwrap();
        } else {
            panic!("Log file not initialized. Call new_log() first.");
        }
    });
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
fn upgrade() {
    init();
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
pub fn generate_random_fs(seed: u64, steps: u64, max_depth: u64) -> u64 {
    seed_rand(seed);

    generate_random_file_structure(
        0,
        steps,
        0,
        max_depth,
        std::path::Path::new("."),
        &mut BTreeMap::new(),
    )
    .unwrap();

    get_seed()
}

#[ic_cdk::query]
pub fn get_log() -> String {
    std::fs::read_to_string("./log.txt").unwrap()
}

fn get_random_file(
    parent_path: &std::path::Path,
    op_count: u64,
    opened_files: &mut BTreeMap<String, File>,
) -> anyhow::Result<std::path::PathBuf> {
    // sometimes return an opened file
    if next_rand(2) < 1 {
        let len = opened_files.len();
        let file = opened_files.iter().nth(next_rand(len as u64) as usize);

        if let Some(f) = file {
            let path = PathBuf::new().join(f.0);

            // just recreate the file, if it doesn't exist
            let _res = fs::OpenOptions::new()
                .write(true)
                .append(false)
                .truncate(false)
                .create(true)
                .open(&path);

            return Ok(path);
        }
    }

    let mut files: Vec<fs::DirEntry> = fs::read_dir(parent_path)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file() && e.file_name().to_string_lossy() != "log.txt")
        .collect();

    files.sort_by(|a, b| {
        a.file_name()
            .to_string_lossy()
            .to_string()
            .cmp(&b.file_name().to_string_lossy().to_string())
    });

    log(&format!("get_random_file from files:{files:?}"));

    if !files.is_empty() {
        let rnd = next_rand(files.len().try_into().unwrap()) as usize;
        let file = files[rnd].path();

        return Ok(file);
    }

    // no files, create a new one for writing
    let path = parent_path.join(format!("file{op_count}.txt"));

    File::create(&path).unwrap();

    Ok(path)
}

fn get_random_dir(
    parent_path: &std::path::Path,
    op_count: u64,
) -> anyhow::Result<std::path::PathBuf> {
    let mut dirs: Vec<fs::DirEntry> = fs::read_dir(parent_path)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .collect();

    // sort folders before choosing the one to enter
    dirs.sort_by(|a, b| {
        a.file_name()
            .to_string_lossy()
            .to_string()
            .cmp(&b.file_name().to_string_lossy().to_string())
    });

    if !dirs.is_empty() {
        let rnd = next_rand(dirs.len().try_into().unwrap()) as usize;
        let path = dirs[rnd].path();

        Ok(path)
    } else {
        let path = parent_path.join(format!("dir{op_count}"));

        let res = fs::create_dir(&path);
        if let Err(_err) = res {
            log(&format!("Failed creating a folder {path:?}"));
        }

        Ok(path)
    }
}

fn rename_opened_file(opened_files: &mut BTreeMap<String, File>, from: &str, to: &str) {
    let f = opened_files.remove(from);
    if let Some(f) = f {
        opened_files.insert(to.to_string(), f);
    }
}

fn permissions_str(per: Permissions) -> String {
    format!("permissions: readonly = {}", per.readonly())
}

fn file_type_str(ftype: FileType) -> String {
    format!(
        "file_type: is_file = {}, is_dir = {}, is_symlink = {}",
        ftype.is_file(),
        ftype.is_dir(),
        ftype.is_symlink()
    )
}

fn generate_random_file_structure(
    mut current_op: u64,
    max_op_count: u64,

    depth: u64,
    max_depth: u64,

    parent_path: &std::path::Path,

    opened_files: &mut BTreeMap<String, File>,
) -> anyhow::Result<u64> {
    let depth = depth + 1;

    if depth == max_depth {
        log("Max depth reached!");
        return Ok(current_op);
    }

    while current_op < max_op_count {
        current_op += 1;
        let action = next_rand(17);

        log(&format!(
            "\n{action} \t({current_op} / {max_op_count}) \t\t\t {parent_path:?}",
        ));
        match action {
            0 => {
                // Create a new file
                let path = parent_path.join(format!("file{current_op}.txt"));
                log(&format!("Open or create file {path:?}"));
                let mut file = File::create(&path).unwrap();
                file.flush()?;
                opened_files.insert(path.as_path().to_string_lossy().to_string(), file);
            }
            1 => {
                if !opened_files.is_empty() {
                    // Close one of the opened files
                    let index = next_rand(opened_files.len() as u64);

                    let key = {
                        let (key, _file) = opened_files.iter().nth(index as usize).unwrap();
                        key.clone()
                    };

                    let _file = opened_files.remove(&key).unwrap();

                    log(&format!("Close opened file {key:?}"));
                } else {
                    current_op -= 1;
                    log("No opened files to close...");
                }
            }
            2 => {
                if !opened_files.is_empty() {
                    // Write something to one of the opened files
                    let index = next_rand(opened_files.len() as u64);

                    let (key, file) = opened_files.iter_mut().nth(index as usize).unwrap();

                    log(&format!(
                        "Write into opened file {key:?} write_into_opened{current_op}"
                    ));

                    writeln!(file, "Sequential write into opened file {key}")?;
                } else {
                    current_op -= 1;
                    log("No opened files to write...");
                }
            }
            3 => {
                // Open with options a new file
                let path = parent_path.join(format!("file{current_op}.txt"));

                let write = next_rand(2) == 1;
                let append = next_rand(2) == 1;
                let truncate = !append && write && next_rand(2) == 1;
                let create = (write || append) && next_rand(2) == 1;

                let mut res = fs::OpenOptions::new()
                    .write(write)
                    .append(append)
                    .truncate(truncate)
                    .create(create)
                    .open(&path);

                let res_str = match &mut res {
                    Ok(f) => {
                        f.flush()?;
                        format!("Ok: {:?}", &path)
                    }
                    Err(_e) => "Err".to_string(),
                };

                log(&format!("Open existing file with options: truncate={truncate},write={write},append={append},create={create} -> {res_str}"));

                if let Ok(file) = res {
                    opened_files.insert(path.to_string_lossy().to_string(), file);
                }
            }
            4 => {
                // Read text from a random file
                let file = get_random_file(parent_path, current_op, opened_files)?;

                if file.exists() {
                    //
                    let mut f = fs::OpenOptions::new().read(true).open(&file)?;

                    let mut buffer = String::new();
                    f.read_to_string(&mut buffer).unwrap();

                    log(&format!(
                        "read from file that exists: {}, content: {buffer}",
                        file.to_string_lossy()
                    ));
                } else {
                    //
                    log(&format!(
                        "read from file: {} that does not exist",
                        file.to_string_lossy()
                    ));
                }
            }
            5 => {
                // Truncate file (delete its contents)
                let file = get_random_file(parent_path, current_op, opened_files)?;

                log(&format!("Truncate {file:?}"));

                let mut f = fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(&file)?;

                f.flush()?;
            }
            6 => {
                // Rename file
                let from = get_random_file(parent_path, current_op, opened_files)?;

                let to = parent_path.join(format!("file{current_op}_renamed.txt"));
                log(&format!("Rename file from {from:?} to {to:?}"));
                let res = fs::rename(&from, &to);
                if let Err(_er) = res {
                    log(&format!("failed to rename from {from:?} to {to:?}"));
                } else {
                    // rename opened files to keep the map info updated
                    rename_opened_file(
                        opened_files,
                        from.as_path().to_string_lossy().as_ref(),
                        to.as_path().to_string_lossy().as_ref(),
                    );
                }
            }
            7 => {
                // Copy file
                let from = get_random_file(parent_path, current_op, opened_files)?;

                let to = parent_path.join(format!("file{current_op}_copy.txt"));
                log(&format!("Copy file from {from:?} to {to:?}"));
                let res = fs::copy(&from, &to);
                if let Err(_er) = res {
                    log(&format!("failed to copy from {from:?} to {to:?}"));
                }
            }
            8 => {
                // Delete file
                let path = get_random_file(parent_path, current_op, opened_files)?;

                // we do not have dangling file support yet, so do not try to delete an opened file...
                if opened_files.contains_key(&path.as_path().to_string_lossy().to_string()) {
                    log(&format!("Skip removing the opened file {path:?}"))
                } else {
                    log(&format!("Remove file {path:?}"));
                    let res = fs::remove_file(&path);
                    if let Err(_er) = res {
                        log(&format!("failed to delete file {path:?}"));
                    }
                }
            }
            9 => {
                // Create directory
                let path = parent_path.join(format!("dir{current_op}"));
                log(&format!("Create subdirectory {path:?}"));
                let res = fs::create_dir(&path);
                if let Err(_er) = res {
                    log(&format!("failed to create directory {path:?}"));
                }
            }
            10 => {
                // Remove directory
                let path = get_random_dir(parent_path, current_op)?;

                log("Closing all opened files");
                // close all opened files to avoid any problems related to dangling opened files
                opened_files.clear();

                log(&format!("Remove directory {path:?}"));
                let res = fs::remove_dir_all(&path);
                if let Err(_er) = res {
                    log(&format!("failed to remove directory {path:?}"));
                }
            }
            11 => {
                // List directory contents
                let mut dir = fs::read_dir(parent_path)?
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .collect::<Vec<_>>();

                dir.sort_by_key(|a| a.to_string_lossy().to_string());

                // store contents
                let save_path = parent_path.join(format!("file{current_op}_dir_contents.txt"));
                log(&format!("Store folder contents to {save_path:?}"));

                let mut save = fs::OpenOptions::new()
                    .create(true)
                    .truncate(false)
                    .write(true)
                    .open(&save_path)?;

                save.write_all("ls:\n".to_string().as_bytes())?;

                for x in &dir {
                    save.write_all(format!("{}\n", x.as_path().to_string_lossy()).as_bytes())?;
                }

                save.flush()?;
            }
            12 => {
                // Get metadata
                let save_path = parent_path.join(format!("file{current_op}_metadata.txt"));
                log(&format!("Store folder contents to {save_path:?}"));

                let mut save = fs::OpenOptions::new()
                    .write(true)
                    .truncate(false)
                    .create(true)
                    .open(&save_path)?;

                let path = get_random_file(parent_path, current_op, opened_files)?;

                let meta = fs::metadata(&path)?;

                save.write_all(
                    format!(
                        "{path:?}.metadata.{:?}\n",
                        permissions_str(meta.permissions())
                    )
                    .as_bytes(),
                )?;

                save.write_all(
                    format!(
                        "{path:?}.metadata.file_type = {:?}\n",
                        file_type_str(meta.file_type())
                    )
                    .as_bytes(),
                )?;

                save.flush()?;
            }
            13 => {
                // exit current directory
                if depth > 1 {
                    log("Exit current folder");

                    return Ok(current_op);
                } else {
                    log("Cannot exit, min depth reached!");
                }
            }
            14 => {
                // Recursively generate inside subfolder
                let dir = get_random_dir(parent_path, current_op)?;

                log(&format!("Enter subdirectory {dir:?}"));

                let res = generate_random_file_structure(
                    current_op,
                    max_op_count,
                    depth,
                    max_depth,
                    &dir,
                    opened_files,
                )?;

                current_op = res;
            }
            15 => {
                // Move file into subdirectory
                let from = get_random_file(parent_path, current_op, opened_files)?;
                let filename = from.file_name().unwrap().to_string_lossy().to_string();

                let dir = get_random_dir(parent_path, current_op)?;

                fs::create_dir_all(&dir)?;
                let to = dir.join(filename);

                log(&format!("Move file {from:?} to {to:?}"));
                let _ = fs::rename(from, to);
            }
            16 => {
                // write some prepared text into one of the files at a random position
                let path = get_random_file(parent_path, current_op, opened_files)?;

                let mut save = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(&path)?;

                let position = next_rand(12000);

                log(&format!("Writing into file {path:?}"));

                let text = format!("Writing at position {position}");

                save.seek(SeekFrom::Start(position))?;

                save.write_all(text.as_bytes())?;

                save.flush()?;
            }
            _ => unreachable!(),
        }
    }

    Ok(current_op)
}

#[ic_cdk::update]
pub fn do_fs_test() -> String {
    crate::tests::do_fs_test_interal()
}

export_candid!();
