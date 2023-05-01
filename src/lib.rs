use std::cell::RefCell;

use ic_stable_structures::DefaultMemoryImpl;
use stable_fs::fs::{DstBuf, Fd, SrcBuf};
use stable_fs::fs::{FdFlags, FdStat, FileSystem, OpenFlags};
use stable_fs::storage::stable::StableStorage;

use stable_fs::storage::types::FileSize;
use wasi_helpers::*;

mod wasi;
mod wasi_helpers;

thread_local! {
    static FS: RefCell<FileSystem> = RefCell::new(
        FileSystem::new(Box::new(StableStorage::new(DefaultMemoryImpl::default()))).unwrap()
    );
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
    ic_cdk::api::print("called __ic_custom_fd_write...");

    if fd < 3 {
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
    }
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
    ic_cdk::api::print(format!("called __ic_custom_fd_read {fd:?}"));

    // for now we don't support reading from the standart streams
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    FS.with(|fs| {
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
    })
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
    ic_cdk::api::print("called __ic_custom_fd_pwrite");

    if fd < 3 {
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
    }
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
    ic_cdk::api::print("called __ic_custom_fd_pread");

    // for now we don't support reading from the standard streams
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    FS.with(|fs| {
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
    })
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
    ic_cdk::api::print("called __ic_custom_fd_seek");

    // standart streams not supported
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    FS.with(|fs| {
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
    })
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
    // _dirflags contains the information on whether to follow the symlinks,
    // the symlinks are not supported yet by the file system
    prevent_elimination(&[dirflags]);

    ic_cdk::api::print("called __ic_custom_path_open");

    FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let path_bytes = std::slice::from_raw_parts(path, path_len as wasi::Size);

        let file_name = std::str::from_utf8_unchecked(path_bytes);

        let fd_stat = FdStat {
            flags: FdFlags::from_bits_truncate(fdflags as u16),
            rights_base: fs_rights_base as u64,
            rights_inheriting: fs_rights_inheriting as u64,
        };

        let open_flags = OpenFlags::from_bits_truncate(oflags as u16);

        let r = fs.open_or_create(parent_fd as Fd, file_name, fd_stat, open_flags);

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
    })
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_close(fd: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_close fd={fd}"));

    FS.with(|fs| {
        let res = fs.borrow_mut().close(fd as Fd);

        match res {
            Ok(_) => wasi::ERRNO_SUCCESS.raw() as i32,
            Err(er) => into_errno(er),
        }
    })
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_filestat_get(fd: i32, ret_val: *mut u8) -> i32 {
    ic_cdk::api::print("called __ic_custom_fd_filestat_get");

    FS.with(|fs| {
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
    })
}

/// Synchronize the data and metadata of a file to disk.
/// Note: This is similar to `fsync` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_sync(fd: i32) -> i32 {
    prevent_elimination(&[fd]);

    ic_cdk::api::print("called __ic_custom_fd_sync");

    wasi::ERRNO_SUCCESS.raw() as i32
}

