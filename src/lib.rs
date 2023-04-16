use std::cell::RefCell;
use std::mem;

use stable_fs::fs::{FileSystem, FdStat, FdFlags, OpenFlags};
use stable_fs::storage::transient::TransientStorage;
use stable_fs::fs::{Fd, SrcBuf, DstBuf};

use wasi_helpers::*;

mod wasi;
mod wasi_helpers;


thread_local! {
    static FS: RefCell<FileSystem> = RefCell::new(FileSystem::new(Box::new(TransientStorage::new())).unwrap());
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __ic_custom_fd_write(fd: i32, iovs: *const wasi::Ciovec, len: i32, res: *mut wasi::Size) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_write..."));

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
pub extern "C" fn __ic_custom_fd_read(fd: i32, iovs: *const wasi::Ciovec, len: i32, res: *mut wasi::Size) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_read"));

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

/// Read from a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `preadv` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_pread(fd: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_pread"));
    0
}

/// Write to a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `pwritev` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_pwrite(fd: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_pwrite"));
    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_seek(fd: i32, delta: i64, whence: i32, res: *mut wasi::Size) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_seek"));
    
    // standart streams not supported
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    FS.with(|fs| {
        
        let mut fs = fs.borrow_mut();

        unsafe {

            match fs.seek(fd as Fd, delta, wasi_helpers::into_stable_fs_wence(whence as u8)) {
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
pub unsafe extern "C" fn __ic_custom_path_open(
    parent_fd: i32,
    _dirflags: i32,
    path: *const u8,
    path_len: i32,

    oflags: i32,
    fs_rights_base: i64,
    fs_rights_inheriting: i64,

    fdflags: i32,
    res: *mut wasi::Size,
) -> i32 {
    // _dirflags contains the information on whether to follow the symlinks,
    // the symlinks are not supported yet by the file system

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



#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_close(fd: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_close fd={}", fd));
    
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
    ic_cdk::api::print(format!("called __ic_custom_fd_filestat_get"));

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
    
                    *ret_val = mem::transmute(value);
                }
    
                wasi::ERRNO_SUCCESS.raw() as i32
            },
            Err(er) => {
                into_errno(er)
            }
        }
    })

}


/// Synchronize the data and metadata of a file to disk.
/// Note: This is similar to `fsync` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_sync(_fd: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_sync"));

    wasi::ERRNO_SUCCESS.raw() as i32
}

