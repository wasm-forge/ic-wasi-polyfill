use std::{
    cell::RefCell,
    env,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read, Seek, Write},
};

use ic_cdk::export_candid;

use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    storable::Bound,
    DefaultMemoryImpl, StableBTreeMap,
};

use ic_stable_structures::memory_manager::VirtualMemory;

const PROFILING: MemoryId = MemoryId::new(100);
const WASI_MEMORY_ID: u8 = 1;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Greetings, {name}!")
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

}

pub fn profiling_init() {
    let memory = MEMORY_MANAGER.with(|m| m.borrow().get(PROFILING));
    ic_stable_structures::Memory::grow(&memory, 4096);
}

#[derive(Clone)]
struct MyChunk(Vec<u8>);

impl ic_stable_structures::Storable for MyChunk {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        self.0.to_bytes()
    }

    fn into_bytes(self) -> std::vec::Vec<u8> {
        self.0
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(<Vec<u8>>::from_bytes(bytes))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 150_000_000,
        is_fixed_size: false,
    };
}

#[derive(Clone)]
struct MyChunk4k(Vec<u8>);

impl ic_stable_structures::Storable for MyChunk4k {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        self.0.to_bytes()
    }

    fn into_bytes(self) -> std::vec::Vec<u8> {
        self.0
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(<Vec<u8>>::from_bytes(bytes))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 4096,
        is_fixed_size: false,
    };
}

thread_local! {
    static LARGE_CHUNK: RefCell<Option<MyChunk>> = RefCell::new(None);

    static MAP: RefCell<StableBTreeMap<u64, MyChunk, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(110))))
    );

    static MAP4K: RefCell<StableBTreeMap<(u64, u64), MyChunk4k, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(111))))
    );
}

#[ic_cdk::update]
pub fn append_chunk(text: String, times: usize) -> (u64, usize) {
    let stime = ic_cdk::api::instruction_counter();

    let res = LARGE_CHUNK.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        let total_len = text.len() * times;

        if chunk.is_none() {
            *chunk = Some(MyChunk(Vec::with_capacity(
                (text.len() + text.len() / 2) * times,
            )));
        }

        let chunk = chunk.as_mut().unwrap();

        let multiplier = 1024 * 4;

        let mut buf = String::with_capacity(text.len() * multiplier);

        for _ in 0..multiplier {
            buf.push_str(&text);
        }

        let buf = buf.as_bytes();

        let l = times / multiplier;
        for _ in 0..l {
            chunk.0.extend_from_slice(buf);
        }

        let rem_len = total_len - l * buf.len();

        chunk.0.extend_from_slice(&buf[0..rem_len]);

        chunk.0.len()
    });

    let etime = ic_cdk::api::instruction_counter();

    (etime - stime, res)
}

#[ic_cdk::update]
pub fn clear_chunk() {
    LARGE_CHUNK.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        if chunk.is_none() {
            return;
        }

        let chunk = chunk.as_mut().unwrap();

        // explicitly destroy contents
        for i in 0..chunk.0.len() {
            chunk.0[i] = 0;
        }

        chunk.0.clear()
    })
}

#[ic_cdk::update]
pub fn zero_chunk() {
    LARGE_CHUNK.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        if chunk.is_none() {
            return;
        }

        let chunk = chunk.as_mut().unwrap();

        // explicitly destroy contents
        for i in 0..chunk.0.len() {
            chunk.0[i] = 0;
        }
    })
}

#[ic_cdk::update]
pub fn read_chunk(offset: usize, size: usize) -> String {
    LARGE_CHUNK.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        let chunk = chunk.as_mut().unwrap();

        std::str::from_utf8(&chunk.0[offset..offset + size])
            .unwrap()
            .to_string()
    })
}

#[ic_cdk::update]
pub fn chunk_size() -> usize {
    LARGE_CHUNK.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        let chunk = chunk.as_mut().unwrap();

        chunk.0.len()
    })
}