/// Return the current offset of a file descriptor.
#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_tell(fd: i32, res: *mut wasi::Filesize) -> i32 {
    ic_cdk::api::print("called __ic_custom_fd_tell");

    // standard streams not supported
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    FS.with(|fs| {
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
    })
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_prestat_get(fd: i32, res: *mut wasi::Prestat) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_prestat_get fd={fd}"));

    FS.with(|fs| {
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
    })
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_prestat_dir_name(
    fd: i32,
    path: *mut u8,
    max_len: i32,
) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_prestat_dir_name fd={fd}"));
    let max_len = max_len as wasi::Size;

    FS.with(|fs| {
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
    })
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_advise(fd: i32, offset: i64, len: i64, advice: i32) -> i32 {
    prevent_elimination(&[offset as i32, len as i32]);
    ic_cdk::api::print("called __ic_custom_fd_advise");

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

    if is_badf {
        return wasi::ERRNO_BADF.raw() as i32;
    }

    wasi::ERRNO_SUCCESS.raw() as i32
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_allocate(fd: i32, offset: i64, len: i64) -> i32 {
    prevent_elimination(&[offset as i32, len as i32]);
    ic_cdk::api::print("called __ic_custom_fd_allocate");

    let mut result = wasi::ERRNO_SUCCESS.raw() as i32;

    FS.with(|fs| {
        let fs = fs.borrow();

        // check fd is real, for now don't do any allocation
        if fs.metadata(fd as Fd).is_err() {
            result = wasi::ERRNO_BADF.raw() as i32;
        };
    });

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_datasync(fd: i32) -> i32 {
    ic_cdk::api::print("called __ic_custom_fd_datasync");

    let mut result = wasi::ERRNO_SUCCESS.raw() as i32;

    FS.with(|fs| {
        let fs = fs.borrow();

        // check if the file descriptor is correct
        if fs.metadata(fd as Fd).is_err() {
            result = wasi::ERRNO_BADF.raw() as i32;
        };

        // we don't do the synchronization for now
    });

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_fd_fdstat_get(fd: i32, ret_fdstat: *mut wasi::Fdstat) -> i32 {
    ic_cdk::api::print("called __ic_custom_fd_fdstat_get");

    FS.with(|fs| {
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
    })
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_fdstat_set_flags(fd: i32, new_flags: i32) -> i32 {
    ic_cdk::api::print("called __ic_custom_fd_fdstat_set_flags");

    FS.with(|fs| {
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
    })
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_fdstat_set_rights(
    fd: i32,
    rights_base: i64,
    rights_inheriting: i64,
) -> i32 {
    ic_cdk::api::print("called __ic_custom_fd_fdstat_set_rights");

    FS.with(|fs| {
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
    })
}

#[no_mangle]
pub extern "C" fn __ic_custom_fd_filestat_set_size(fd: i32, size: i64) -> i32 {
    prevent_elimination(&[size as i32]);
    ic_cdk::api::print("called __ic_custom_fd_filestat_set_size");

    FS.with(|fs| {
        let fs = fs.borrow_mut();

        let stat = fs.get_stat(fd as Fd);

        match stat {
            Ok((_ftype, _fdstat)) => {
                // set_size is not supported yet

                wasi::ERRNO_SUCCESS.raw() as i32
            }
            Err(err) => wasi_helpers::into_errno(err),
        }
    })
}

#[no_mangle]
pub extern "C" fn __ic_custom_fd_filestat_set_times(
    fd: i32,
    atim: i64,
    mtim: i64,
    fst_flags: i32,
) -> i32 {
    ic_cdk::api::print("called __ic_custom_fd_filestat_set_times");

    let fst_flags = fst_flags as wasi::Fstflags;

    FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let meta = fs.metadata(fd as u32);

        match meta {
            Ok(_) => {
                // TODO: add option to get the current time based on the flags
                // for now only assign the clock specified in atim and mtim

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
    })
}

/// Read directory entries from a directory.
/// When successful, the contents of the output buffer consist of a sequence of
/// directory entries. Each directory entry consists of a `dirent` object,
/// followed by `dirent::d_namlen` bytes holding the name of the directory
/// entry.
/// This function fills the output buffer as much as possible, potentially
/// truncating the last directory entry. This allows the caller to grow its
/// read buffer size in case it's too small to fit a single large directory
/// entry, or skip the oversized directory entry.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_readdir(
    fd: i32,
    bytes: *mut u8,
    bytes_len: i32,
    cookie: i64,
    res: *mut wasi::Size,
) -> i32 {
    ic_cdk::api::print("called __ic_custom_fd_readdir");

    FS.with(|fs| {
        let fs = fs.borrow();
        wasi_helpers::fd_readdir(&fs, fd, cookie, bytes, bytes_len, res)
    })
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_renumber(fd_from: i32, fd_to: i32) -> i32 {
    ic_cdk::api::print("called __ic_custom_fd_renumber");

    FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let result = fs.renumber(fd_from as Fd, fd_to as Fd);

        match result {
            Ok(()) => wasi::ERRNO_SUCCESS.raw() as i32,
            Err(err) => into_errno(err),
        }
    })
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_random_get(buf: *mut u8, buf_len: wasi::Size) -> i32 {
    ic_cdk::api::print("called __ic_custom_random_get");

    let buf = std::slice::from_raw_parts_mut(buf, buf_len);
    for b in buf {
        *b = 0;
    }
    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_environ_get(arg0: i32, arg1: i32) -> i32 {
    ic_cdk::api::print("called __ic_custom_environ_get");
    prevent_elimination(&[arg0, arg1]);
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
    ic_cdk::api::print("called __ic_custom_environ_sizes_get");
    *len1 = 0;
    *len2 = 0;
    0
}

#[no_mangle]
pub extern "C" fn __ic_custom_proc_exit(code: i32) -> ! {
    panic!("WASI proc_exit called with code: {code}");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_args_get(arg0: i32, arg1: i32) -> i32 {
    ic_cdk::api::print("called __ic_custom_args_get");
    prevent_elimination(&[arg0, arg1]);
    // No-op.
    0
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
/// Return command-line argument data sizes.
pub unsafe extern "C" fn __ic_custom_args_sizes_get(
    len1: *mut wasi::Size,
    len2: *mut wasi::Size,
) -> i32 {
    ic_cdk::api::print("called __ic_custom_arg_sizes_get");
    *len1 = 0;
    *len2 = 0;
    0
}

/// Return the resolution of a clock.
/// Implementations are required to provide a non-zero value for supported clocks. For unsupported clocks,
/// return `errno::inval`.
/// Note: This is similar to `clock_getres` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_clock_res_get(arg0: i32, arg1: i32) -> i32 {
    prevent_elimination(&[arg0, arg1]);
    unimplemented!("WASI custom_clock_res_get is not implemented");
}

/// Return the time value of a clock.
/// Note: This is similar to `clock_gettime` in POSIX.
#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __ic_custom_clock_time_get(
    id: i32,
    precision: i64,
    result: *mut u64,
) -> i32 {
    prevent_elimination(&[id, precision as i32]);
    *result = 0;
    unimplemented!("WASI clock_time_get is not implemented");
}

/// Create a directory.
/// Note: This is similar to `mkdirat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_create_directory(arg0: i32, arg1: i32, arg2: i32) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2]);
    unimplemented!("WASI path_create_directory is not implemented");
}

/// Return the attributes of a file or directory.
/// Note: This is similar to `stat` in POSIX.
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
    prevent_elimination(&[flags]);
    FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let path_bytes = std::slice::from_raw_parts(path, path_len as wasi::Size);

        let file_name = std::str::from_utf8_unchecked(path_bytes);

        let fd_stat = FdStat {
            flags: FdFlags::from_bits_truncate(0),
            rights_base: 0,
            rights_inheriting: 0,
        };

        let open_flags = OpenFlags::empty();

        let fd = fs.open_or_create(parent_fd as Fd, file_name, fd_stat, open_flags);

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
    })
}

/// Adjust the timestamps of a file or directory.
/// Note: This is similar to `utimensat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_filestat_set_times(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i64,
    arg5: i64,
    arg6: i32,
) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2, arg3, arg4 as i32, arg5 as i32, arg6]);
    unimplemented!("WASI path_filestat_set_times is not implemented");
}