/// Return the current offset of a file descriptor.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_tell(fd: i32, res: *mut wasi::Size) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_tell"));

    // standart streams not supported
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    FS.with(|fs| {
        
        let mut fs = fs.borrow_mut();

        unsafe {

            match fs.tell(fd as Fd) {
                Ok(pos) => {
                    *res = pos as wasi::Size;

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
pub extern "C" fn __ic_custom_fd_prestat_get(fd: i32, res: *mut wasi::Prestat) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_prestat_get fd={}", fd));

    FS.with(|fs| {
        
        let fs = fs.borrow();

        if fd as Fd == fs.root_fd() {
            let root_len = fs.root_path().len();

            let prestat = wasi::Prestat {
                tag: 0,
                u: wasi::PrestatU {
                    dir: wasi::PrestatDir {
                        pr_name_len: root_len
                    }
                }
            };
            
            unsafe {
                *res = prestat;
            }

            return wasi::ERRNO_SUCCESS.raw() as i32;
        } else {
            return wasi::ERRNO_BADF.raw() as i32;
        }
    })


}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_prestat_dir_name(fd: i32, path: *mut u8, max_len: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_prestat_dir_name fd={}", fd));
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
            
            return wasi::ERRNO_SUCCESS.raw() as i32;
        } else {
            return wasi::ERRNO_BADF.raw() as i32;
        }
    })
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_advise(fd: i32, _offset: i64, _len: i64, advice: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_advise"));

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
pub extern "C" fn __ic_custom_fd_allocate(fd: i32, _offset: i64, _len: i64) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_allocate"));

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
    ic_cdk::api::print(format!("called __ic_custom_fd_datasync"));

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
pub extern "C" fn __ic_custom_fd_fdstat_get(fd: i32, ret_fdstat: *mut wasi::Fdstat) -> i32 {
    ic_cdk::api::print("called __ic_custom_fd_fdstat_get");

    FS.with(|fs| {
        
        let fs = fs.borrow();

        let stat = fs.get_stat(fd as Fd);

        match stat {

            Ok((ftype, fdstat)) => {

                let tmp_fd_stat = wasi::Fdstat {
                    fs_filetype: into_wasi_filetype(ftype),
                    fs_flags: fdstat.flags.bits() as u16,
                    fs_rights_base: fdstat.rights_base,
                    fs_rights_inheriting: fdstat.rights_inheriting
                };

                unsafe {
                    *ret_fdstat = tmp_fd_stat;
                }

                wasi::ERRNO_SUCCESS.raw() as i32
            },
            Err(err) => {
                wasi_helpers::into_errno(err)
            }
        }
    })

}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_fdstat_set_flags(fd: i32, new_flags: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_fdstat_set_flags"));

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
                    Ok(_) => {
                        wasi::ERRNO_SUCCESS.raw() as i32
                    },
                    Err(err) => {
                        wasi_helpers::into_errno(err)
                    }
                }
            },
            Err(err) => {
                wasi_helpers::into_errno(err)
            }
        }
    })
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_fdstat_set_rights(fd: i32, rights_base: i64, rights_inheriting: i64) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_fdstat_set_rights"));

    FS.with(|fs| {
        

        let mut fs = fs.borrow_mut();

        let stat = fs.get_stat(fd as Fd);

        match stat {
            Ok((_ftype, mut fdstat)) => {

                fdstat.rights_base = rights_base as u64;
                fdstat.rights_inheriting = rights_inheriting as u64;

                match fs.set_stat(fd as Fd, fdstat) {
                    Ok(_) => {
                        wasi::ERRNO_SUCCESS.raw() as i32
                    },
                    Err(err) => {
                        wasi_helpers::into_errno(err)
                    }
                }
            },
            Err(err) => {
                wasi_helpers::into_errno(err)
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn __ic_custom_fd_filestat_set_size(fd: i32, _size: i64) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_filestat_set_size"));

    FS.with(|fs| {
        
        let fs = fs.borrow_mut();

        let stat = fs.get_stat(fd as Fd);

        match stat {
            Ok((_ftype, _fdstat)) => {

                // set_size is not supported yet

                wasi::ERRNO_SUCCESS.raw() as i32
            },
            Err(err) => {
                wasi_helpers::into_errno(err)
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn __ic_custom_fd_filestat_set_times(fd: i32, atim: i64, mtim: i64, fst_flags: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_filestat_set_times"));

    let fst_flags = fst_flags as wasi::Fstflags;

    FS.with(|fs| {
        
        let mut fs = fs.borrow_mut();

        let meta = fs.metadata(fd as u32);

        match meta {
            Ok(_) => {

                // for now don't 
                if fst_flags & wasi::FSTFLAGS_ATIM > 0 {
                    fs.set_accessed_time(fd as Fd, atim as u64);
                }

                if fst_flags & wasi::FSTFLAGS_MTIM > 0 {
                    fs.set_accessed_time(fd as Fd, mtim as u64);
                }

                wasi::ERRNO_SUCCESS.raw() as i32

            },
            Err(err) => {
                wasi_helpers::into_errno(err)
            }
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
pub extern "C" fn __ic_custom_fd_readdir(fd: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_readdir"));
    
    0
}

/// Atomically replace a file descriptor by renumbering another file descriptor.
/// Due to the strong focus on thread safety, this environment does not provide
/// a mechanism to duplicate or renumber a file descriptor to an arbitrary
/// number, like `dup2()`. This would be prone to race conditions, as an actual
/// file descriptor with the same number could be allocated by a different
/// thread at the same time.
/// This function provides a way to atomically renumber file descriptors, which
/// would disappear if `dup2()` were to be removed entirely.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_renumber(fd: i32, _arg1: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_renumber"));
    0
}


#[no_mangle]
#[inline(never)]
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
pub extern "C" fn __ic_custom_environ_get(_environ: *mut *mut u8, _environ_buf: *mut u8) -> i32 {
    ic_cdk::api::print("called __ic_custom_environ_get");


    0
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __ic_custom_environ_sizes_get(len1: *mut wasi::Size, len2: *mut wasi::Size) -> i32 {
    ic_cdk::api::print("called __ic_custom_environ_sizes_get");
    *len1 = 0;
    *len2 = 0;
    0
}

#[no_mangle]
pub  extern "C" fn __ic_custom_proc_exit(_arg0: i32) -> ! {
    ic_cdk::api::print("called __ic_custom_proc_exit");

    panic!("exit")
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_args_get(_arg0: i32, _arg1: i32) -> i32 {
    ic_cdk::api::print("called __ic_custom_args_get");

    0
}

/// Return command-line argument data sizes.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_args_sizes_get(_arg0: i32, _arg1: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_args_sizes_get"));

    0
}


/// Return the resolution of a clock.
/// Implementations are required to provide a non-zero value for supported clocks. For unsupported clocks,
/// return `errno::inval`.
/// Note: This is similar to `clock_getres` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_clock_res_get(_arg0: i32, _arg1: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_clock_res_get"));
    0
}

/// Return the time value of a clock.
/// Note: This is similar to `clock_gettime` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_clock_time_get(id: i32, precission: i64, result: *mut u64) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_clock_res_get"));


    0
}

/// Create a directory.
/// Note: This is similar to `mkdirat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_create_directory(_arg0: i32, _arg1: i32, _arg2: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_path_create_directory"));
    0
}

/// Return the attributes of a file or directory.
/// Note: This is similar to `stat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_filestat_get(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_path_filestat_get"));
    0
}

/// Adjust the timestamps of a file or directory.
/// Note: This is similar to `utimensat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_filestat_set_times(
    _arg0: i32,
    _arg1: i32,
    _arg2: i32,
    _arg3: i32,
    _arg4: i64,
    _arg5: i64,
    _arg6: i32,
) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_path_filestat_set_times"));
    0
}

/// Create a hard link.
/// Note: This is similar to `linkat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_link(
    _arg0: i32,
    _arg1: i32,
    _arg2: i32,
    _arg3: i32,
    _arg4: i32,
    _arg5: i32,
    _arg6: i32,
) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_path_link"));
    0
}

/// Read the contents of a symbolic link.
/// Note: This is similar to `readlinkat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_readlink(
    _arg0: i32,
    _arg1: i32,
    _arg2: i32,
    _arg3: i32,
    _arg4: i32,
    _arg5: i32,
) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_path_readlink"));
    0
}

/// Remove a directory.
/// Return `errno::notempty` if the directory is not empty.
/// Note: This is similar to `unlinkat(fd, path, AT_REMOVEDIR)` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_remove_directory(_arg0: i32, _arg1: i32, _arg2: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_path_remove_directory"));
    0
}

/// Rename a file or directory.
/// Note: This is similar to `renameat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_rename(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32, _arg5: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_path_rename"));
    0
}