#[ic_cdk::update]
pub fn store_chunk(filename: String) -> (u64, usize) {
    let stime = ic_cdk::api::instruction_counter();

    let res = LARGE_CHUNK.with(|chunk| {
        let chunk = chunk.borrow_mut();

        let chunk = &chunk.as_ref().unwrap().0;

        let mut f = File::create(filename).expect("Unable to create file");

        f.seek(std::io::SeekFrom::End(0)).unwrap();

        f.write_all(chunk).expect("Unable to write data");

        f.flush().unwrap();

        f.metadata().unwrap().len() as usize
    });

    let etime = ic_cdk::api::instruction_counter();

    (etime - stime, res)
}

#[ic_cdk::update]
pub fn store_chunk_map(key: u64) -> (u64, usize) {
    let stime = ic_cdk::api::instruction_counter();

    let res = LARGE_CHUNK.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        let chunk = chunk.take();

        let chunk = chunk.unwrap();

        let len = chunk.0.len();

        MAP.with(|mp| {
            let mut mp = mp.borrow_mut();

            mp.insert(key, chunk);
        });

        len
    });

    let etime = ic_cdk::api::instruction_counter();

    (etime - stime, res)
}

#[ic_cdk::update]
pub fn store_chunk_map4k(key: u64) -> (u64, usize) {
    let stime = ic_cdk::api::instruction_counter();

    let res = LARGE_CHUNK.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        let chunk = chunk.take();

        let chunk = chunk.unwrap();

        let mut len = 0;

        let mut idx = 0;

        MAP4K.with(|mp| {
            let mut mp = mp.borrow_mut();

            loop {
                let upper = std::cmp::min((&chunk.0).len(), ((idx + 1) * 4096) as usize);
                let lower = std::cmp::min((&chunk.0).len(), (idx * 4096) as usize);

                if lower == upper {
                    break;
                }

                let slice = &chunk.0[lower..upper];

                let mut vec: Vec<u8> = Vec::with_capacity(4096);
                vec.extend_from_slice(slice);

                len += vec.len();

                if vec.len() > 0 {
                    mp.insert((key, idx as u64), MyChunk4k(vec));
                } else {
                    break;
                }

                idx += 1;
            }
        });

        len
    });

    let etime = ic_cdk::api::instruction_counter();

    (etime - stime, res)
}

#[ic_cdk::update]
pub fn load_chunk(filename: String) -> (u64, usize) {
    let stime = ic_cdk::api::instruction_counter();

    let res = LARGE_CHUNK.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        let chunk = chunk.as_mut().unwrap();

        let mut f = File::open(filename).expect("Unable to create file");

        let _size = f.metadata().unwrap().len() as usize;

        //        chunk.0.clear();
        //       chunk.0.reserve(size);

        f.seek(std::io::SeekFrom::Start(0)).unwrap();

        let size = f.read(chunk.0.as_mut()).unwrap();

        size
    });

    let etime = ic_cdk::api::instruction_counter();

    (etime - stime, res)
}

#[ic_cdk::update]
pub fn load_chunk_map(key: u64) -> (u64, usize) {
    let stime = ic_cdk::api::instruction_counter();

    let res = LARGE_CHUNK.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        MAP.with(|mp| {
            let mp = mp.borrow_mut();

            let read = mp.get(&key).unwrap();

            *chunk = Some(read);
        });

        (*chunk).as_ref().unwrap().0.len()
    });

    let etime = ic_cdk::api::instruction_counter();

    (etime - stime, res)
}

#[ic_cdk::init]
fn init() {
    profiling_init();

    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(
            &[0u8; 32],
            &[],
            &m,
            WASI_MEMORY_ID..WASI_MEMORY_ID + 10,
        );
    });
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    profiling_init();

    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(
            &[0u8; 32],
            &[],
            &m,
            WASI_MEMORY_ID..WASI_MEMORY_ID + 10,
        );
    });
}