/// Create a hard link.
/// Note: This is similar to `linkat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_link(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    arg5: i32,
    arg6: i32,
) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2, arg3, arg4, arg5, arg6]);
    unimplemented!("WASI path_link is not implemented");
}

/// Read the contents of a symbolic link.
/// Note: This is similar to `readlinkat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_readlink(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    arg5: i32,
) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2, arg3, arg4, arg5]);
    unimplemented!("WASI path_readlink is not implemented");
}

/// Remove a directory.
/// Return `errno::notempty` if the directory is not empty.
/// Note: This is similar to `unlinkat(fd, path, AT_REMOVEDIR)` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_remove_directory(arg0: i32, arg1: i32, arg2: i32) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2]);
    unimplemented!("WASI path_remove_directory is not implemented");
}

/// Rename a file or directory.
/// Note: This is similar to `renameat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_rename(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    arg5: i32,
) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2, arg3, arg4, arg5]);
    unimplemented!("WASI path_rename is not implemented");
}

/// Create a symbolic link.
/// Note: This is similar to `symlinkat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_symlink(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2, arg3, arg4]);
    unimplemented!("WASI path_symlink is not implemented");
}

/// Unlink a file.
/// Return `errno::isdir` if the path refers to a directory.
/// Note: This is similar to `unlinkat(fd, path, 0)` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_unlink_file(arg0: i32, arg1: i32, arg2: i32) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2]);
    ic_cdk::api::print("called __ic_custom_path_unlink_file");
    // TODO: implement.
    // No-op for now.
    0
}