/// Create a symbolic link.
/// Note: This is similar to `symlinkat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_symlink(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32) -> i32 {
    0
}

/// Unlink a file.
/// Return `errno::isdir` if the path refers to a directory.
/// Note: This is similar to `unlinkat(fd, path, 0)` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_path_unlink_file(_arg0: i32, _arg1: i32, _arg2: i32) -> i32 {
    0
}

/// Concurrently poll for the occurrence of a set of events.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_poll_oneoff(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32) -> i32 {
    0
}


/// Send a signal to the process of the calling thread.
/// Note: This is similar to `raise` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_proc_raise(_arg0: i32) -> i32 {
    0
}

/// Temporarily yield execution of the calling thread.
/// Note: This is similar to `sched_yield` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sched_yield() -> i32 {
    0
}

/// Accept a new incoming connection.
/// Note: This is similar to `accept` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sock_accept(_arg0: i32, _arg1: i32, _arg2: i32) -> i32 {
    0
}

/// Receive a message from a socket.
/// Note: This is similar to `recv` in POSIX, though it also supports reading
/// the data into multiple buffers in the manner of `readv`.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sock_recv(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32, _arg5: i32) -> i32 {
    0
}

/// Send a message on a socket.
/// Note: This is similar to `send` in POSIX, though it also supports writing
/// the data from multiple buffers in the manner of `writev`.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sock_send(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32) -> i32 {
    0
}