#[ic_cdk::update]
fn write_kb_text_buf(filename: String, kb_size: usize) -> u64 {
    let stime = ic_cdk::api::instruction_counter();

    // 64 byte block
    let text = "0123456789012345678901234567890123456789012345678901234567890123";

    let f = File::create(filename).expect("Unable to create file");
    let mut f = BufWriter::new(f);

    f.seek(std::io::SeekFrom::End(0)).unwrap();

    let times = 1024 * kb_size / text.len();

    for _ in 0..times {
        f.write_all(text.as_bytes()).expect("Unable to write data");
    }

    f.flush().unwrap();

    let etime = ic_cdk::api::instruction_counter();
    etime - stime
}

#[ic_cdk::update]
fn write_kb_text(filename: String, kb_size: usize) -> u64 {
    let stime = ic_cdk::api::instruction_counter();

    // 64 byte block
    let text = "0123456789012345678901234567890123456789012345678901234567890123";

    let mut f = File::create(filename).expect("Unable to create file");

    f.seek(std::io::SeekFrom::End(0)).unwrap();

    let times = 1024 * kb_size / text.len();

    for _ in 0..times {
        f.write_all(text.as_bytes()).expect("Unable to write data");
    }

    f.flush().unwrap();

    let etime = ic_cdk::api::instruction_counter();
    etime - stime
}

#[ic_cdk::update]
fn write_mib_text(filename: String, mib_size: usize) -> u64 {
    let stime = ic_cdk::api::instruction_counter();

    // 64 byte block
    let text = "0123456789012345678901234567890123456789012345678901234567890123";

    let f = File::create(filename).expect("Unable to create file");
    let mut f = BufWriter::new(f);

    f.seek(std::io::SeekFrom::End(0)).unwrap();

    let times = 1024 * 1024 * mib_size / text.len();

    for _ in 0..times {
        f.write_all(text.as_bytes()).expect("Unable to write data");
    }

    f.flush().unwrap();

    let etime = ic_cdk::api::instruction_counter();
    etime - stime
}

#[ic_cdk::update]
fn append_text(filename: String, text: String, times: usize) -> u64 {
    let stime = ic_cdk::api::instruction_counter();

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();

    let mut f = BufWriter::new(file);

    for _ in 0..times {
        f.write_all(text.as_bytes()).expect("Unable to write data");
    }

    f.flush().unwrap();

    let etime = ic_cdk::api::instruction_counter();
    etime - stime
}

#[ic_cdk::query]
fn read_text(filename: String, offset: i64, size: usize) -> String {
    let mut f = OpenOptions::new()
        .read(true)
        .write(false)
        .open(filename)
        .unwrap();

    f.seek(std::io::SeekFrom::Start(offset as u64)).unwrap();

    let mut content = String::with_capacity(size);

    f.take(size as u64).read_to_string(&mut content).unwrap();

    content
}

#[ic_cdk::query]
fn file_size(filename: String) -> usize {
    let f = File::open(filename).unwrap();

    let pos = f.metadata().unwrap().len();

    pos as usize
}

/*
#[ic_cdk::update]
fn fs_write_kb_text(filename: String, kb_size: usize) -> u64 {
    use stable_fs::fs::{FdStat, OpenFlags, SrcBuf, Whence};

    let stime = ic_cdk::api::instruction_counter();

    ic_wasi_polyfill::FS.with(|fs| {

        let mut fs = fs.borrow_mut();

        let dir = fs.root_fd();

        // 64 byte block
        let text = "0123456789012345678901234567890123456789012345678901234567890123";

        let write_content = [
            SrcBuf {
                buf: text.as_ptr(),
                len: text.len(),
            },
        ];

        let fd = fs
            .open(dir, filename.as_str(), FdStat::default(), OpenFlags::CREATE, 0)
            .unwrap();

        let _ = fs.seek(fd, 0, Whence::END);

        let times = 1024 * kb_size / text.len();

        for _ in 0..times {
            fs.write_vec(fd, write_content.as_ref()).unwrap();
        }

        let _ = fs.close(fd);

    });

    let etime = ic_cdk::api::instruction_counter();

    etime - stime
}
*/

#[ic_cdk::update]
fn read_kb(filename: String, kb_size: usize, offset: u64) -> Vec<u8> {
    let size = kb_size * 1024;

    let mut res = Vec::with_capacity(size);

    let f = File::open(filename).expect("Unable to open file");

    let mut f = std::io::BufReader::new(f);

    f.seek(std::io::SeekFrom::Start(offset)).unwrap();

    let _ = f.read(res.as_mut_slice()).unwrap();

    res
}