/// Concurrently poll for the occurrence of a set of events.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_poll_oneoff(arg0: i32, arg1: i32, arg2: i32, arg3: i32) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2, arg3]);
    unimplemented!("WASI poll_oneoff is not implemented");
}

/// Send a signal to the process of the calling thread.
/// Note: This is similar to `raise` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_proc_raise(arg0: i32) -> i32 {
    prevent_elimination(&[arg0]);
    unimplemented!("WASI proc_raise is not implemented");
}

/// Temporarily yield execution of the calling thread.
/// Note: This is similar to `sched_yield` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sched_yield() -> i32 {
    // No-op.
    0
}

/// Accept a new incoming connection.
/// Note: This is similar to `accept` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sock_accept(arg0: i32, arg1: i32, arg2: i32) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2]);
    unimplemented!("WASI sock_accept is not supported");
}

/// Receive a message from a socket.
/// Note: This is similar to `recv` in POSIX, though it also supports reading
/// the data into multiple buffers in the manner of `readv`.
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

/// Send a message on a socket.
/// Note: This is similar to `send` in POSIX, though it also supports writing
/// the data from multiple buffers in the manner of `writev`.
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

/// Shut down socket send and receive channels.
/// Note: This is similar to `shutdown` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sock_shutdown(arg0: i32, arg1: i32) -> i32 {
    prevent_elimination(&[arg0, arg1]);
    unimplemented!("WASI sock_shutdown is not supported");
}

thread_local! {
    static COUNTER: RefCell<i32> = RefCell::new(0);
}

/// A helper to introduce an artificial usage of arguments such that an
/// optimizing compiler doesn't remove the argument.
fn prevent_elimination(args: &[i32]) {
    COUNTER.with(|var| {
        if *var.borrow() == -1 {
            ic_cdk::api::print(format!("args: {args:?}"));
        }
    });
}

// the init function ensures the module is not thrown away by the linker
#[no_mangle]
pub extern "C" fn init() {
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
                __ic_custom_clock_res_get(0, 0);
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
                __ic_custom_path_create_directory(0, 0, 0);
                __ic_custom_path_filestat_get(0, 0, null::<u8>(), 0, null_mut::<wasi::Filestat>());
                __ic_custom_path_filestat_set_times(0, 0, 0, 0, 0, 0, 0);
                __ic_custom_path_link(0, 0, 0, 0, 0, 0, 0);
                __ic_custom_path_readlink(0, 0, 0, 0, 0, 0);
                __ic_custom_path_remove_directory(0, 0, 0);
                __ic_custom_path_rename(0, 0, 0, 0, 0, 0);
                __ic_custom_path_symlink(0, 0, 0, 0, 0);
                __ic_custom_path_unlink_file(0, 0, 0);
                __ic_custom_poll_oneoff(0, 0, 0, 0);
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