/// Shut down socket send and receive channels.
/// Note: This is similar to `shutdown` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_sock_shutdown(_arg0: i32, _arg1: i32) -> i32 {
    ic_cdk::api::print("called __ic_custom_sock_shutdown");
    0
}

thread_local! {

    static COUNTER: RefCell<i32> = RefCell::new(0);
}

// the init function ensures the module is not thrown away by the linker
#[no_mangle]
pub extern "C" fn init() {


    COUNTER.with(|var| {

        if var.borrow().clone() == -1 {

            // dummy calls to trick the linker not to throw away the functions
            unsafe {
                __ic_custom_fd_write(0, 0 as *const wasi::Ciovec, 0, 0 as *mut wasi::Size);
                __ic_custom_fd_read(0, 0 as *const wasi::Ciovec, 0, 0 as *mut wasi::Size);
                __ic_custom_fd_close(0);

                __ic_custom_fd_prestat_get(0, 0 as *mut wasi::Prestat);
                __ic_custom_fd_prestat_dir_name(0, 0 as *mut u8, 0);

                __ic_custom_path_open(0,0,0 as *const u8,0,0,0,0,0,0 as *mut wasi::Size);
                __ic_custom_random_get(0 as *mut u8, 0);

                __ic_custom_environ_get(0 as *mut *mut u8, 0 as *mut u8);
                __ic_custom_environ_sizes_get(0 as *mut wasi::Size, 0 as *mut wasi::Size);

                __ic_custom_args_get(0, 0);
                __ic_custom_args_sizes_get(0, 0);
                __ic_custom_clock_res_get(0, 0);
                __ic_custom_clock_time_get(0, 0, 0 as *mut u64);
                __ic_custom_fd_advise(0, 0, 0, 0);
                __ic_custom_fd_allocate(0, 0, 0);
                __ic_custom_fd_datasync(0);
                __ic_custom_fd_fdstat_get(0, 0 as *mut wasi::Fdstat);
                __ic_custom_fd_fdstat_set_flags(0, 0);
                __ic_custom_fd_fdstat_set_rights(0, 0, 0);
                __ic_custom_fd_filestat_get(0, 0 as *mut u8);
                __ic_custom_fd_filestat_set_size(0, 0);
                __ic_custom_fd_filestat_set_times(0, 0, 0, 0);
                __ic_custom_fd_pread(0, 0, 0, 0, 0);
                __ic_custom_fd_pwrite(0, 0, 0, 0, 0);
                
                __ic_custom_fd_readdir(0, 0, 0, 0, 0);
                __ic_custom_fd_renumber(0, 0);
                __ic_custom_fd_seek(0, 0, 0, 0 as *mut wasi::Size);
                __ic_custom_fd_sync(0);
                __ic_custom_fd_tell(0, 0 as *mut wasi::Size);
                __ic_custom_path_create_directory(0, 0, 0);
                __ic_custom_path_filestat_get(0, 0, 0, 0, 0);
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