// delete file
#[ic_cdk::query]
fn delete_file(filename: String) {
    fs::remove_file(filename).unwrap();
}

// delete folder
#[ic_cdk::query]
fn delete_folder(path: String) {
    fs::remove_dir_all(path).unwrap();
}

#[ic_cdk::query]
fn current_dir() -> String {
    let res = env::current_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();

    res
}

#[ic_cdk::query]
fn set_current_dir(path: String) {
    env::set_current_dir(path).unwrap();
}

#[ic_cdk::query]
fn list_files(path: String) -> Vec<String> {
    let mut res = vec![];
    let entries = fs::read_dir(path).unwrap();

    for entry in entries {
        let entry = entry.unwrap();

        let last = entry.file_name().into_string().unwrap();

        res.push(last);
    }

    res
}

fn list_all_files_recursive(path: &String, files: &mut Vec<String>) {
    let entries = fs::read_dir(&path).unwrap();

    for entry in entries {
        let entry = entry.unwrap();

        let folder_name = entry.path().into_os_string().into_string().unwrap();

        println!("{}", &folder_name);
        files.push(folder_name.clone());

        if entry.metadata().unwrap().is_dir() {
            list_all_files_recursive(&folder_name, files);
        }
    }
}

#[ic_cdk::query]
fn list_all_files(path: String) -> Vec<String> {
    println!("Reading directory: {}", path);

    let mut res = vec![];
    list_all_files_recursive(&path, &mut res);

    res
}

#[ic_cdk::update]
fn create_depth_folders(path: String, count: usize) -> String {
    let mut dir_name = "d0".to_string();

    for num in 1..count {
        dir_name = format!("{}/d{}", dir_name, num);
    }

    dir_name = format!("{}/{}", path, dir_name);

    fs::create_dir_all(&dir_name).unwrap();

    dir_name
}

#[ic_cdk::update]
fn delete_depth_folders(path: String, count: usize) -> String {
    let mut dir_name = "d0".to_string();

    for num in 1..count {
        dir_name = format!("{}/d{}", dir_name, num);
    }

    dir_name = format!("{}/{}", path, dir_name);

    fs::remove_dir_all(&dir_name).unwrap();

    dir_name
}

#[ic_cdk::update]
fn create_files(path: String, count: usize) -> u64 {
    let stime = ic_cdk::api::instruction_counter();

    for num in 0..count {
        let filename = format!("{}/{}.txt", path, num);
        let mut file = File::create(&filename).unwrap();

        // 64 byte block + file name
        let text = format!(
            "0123456789012345678901234567890123456789012345678901234567890123:{}",
            filename
        );

        file.write_all(text.as_bytes()).unwrap();
        file.flush().unwrap();
    }

    let etime = ic_cdk::api::instruction_counter();

    etime - stime
}

#[ic_cdk::update]
fn check_new_dir_is_writable(dirname: String) -> String {
    std::fs::create_dir(&dirname).unwrap();

    let md = fs::metadata(&dirname).unwrap();

    let permissions = md.permissions();
    let readonly = permissions.readonly();

    if readonly {
        "Is readonly".to_string()
    } else {
        "Is writable".to_string()
    }
}

#[ic_cdk::update]
fn check_dir_is_writable(dirname: String) -> String {
    let md = fs::metadata(&dirname).unwrap();

    let permissions = md.permissions();
    let readonly = permissions.readonly();

    if readonly {
        "Is readonly".to_string()
    } else {
        "Is writable".to_string()
    }
}

#[ic_cdk::update]
fn check_new_file_is_writable(file: String) -> String {
    std::fs::File::create(&file).unwrap();

    let md = fs::metadata(&file).unwrap();

    let permissions = md.permissions();
    let readonly = permissions.readonly();

    if readonly {
        "Is readonly".to_string()
    } else {
        "Is writable".to_string()
    }
}

export_candid!();
