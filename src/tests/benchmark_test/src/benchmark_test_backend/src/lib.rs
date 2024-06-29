use std::{cell::RefCell, fs::{self, File}, io::{BufReader, BufWriter, Read, Seek, Write}};
use ic_stable_structures::{memory_manager::{MemoryId, MemoryManager}, DefaultMemoryImpl, Memory};
use stable_fs::fs::{FdStat, OpenFlags, SrcBuf, Whence};


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
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    profiling_init();

    let wasi_memory = MEMORY_MANAGER.with(|m| m.borrow().get(WASI_MEMORY_ID));
    ic_wasi_polyfill::init_with_memory(&[0u8; 32], &[], wasi_memory);    
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
fn fs_write_kb_text(filename: String, kb_size: usize) -> u64 {
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
            .open_or_create(dir, filename.as_str(), FdStat::default(), OpenFlags::CREATE, 0)
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


#[ic_cdk::update]
fn read_kb(filename: String, kb_size: usize, offset: u64) -> Vec<u8> {
    let size = kb_size * 1024;

    let mut res = Vec::with_capacity(size);

    let f = File::open(filename).expect("Unable to open file");

    let mut f = BufReader::new(f);

    f.seek(std::io::SeekFrom::Start(offset)).unwrap();

    f.read(res.as_mut_slice()).unwrap();

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
fn list_files(path: String) -> Vec<String> {
    println!("Reading directory: {}", path);

    let mut res = vec![];
    let entries = fs::read_dir(path).unwrap();

    for entry in entries {
        let entry = entry.unwrap();

        res.push(entry.path().into_os_string().into_string().unwrap());
    }

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
