use std::cell::RefCell;

use ic_stable_structures::DefaultMemoryImpl;
use rand::{RngCore, SeedableRng};
use stable_fs::fs::{DstBuf, Fd, SrcBuf};
use stable_fs::fs::{FdFlags, FdStat, FileSystem, OpenFlags};
use stable_fs::storage::stable::StableStorage;

use stable_fs::storage::types::FileSize;
use wasi_helpers::*;

mod wasi;
mod wasi_helpers;

thread_local! {
    static RNG : RefCell<Option<rand::rngs::StdRng>> = RefCell::new(None);
    static FS: RefCell<FileSystem> = RefCell::new(
        FileSystem::new(Box::new(StableStorage::new(DefaultMemoryImpl::default()))).unwrap()
    );
}

macro_rules! debug_println {
//    ($fmt:expr) => (ic_cdk::api::print(format!($fmt)));
//    ($fmt:expr, $($arg:tt)*) => (ic_cdk::api::print(format!($fmt, $($arg)*)));
    ($fmt:expr) => {};
    ($fmt:expr, $($arg:tt)*) => {};
}

macro_rules! debug_instructions {
    ($fn_name:literal, $stime:expr) => (let etime=ic_cdk::api::instruction_counter();ic_cdk::api::print(format!("{} instructions: \t{}", $fn_name, etime-($stime))));
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_write(
    fd: i32,
    iovs: *const wasi::Ciovec,
    len: i32,
    res: *mut wasi::Size,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_write fd={fd:?} len={len:?}");

    let result = if fd < 3 {
        wasi_helpers::forward_to_debug(iovs, len, res)
    } else {
        FS.with(|fs| {
            let mut fs = fs.borrow_mut();
            let src_io_vec = iovs as *const SrcBuf;
            let src_io_vec = std::slice::from_raw_parts(src_io_vec, len as wasi::Size);

            match fs.write_vec(fd as Fd, src_io_vec) {
                Ok(r) => {
                    *res = r as wasi::Size;
                    wasi::ERRNO_SUCCESS.raw() as i32
                }
                Err(er) => {
                    *res = 0;
                    into_errno(er)
                }
            }
        })
    };

    debug_instructions!("__ic_custom_fd_write", start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_read(
    fd: i32,
    iovs: *const wasi::Ciovec,
    len: i32,
    res: *mut wasi::Size,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_read fd={fd:?} len={len:?}");

    // for now we don't support reading from the standard streams
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();
        let dst_io_vec = iovs as *const DstBuf;

        unsafe {
            let dst_io_vec = std::slice::from_raw_parts(dst_io_vec, len as wasi::Size);

            match fs.read_vec(fd as Fd, dst_io_vec) {
                Ok(r) => {
                    *res = r as wasi::Size;
                    wasi::ERRNO_SUCCESS.raw() as i32
                }
                Err(er) => {
                    *res = 0;
                    into_errno(er)
                }
            }
        }
    });


    debug_instructions!("__ic_custom_fd_read", start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_pwrite(
    fd: i32,
    iovs: *const wasi::Ciovec,
    len: i32,
    offset: i64,
    res: *mut wasi::Size,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_pwrite fd={fd:?} len={len:?}");

    let result = if fd < 3 {
        wasi_helpers::forward_to_debug(iovs, len, res)
    } else {
        FS.with(|fs| {
            let mut fs = fs.borrow_mut();
            let src_io_vec = iovs as *const SrcBuf;
            let src_io_vec = std::slice::from_raw_parts(src_io_vec, len as wasi::Size);
            match fs.write_vec_with_offset(fd as Fd, src_io_vec, offset as FileSize) {
                Ok(r) => {
                    *res = r as wasi::Size;

                    wasi::ERRNO_SUCCESS.raw() as i32
                }
                Err(er) => {
                    *res = 0;
                    into_errno(er)
                }
            }
        })
    };

    debug_instructions!("__ic_custom_fd_pwrite", start);
    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_pread(
    fd: i32,
    iovs: *const wasi::Ciovec,
    len: i32,
    offset: i64,
    res: *mut wasi::Size,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_pread fd={fd:?} len={len:?}");

    // for now we don't support reading from the standard streams
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();
        let dst_io_vec = iovs as *const DstBuf;

        unsafe {
            let dst_io_vec = std::slice::from_raw_parts(dst_io_vec, len as wasi::Size);

            match fs.read_vec_with_offset(fd as Fd, dst_io_vec, offset as FileSize) {
                Ok(r) => {
                    *res = r as wasi::Size;
                    wasi::ERRNO_SUCCESS.raw() as i32
                }
                Err(er) => {
                    *res = 0;
                    into_errno(er)
                }
            }
        }
    });

    debug_instructions!("__ic_custom_fd_pread", start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_seek(
    fd: i32,
    delta: i64,
    whence: i32,
    res: *mut wasi::Filesize,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_seek fd={fd:?} delta={delta:?} whence={whence:?}");

    // standart streams not supported
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        unsafe {
            match fs.seek(
                fd as Fd,
                delta,
                wasi_helpers::into_stable_fs_wence(whence as u8),
            ) {
                Ok(r) => {
                    *res = r as wasi::Filesize;
                    
                    wasi::ERRNO_SUCCESS.raw() as i32
                }
                Err(er) => {
                    *res = 0;
                    into_errno(er)
                }
            }
        }
    });


    debug_instructions!("__ic_custom_fd_seek", start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_path_open(
    parent_fd: i32,
    dirflags: i32,
    path: *const u8,
    path_len: i32,

    oflags: i32,
    fs_rights_base: i64,
    fs_rights_inheriting: i64,

    fdflags: i32,
    res: *mut u32,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    // dirflags contains the information on whether to follow the symlinks,
    // the symlinks are not supported yet by the file system
    prevent_elimination(&[dirflags]);


    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let file_name = get_file_name(path, path_len as wasi::Size);

        debug_println!(
            "called __ic_custom_path_open parent_fd={parent_fd:?} file_name={file_name:?}"
        );

        let fd_stat = FdStat {
            flags: FdFlags::from_bits_truncate(fdflags as u16),
            rights_base: fs_rights_base as u64,
            rights_inheriting: fs_rights_inheriting as u64,
        };

        let open_flags = OpenFlags::from_bits_truncate(oflags as u16);

        let now = ic_cdk::api::time();

        let r = fs.open_or_create(parent_fd as Fd, file_name, fd_stat, open_flags, now);

        match r {
            Ok(r) => {
                *res = r;
                wasi::ERRNO_SUCCESS.raw() as i32
            }
            Err(er) => {
                *res = 0;
                into_errno(er)
            }
        }
    });

    debug_instructions!("__ic_custom_path_open", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_close(fd: i32) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_close fd={fd:?}");

    let result = FS.with(|fs| {
        let res = fs.borrow_mut().close(fd as Fd);
        
        match res {
            Ok(_) => wasi::ERRNO_SUCCESS.raw() as i32,
            Err(er) => into_errno(er),
        }
    });

    debug_instructions!("__ic_custom_fd_close", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_filestat_get(fd: i32, ret_val: *mut u8) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_filestat_get fd={fd:?}");

    let result = FS.with(|fs| {
        let fs = fs.borrow();
        let res = fs.metadata(fd as u32);

        match res {
            Ok(metadata) => {
                let value: wasi::Filestat = wasi::Filestat {
                    dev: 0,
                    ino: metadata.node,
                    filetype: into_wasi_filetype(metadata.file_type),
                    nlink: metadata.link_count,
                    size: metadata.size,
                    atim: metadata.times.accessed,
                    mtim: metadata.times.modified,
                    ctim: metadata.times.created,
                };

                unsafe {
                    let ret_val = ret_val as *mut wasi::Filestat;
                    *ret_val = value;
                }

                wasi::ERRNO_SUCCESS.raw() as i32
            }
            Err(er) => into_errno(er),
        }
    });

    debug_instructions!("__ic_custom_fd_filestat_get", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_sync(fd: i32) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    prevent_elimination(&[fd]);

    debug_println!("called __ic_custom_fd_sync fd={fd:?}");

    debug_instructions!("__ic_custom_fd_sync", start);
    wasi::ERRNO_SUCCESS.raw() as i32
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_tell(fd: i32, res: *mut wasi::Filesize) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_tell {fd:?}");

    // standard streams not supported
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        unsafe {
            match fs.tell(fd as Fd) {
                Ok(pos) => {
                    *res = pos as wasi::Filesize;
                    
                    wasi::ERRNO_SUCCESS.raw() as i32
                }
                Err(er) => {
                    *res = 0;
                    into_errno(er)
                }
            }
        }
    });

    debug_instructions!("__ic_custom_fd_tell", start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_prestat_get(fd: i32, res: *mut wasi::Prestat) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_prestat_get fd={fd:?}");

    let result = FS.with(|fs| {
        let fs = fs.borrow();

        if fd as Fd == fs.root_fd() {
            let root_len = fs.root_path().len();

            let prestat = wasi::Prestat {
                tag: 0,
                u: wasi::PrestatU {
                    dir: wasi::PrestatDir {
                        pr_name_len: root_len,
                    },
                },
            };

            unsafe {
                *res = prestat;
            }

            wasi::ERRNO_SUCCESS.raw() as i32
        } else {
            wasi::ERRNO_BADF.raw() as i32
        }
    });

    debug_instructions!("__ic_custom_fd_prestat_get", start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_prestat_dir_name(
    fd: i32,
    path: *mut u8,
    max_len: i32,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_prestat_dir_name fd={fd:?}");
    let max_len = max_len as wasi::Size;

    let result = FS.with(|fs| {
        let fs = fs.borrow();

        if fd as Fd == fs.root_fd() {
            let max_len = std::cmp::max(max_len as i32, fs.root_path().len() as i32) as usize;
            for i in 0..max_len {
                unsafe {
                    path.add(i).write(fs.root_path().as_bytes()[i]);
                }
            }

            wasi::ERRNO_SUCCESS.raw() as i32
        } else {
            wasi::ERRNO_BADF.raw() as i32
        }
    });

    debug_instructions!("__ic_custom_fd_prestat_dir_name", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_advise(fd: i32, offset: i64, len: i64, advice: i32) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    prevent_elimination(&[offset as i32, len as i32]);

    debug_println!(
        "called __ic_custom_fd_advise fd={fd:?} offset={offset:?} len={len:?} advice={advice:?}"
    );

    if advice as u32 > 5 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    let mut is_badf = false;

    FS.with(|fs| {
        let fs = fs.borrow();

        // check fd is real
        if fs.metadata(fd as Fd).is_err() {
            is_badf = true;
        }
    });

    debug_instructions!("__ic_custom_fd_advise", start);

    if is_badf {
        return wasi::ERRNO_BADF.raw() as i32;
    }
    
    wasi::ERRNO_SUCCESS.raw() as i32
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_allocate(fd: i32, offset: i64, len: i64) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    prevent_elimination(&[offset as i32, len as i32]);
    debug_println!("called __ic_custom_fd_allocate");

    let mut result = wasi::ERRNO_SUCCESS.raw() as i32;

    FS.with(|fs| {
        let fs = fs.borrow();

        // check fd is real, for now don't do any allocation
        if fs.metadata(fd as Fd).is_err() {
            result = wasi::ERRNO_BADF.raw() as i32;
        };
    });

    debug_instructions!("__ic_custom_fd_allocate", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_datasync(fd: i32) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_datasync fd={fd:?}");

    let mut result = wasi::ERRNO_SUCCESS.raw() as i32;

    FS.with(|fs| {
        let fs = fs.borrow();

        // check if the file descriptor is correct
        if fs.metadata(fd as Fd).is_err() {
            result = wasi::ERRNO_BADF.raw() as i32;
        };

        // we don't do the synchronization for now
    });

    debug_instructions!("__ic_custom_fd_datasync", start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_fdstat_get(fd: i32, ret_fdstat: *mut wasi::Fdstat) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_fdstat_get fd={fd:?}");

    let result = FS.with(|fs| {
        let fs = fs.borrow();

        let stat = fs.get_stat(fd as Fd);

        match stat {
            Ok((ftype, fdstat)) => {
                let tmp_fd_stat = wasi::Fdstat {
                    fs_filetype: into_wasi_filetype(ftype),
                    fs_flags: fdstat.flags.bits(),
                    fs_rights_base: fdstat.rights_base,
                    fs_rights_inheriting: fdstat.rights_inheriting,
                };

                unsafe {
                    *ret_fdstat = tmp_fd_stat;
                }

                wasi::ERRNO_SUCCESS.raw() as i32
            }
            Err(err) => wasi_helpers::into_errno(err),
        }
    });

    debug_instructions!("__ic_custom_fd_fdstat_get", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_fdstat_set_flags(fd: i32, new_flags: i32) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_fdstat_set_flags fd={fd:?} new_flags={new_flags:?}");

    let result = FS.with(|fs| {
        let new_flags = new_flags as wasi::Fdflags;

        let mut fs = fs.borrow_mut();

        let stat = fs.get_stat(fd as Fd);

        match stat {
            Ok((_ftype, mut fdstat)) => {
                let new_flags = FdFlags::from_bits(new_flags);

                if new_flags.is_none() {
                    return wasi::ERRNO_INVAL.raw() as i32;
                }

                fdstat.flags = new_flags.unwrap();

                match fs.set_stat(fd as Fd, fdstat) {
                    Ok(_) => wasi::ERRNO_SUCCESS.raw() as i32,
                    Err(err) => wasi_helpers::into_errno(err),
                }
            }
            Err(err) => wasi_helpers::into_errno(err),
        }
    });

    debug_instructions!("__ic_custom_fd_fdstat_set_flags", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_fdstat_set_rights(
    fd: i32,
    rights_base: i64,
    rights_inheriting: i64,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_fdstat_set_rights fd={fd:?} rights_base={rights_base:?} rights_inheriting={rights_inheriting:?}");

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let stat = fs.get_stat(fd as Fd);

        match stat {
            Ok((_ftype, mut fdstat)) => {
                fdstat.rights_base = rights_base as u64;
                fdstat.rights_inheriting = rights_inheriting as u64;

                match fs.set_stat(fd as Fd, fdstat) {
                    Ok(_) => wasi::ERRNO_SUCCESS.raw() as i32,
                    Err(err) => wasi_helpers::into_errno(err),
                }
            }
            Err(err) => wasi_helpers::into_errno(err),
        }
    });

    debug_instructions!("__ic_custom_fd_fdstat_set_rights", start);

    result
}

#[no_mangle]
pub extern "C" fn __ic_custom_fd_filestat_set_size(fd: i32, size: i64) -> i32 {
    let start = ic_cdk::api::instruction_counter();

    prevent_elimination(&[size as i32]);
    debug_println!("called __ic_custom_fd_filestat_set_size fd={fd:?} size={size:?}");

    let result = FS.with(|fs| {
        let fs = fs.borrow_mut();

        let stat = fs.get_stat(fd as Fd);

        match stat {
            Ok((_ftype, _fdstat)) => {
                // set_size is not supported yet

                wasi::ERRNO_SUCCESS.raw() as i32
            }
            Err(err) => wasi_helpers::into_errno(err),
        }
    });

    debug_instructions!("__ic_custom_fd_filestat_set_size", start);

    result
}

#[no_mangle]
pub extern "C" fn __ic_custom_fd_filestat_set_times(
    fd: i32,
    atim: i64,
    mtim: i64,
    fst_flags: i32,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_filestat_set_times fd={fd:?} atim={atim:?} mtim={mtim:?} fst_flags={fst_flags:?}");

    let fst_flags = fst_flags as wasi::Fstflags;

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();
        let mut atim = atim as u64;
        let mut mtim = mtim as u64;

        let meta = fs.metadata(fd as u32);

        match meta {
            Ok(_) => {
                let now = ic_cdk::api::time();

                if fst_flags & wasi::FSTFLAGS_ATIM_NOW > 0 {
                    atim = now;
                }

                if fst_flags & wasi::FSTFLAGS_MTIM_NOW > 0 {
                    mtim = now;
                }

                if fst_flags & wasi::FSTFLAGS_ATIM > 0 {
                    let _ = fs.set_accessed_time(fd as Fd, atim as u64);
                }

                if fst_flags & wasi::FSTFLAGS_MTIM > 0 {
                    let _ = fs.set_modified_time(fd as Fd, mtim as u64);
                }

                wasi::ERRNO_SUCCESS.raw() as i32
            }
            Err(err) => wasi_helpers::into_errno(err),
        }
    });

    debug_instructions!("__ic_custom_fd_filestat_set_times", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_readdir(
    fd: i32,
    bytes: *mut u8,
    bytes_len: i32,
    cookie: i64,
    res: *mut wasi::Size,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_readdir fd={fd:?}");

    let result = FS.with(|fs| {
        let fs = fs.borrow();
        wasi_helpers::fd_readdir(&fs, fd, cookie, bytes, bytes_len, res)
    });

    debug_instructions!("__ic_custom_fd_readdir", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_renumber(fd_from: i32, fd_to: i32) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_fd_renumber fd_from={fd_from:?} fd_to={fd_to:?}");

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let result = fs.renumber(fd_from as Fd, fd_to as Fd);

        match result {
            Ok(()) => wasi::ERRNO_SUCCESS.raw() as i32,
            Err(err) => into_errno(err),
        }
    });

    debug_instructions!("__ic_custom_fd_renumber", start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_random_get(buf: *mut u8, buf_len: wasi::Size) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_random_get");

    let buf = std::slice::from_raw_parts_mut(buf, buf_len);
    RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        let rng = rng.as_mut().unwrap();
        rng.fill_bytes(buf);
    });

    debug_instructions!("__ic_custom_random_get", start);

    wasi::ERRNO_SUCCESS.raw() as i32
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_environ_get(arg0: i32, arg1: i32) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_environ_get");
    prevent_elimination(&[arg0, arg1]);

    debug_instructions!("__ic_custom_environ_get", start);
    // No-op.
    0
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_environ_sizes_get(
    len1: *mut wasi::Size,
    len2: *mut wasi::Size,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    debug_println!("called __ic_custom_environ_sizes_get");
    *len1 = 0;
    *len2 = 0;
    debug_instructions!("__ic_custom_environ_sizes_get", start);
    0
}

#[no_mangle]
pub extern "C" fn __ic_custom_proc_exit(code: i32) -> ! {
    panic!("WASI proc_exit called with code: {code}");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_args_get(arg0: i32, arg1: i32) -> i32 {
    debug_println!("called __ic_custom_args_get");
    prevent_elimination(&[arg0, arg1]);
    // No-op.
    0
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_args_sizes_get(
    len1: *mut wasi::Size,
    len2: *mut wasi::Size,
) -> i32 {
    debug_println!("called __ic_custom_arg_sizes_get");
    *len1 = 0;
    *len2 = 0;
    0
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_clock_res_get(id: i32, result: *mut u64) -> i32 {
    prevent_elimination(&[id]);
    debug_println!("called __ic_custom_clock_res_get");

    *result = 1_000_000_000; // 1 second.
    wasi::ERRNO_SUCCESS.raw() as i32
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_clock_time_get(
    id: i32,
    precision: i64,
    result: *mut u64,
) -> i32 {
    prevent_elimination(&[id, precision as i32]);
    debug_println!("called __ic_custom_clock_time_get");
    *result = ic_cdk::api::time();
    wasi::ERRNO_SUCCESS.raw() as i32
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_path_create_directory(
    parent_fd: i32,
    path: *const u8,
    path_len: i32,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let dir_name = get_file_name(path, path_len as wasi::Size);


        debug_println!("called __ic_custom_path_create_directory parent_fd={parent_fd:?} dir_name={dir_name:?}");

        let fd_stat = FdStat {
            flags: FdFlags::from_bits_truncate(0),
            rights_base: 0,
            rights_inheriting: 0,
        };

        let now = ic_cdk::api::time();

        let fd = fs.create_dir(parent_fd as Fd, dir_name, fd_stat, now);

        match fd {
            Ok(fd) => {
                let _ = fs.close(fd);
                wasi::ERRNO_SUCCESS.raw() as i32
            }
            Err(er) => into_errno(er),
        }
    });

    debug_instructions!("__ic_custom_path_create_directory", start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_path_filestat_get(
    parent_fd: i32,
    flags: i32,
    path: *const u8,
    path_len: i32,
    result: *mut wasi::Filestat,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();

    prevent_elimination(&[flags]);
    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let file_name = get_file_name(path, path_len as wasi::Size);

        debug_println!(
            "called __ic_custom_path_filestat_get parent_fd={parent_fd:?} file_name={file_name:?}"
        );

        let fd_stat = FdStat {
            flags: FdFlags::from_bits_truncate(0),
            rights_base: 0,
            rights_inheriting: 0,
        };

        let open_flags = OpenFlags::empty();

        let fd = fs.open_or_create(parent_fd as Fd, file_name, fd_stat, open_flags, 0);

        match fd {
            Ok(fd) => {
                let res = fs.metadata(fd);

                match res {
                    Ok(metadata) => {
                        *result = wasi::Filestat {
                            dev: 0,
                            ino: metadata.node,
                            filetype: into_wasi_filetype(metadata.file_type),
                            nlink: metadata.link_count,
                            size: metadata.size,
                            atim: metadata.times.accessed,
                            mtim: metadata.times.modified,
                            ctim: metadata.times.created,
                        };
                        let _ = fs.close(fd);
                        wasi::ERRNO_SUCCESS.raw() as i32
                    }
                    Err(er) => {
                        let _ = fs.close(fd);
                        into_errno(er)
                    }
                }
            }
            Err(er) => into_errno(er),
        }
    });

    debug_instructions!("__ic_custom_path_filestat_get", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_filestat_set_times(
    parent_fd: i32,
    flags: i32,
    path: *const u8,
    path_len: i32,
    atim: i64,
    mtim: i64,
    fst_flags: i32,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    prevent_elimination(&[flags]);

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let file_name = get_file_name(path, path_len as wasi::Size);

        debug_println!("called __ic_custom_path_filestat_set_times parent_fd={parent_fd:?} file_name={file_name:?} atim={atim:?} mtim={mtim:?}");

        let fd_stat = FdStat {
            flags: FdFlags::from_bits_truncate(0),
            rights_base: 0,
            rights_inheriting: 0,
        };

        let fst_flags = fst_flags as wasi::Fstflags;
        let mut atim = atim as u64;
        let mut mtim = mtim as u64;

        let open_flags = OpenFlags::empty();

        let fd = fs.open_or_create(parent_fd as Fd, file_name, fd_stat, open_flags, 0);

        match fd {
            Ok(fd) => {

                let res = fs.metadata(fd);

                match res {
                    Ok(mut metadata) => {

                        let now = ic_cdk::api::time();

                        if fst_flags & wasi::FSTFLAGS_ATIM_NOW > 0 {
                            atim = now;
                        }

                        if fst_flags & wasi::FSTFLAGS_MTIM_NOW > 0 {
                            mtim = now;
                        }

                        if fst_flags & wasi::FSTFLAGS_ATIM > 0 {
                            metadata.times.accessed = atim;
                        }

                        if fst_flags & wasi::FSTFLAGS_MTIM > 0 {
                            metadata.times.modified = mtim;
                        }

                        let res = fs.set_metadata(fd, metadata);

                        let _ = fs.close(fd);

                        match res {
                            Ok(_) => {
                                wasi::ERRNO_SUCCESS.raw() as i32
                            }
                            Err(er) => {
                                into_errno(er)
                            }
                        }
                    }
                    Err(er) => {
                        let _ = fs.close(fd);
                        into_errno(er)
                    }
                }

            }
            Err(er) => into_errno(er),
        }
    });

    debug_instructions!("__ic_custom_path_filestat_set_times", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_link(
    old_fd: i32,
    old_flags: i32,
    old_path: *const u8,
    old_path_len: i32,
    new_fd: i32,
    new_path: *const u8,
    new_path_len: i32,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();
    prevent_elimination(&[old_flags]);

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let old_path = get_file_name(old_path, old_path_len as wasi::Size);
        let new_path = get_file_name(new_path, new_path_len as wasi::Size);

        debug_println!("called __ic_custom_path_link old_parent_fd={old_fd:?} old_path={old_path:?} <- new_parent_fd={new_fd:?} new_path={new_path:?}");

        let fd = fs.create_hard_link(old_fd as Fd, old_path, new_fd as Fd, new_path);

        match fd {
            Ok(fd) => {
                let _ = fs.close(fd);
                wasi::ERRNO_SUCCESS.raw() as i32
            }
            Err(er) => into_errno(er),
        }
    });

    debug_instructions!("__ic_custom_path_link", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_readlink(
    fd: i32,
    path: *const u8,
    path_len: i32,
    buf: i32,
    buf_len: i32,
    rp0: i32,
) -> i32 {
    prevent_elimination(&[fd, path as i32, path_len, buf, buf_len, rp0]);
    unimplemented!("WASI path_readlink is not implemented");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_remove_directory(
    parent_fd: i32,
    path: *const u8,
    path_len: i32,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let file_name = get_file_name(path, path_len as wasi::Size);

        debug_println!(
            "called __ic_custom_path_remove_directory file parent_fd={parent_fd:?} file_name={file_name:?}"
        );

        let res = fs.remove_dir(parent_fd as Fd, file_name);
        match res {
            Ok(()) => wasi::ERRNO_SUCCESS.raw() as i32,
            Err(er) => into_errno(er),
        }
    });

    debug_instructions!("__ic_custom_path_remove_directory", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_rename(
    old_fd: i32,
    old_path: *const u8,
    old_path_len: i32,
    new_fd: i32,
    new_path: *const u8,
    new_path_len: i32,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();


    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let old_path = get_file_name(old_path, old_path_len as wasi::Size);
        let new_path = get_file_name(new_path, new_path_len as wasi::Size);

        debug_println!("called __ic_custom_path_rename old_parent_fd={old_fd:?} old_path={old_path:?} -> new_parent_fd={new_fd:?} new_path={new_path:?}");

        let fd = fs.rename(old_fd as Fd, old_path, new_fd as Fd, new_path);

        match fd {
            Ok(fd) => {
                let _ = fs.close(fd);
                wasi::ERRNO_SUCCESS.raw() as i32
            }
            Err(er) => into_errno(er),
        }
    });

    debug_instructions!("__ic_custom_path_rename", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_symlink(
    old_path: i32,
    old_path_len: i32,
    fd: i32,
    new_path: i32,
    new_path_len: i32,
) -> i32 {
    prevent_elimination(&[old_path, old_path_len, fd, new_path, new_path_len]);
    unimplemented!("WASI path_symlink is not implemented");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_unlink_file(
    parent_fd: i32,
    path: *const u8,
    path_len: i32,
) -> i32 {
    let start = ic_cdk::api::instruction_counter();

    let result = FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let file_name = get_file_name(path, path_len as wasi::Size);

        debug_println!(
            "called __ic_custom_path_unlink file parent_fd={parent_fd:?} file_name={file_name:?}"
        );

        let res = fs.remove_file(parent_fd as Fd, file_name);
        match res {
            Ok(()) => wasi::ERRNO_SUCCESS.raw() as i32,
            Err(er) => into_errno(er),
        }
    });

    debug_instructions!("__ic_custom_path_unlink", start);

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_poll_oneoff(
    in_: *const wasi::Subscription,
    out: *mut wasi::Event,
    nsubscriptions: i32,
    rp0: i32,
) -> i32 {
    prevent_elimination(&[in_ as i32, out as i32, nsubscriptions, rp0]);
    unimplemented!("WASI poll_oneoff is not implemented");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_proc_raise(sig: i32) -> i32 {
    prevent_elimination(&[sig]);
    unimplemented!("WASI proc_raise is not implemented");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sched_yield() -> i32 {
    // No-op.
    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sock_accept(arg0: i32, arg1: i32, arg2: i32) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2]);
    unimplemented!("WASI sock_accept is not supported");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sock_recv(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    arg5: i32,
) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2, arg3, arg4, arg5]);
    unimplemented!("WASI sock_recv is not supported");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sock_send(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2, arg3, arg4]);
    unimplemented!("WASI sock_send is not supported");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sock_shutdown(arg0: i32, arg1: i32) -> i32 {
    prevent_elimination(&[arg0, arg1]);
    unimplemented!("WASI sock_shutdown is not supported");
}

thread_local! {
    static COUNTER: RefCell<i32> = RefCell::new(0);
}

fn prevent_elimination(args: &[i32]) {
    COUNTER.with(|var| {
        if *var.borrow() == -1 {
            ic_cdk::api::print(format!("args: {args:?}"));
        }
    });
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe fn init_seed(seed: &[u8]) {
    raw_init_seed(seed.as_ptr(), seed.len());
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn raw_init_seed(seed: *const u8, len: usize) {
    if seed.is_null() || len == 0 {
        return;
    }

    let len = usize::min(len, 32);

    let mut seed_buf: [u8; 32] = [0u8; 32];
    unsafe { std::ptr::copy_nonoverlapping(seed, seed_buf.as_mut_ptr(), len) }

    RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        *rng = Some(rand::rngs::StdRng::from_seed(seed_buf));
    });
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn raw_init(seed: *const u8, len: usize) {
    raw_init_seed(seed, len);

    COUNTER.with(|var| {
        if *var.borrow() == -1 {
            // dummy calls to trick the linker not to throw away the functions
            unsafe {
                use std::ptr::{null, null_mut};
                __ic_custom_fd_write(0, null::<wasi::Ciovec>(), 0, null_mut::<wasi::Size>());
                __ic_custom_fd_read(0, null::<wasi::Ciovec>(), 0, null_mut::<wasi::Size>());
                __ic_custom_fd_close(0);

                __ic_custom_fd_prestat_get(0, null_mut::<wasi::Prestat>());
                __ic_custom_fd_prestat_dir_name(0, null_mut::<u8>(), 0);

                __ic_custom_path_open(0, 0, null::<u8>(), 0, 0, 0, 0, 0, null_mut::<u32>());
                __ic_custom_random_get(null_mut::<u8>(), 0);

                __ic_custom_environ_get(0, 0);
                __ic_custom_environ_sizes_get(null_mut::<wasi::Size>(), null_mut::<wasi::Size>());

                __ic_custom_args_get(0, 0);
                __ic_custom_args_sizes_get(null_mut::<wasi::Size>(), null_mut::<wasi::Size>());
                __ic_custom_clock_res_get(0, null_mut::<u64>());
                __ic_custom_clock_time_get(0, 0, null_mut::<u64>());
                __ic_custom_fd_advise(0, 0, 0, 0);
                __ic_custom_fd_allocate(0, 0, 0);
                __ic_custom_fd_datasync(0);
                __ic_custom_fd_fdstat_get(0, null_mut::<wasi::Fdstat>());
                __ic_custom_fd_fdstat_set_flags(0, 0);
                __ic_custom_fd_fdstat_set_rights(0, 0, 0);
                __ic_custom_fd_filestat_get(0, null_mut::<u8>());
                __ic_custom_fd_filestat_set_size(0, 0);
                __ic_custom_fd_filestat_set_times(0, 0, 0, 0);
                __ic_custom_fd_pread(0, null::<wasi::Ciovec>(), 0, 0, null_mut::<wasi::Size>());
                __ic_custom_fd_pwrite(0, null::<wasi::Ciovec>(), 0, 0, null_mut::<wasi::Size>());
                __ic_custom_fd_readdir(0, null_mut::<u8>(), 0, 0, null_mut::<wasi::Size>());
                __ic_custom_fd_renumber(0, 0);
                __ic_custom_fd_seek(0, 0, 0, null_mut::<wasi::Filesize>());
                __ic_custom_fd_sync(0);
                __ic_custom_fd_tell(0, null_mut::<wasi::Filesize>());
                __ic_custom_path_create_directory(0, null::<u8>(), 0);
                __ic_custom_path_filestat_get(0, 0, null::<u8>(), 0, null_mut::<wasi::Filestat>());
                __ic_custom_path_filestat_set_times(0, 0, null::<u8>(), 0, 0, 0, 0);
                __ic_custom_path_link(0, 0, null::<u8>(), 0, 0, null::<u8>(), 0);
                __ic_custom_path_readlink(0, null::<u8>(), 0, 0, 0, 0);
                __ic_custom_path_remove_directory(0, null::<u8>(), 0);
                __ic_custom_path_rename(0, null::<u8>(), 0, 0, null::<u8>(), 0);
                __ic_custom_path_symlink(0, 0, 0, 0, 0);
                __ic_custom_path_unlink_file(0, null::<u8>(), 0);

                __ic_custom_poll_oneoff(
                    null::<wasi::Subscription>(),
                    null_mut::<wasi::Event>(),
                    0,
                    0,
                );
                __ic_custom_proc_raise(0);
                __ic_custom_sched_yield();

                __ic_custom_sock_accept(0, 0, 0);
                __ic_custom_sock_recv(0, 0, 0, 0, 0, 0);
                __ic_custom_sock_send(0, 0, 0, 0, 0);
                __ic_custom_sock_shutdown(0, 0);

                __ic_custom_proc_exit(0);
            }
        }
    })
}

// the init function ensures the module is not thrown away by the linker
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe fn init(seed: &[u8]) {
    raw_init(seed.as_ptr(), seed.len());
}
