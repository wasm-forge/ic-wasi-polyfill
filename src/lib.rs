use std::cell::RefCell;

use stable_fs::fs::{FileSystem, FdStat, FdFlags, OpenFlags};
use stable_fs::storage::transient::TransientStorage;
use stable_fs::fs::{Fd, SrcBuf, DstBuf};

use wasi_helpers::into_errno;

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
            let src_io_vec = std::slice::from_raw_parts(src_io_vec, len as usize);
            
            match fs.write_vec(fd as Fd, src_io_vec) {
                Ok(r) => {
                    *res = r as usize;
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

            let dst_io_vec = std::slice::from_raw_parts(dst_io_vec, len as usize);

            match fs.read_vec(fd as Fd, dst_io_vec) {
                Ok(r) => {
                    *res = r as usize;
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
    dirflags: i32,
    path: *const u8,
    path_len: i32,

    oflags: i32,
    fs_rights_base: i64,
    fs_rights_inheriting: i64,

    fdflags: i32,
    res: *mut wasi::Size,
) -> i32 {

    ic_cdk::api::print("called __ic_custom_path_open");

    FS.with(|fs| {
        
        let mut fs = fs.borrow_mut();
        
        let path_bytes = std::slice::from_raw_parts(path, path_len as usize);
        
        let file_name = std::str::from_utf8_unchecked(path_bytes);

        let fd_stat = FdStat {
            flags: FdFlags::from_bits_truncate(fdflags as u16),
            rights_base: fs_rights_base as u64,
            rights_inheriting: fs_rights_inheriting as u64,
        };

        let open_flags = OpenFlags::from_bits_truncate(oflags as u16);

        let res = fs.open_or_create(parent_fd as Fd, file_name, fd_stat, open_flags);

        match res {
            Ok(_) => wasi::ERRNO_SUCCESS.raw() as i32,
            Err(er) => into_errno(er),
        }
    })
}



#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __ic_custom_fd_close(fd: i32) -> i32 {
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
pub unsafe extern "C" fn __ic_custom_fd_prestat_get(fd: i32, _res: *mut wasi::Size) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_prestat_get fd={}", fd));
    wasi::ERRNO_BADF.raw() as i32
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __ic_custom_fd_prestat_dir_name(fd: i32, _path: *mut u8, _path_len: wasi::Size) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_prestat_dir_name fd={}", fd));
    wasi::ERRNO_INVAL.raw() as i32
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
pub extern "C" fn __ic_custom_clock_time_get(_arg0: i32, _arg1: i64, _arg2: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_clock_res_get"));
    0
}

/// Provide file advisory information on a file descriptor.
/// Note: This is similar to `posix_fadvise` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_advise(_arg0: i32, _arg1: i64, _arg2: i64, _arg3: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_advise"));
    0
}

/// Force the allocation of space in a file.
/// Note: This is similar to `posix_fallocate` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_allocate(_arg0: i32, _arg1: i64, _arg2: i64) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_allocate"));
    0
}


/// Synchronize the data of a file to disk.
/// Note: This is similar to `fdatasync` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_datasync(_arg0: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_datasync"));
    0
}

/// Get the attributes of a file descriptor.
/// Note: This returns similar flags to `fsync(fd, F_GETFL)` in POSIX, as well as additional fields.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_fdstat_get(_arg0: i32, _arg1: i32) -> i32 {
    ic_cdk::api::print("called __ic_custom_fd_fdstat_get");

    0
}

/// Adjust the flags associated with a file descriptor.
/// Note: This is similar to `fcntl(fd, F_SETFL, flags)` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_fdstat_set_flags(_arg0: i32, _arg1: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_fdstat_set_flags"));
    0
}

/// Adjust the rights associated with a file descriptor.
/// This can only be used to remove rights, and returns `errno::notcapable` if called in a way that would attempt to add rights
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_fdstat_set_rights(_arg0: i32, _arg1: i64, _arg2: i64) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_fdstat_set_rights"));
    0
}

/// Return the attributes of an open file.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_filestat_get(_arg0: i32, _arg1: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_filestat_get"));
    0
}

/// Adjust the size of an open file. If this increases the file's size, the extra bytes are filled with zeros.
/// Note: This is similar to `ftruncate` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_filestat_set_size(_arg0: i32, _arg1: i64) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_filestat_set_size"));
    0
}

/// Adjust the timestamps of an open file or directory.
/// Note: This is similar to `futimens` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_filestat_set_times(_arg0: i32, _arg1: i64, _arg2: i64, _arg3: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_filestat_set_times"));
    0
}

/// Read from a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `preadv` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_pread(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_pread"));
    0
}

/// Write to a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `pwritev` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_pwrite(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_pwrite"));
    0
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
pub extern "C" fn __ic_custom_fd_readdir(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32) -> i32 {
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
pub extern "C" fn __ic_custom_fd_renumber(_arg0: i32, _arg1: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_renumber"));
    0
}

/// Move the offset of a file descriptor.
/// Note: This is similar to `lseek` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_seek(_arg0: i32, _arg1: i64, _arg2: i32, _arg3: i32) -> i32 {
       ic_cdk::api::print(format!("called __ic_custom_fd_seek"));

    0
}

/// Synchronize the data and metadata of a file to disk.
/// Note: This is similar to `fsync` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_sync(_arg0: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_sync"));
    0
}

/// Return the current offset of a file descriptor.
/// Note: This is similar to `lseek(fd, 0, SEEK_CUR)` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __ic_custom_fd_tell(_arg0: i32, _arg1: i32) -> i32 {
    ic_cdk::api::print(format!("called __ic_custom_fd_tell"));
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

                __ic_custom_fd_prestat_get(0, 0 as *mut wasi::Size);
                __ic_custom_fd_prestat_dir_name(0, 0 as *mut u8, 0);

                __ic_custom_path_open(0,0,0 as *const u8,0,0,0,0,0,0 as *mut wasi::Size);
                __ic_custom_random_get(0 as *mut u8, 0);

                __ic_custom_environ_get(0 as *mut *mut u8, 0 as *mut u8);
                __ic_custom_environ_sizes_get(0 as *mut wasi::Size, 0 as *mut wasi::Size);

                __ic_custom_args_get(0, 0);
                __ic_custom_args_sizes_get(0, 0);
                __ic_custom_clock_res_get(0, 0);
                __ic_custom_clock_time_get(0, 0, 0);
                __ic_custom_fd_advise(0, 0, 0, 0);
                __ic_custom_fd_allocate(0, 0, 0);
                __ic_custom_fd_datasync(0);
                __ic_custom_fd_fdstat_get(0, 0);
                __ic_custom_fd_fdstat_set_flags(0, 0);
                __ic_custom_fd_fdstat_set_rights(0, 0, 0);
                __ic_custom_fd_filestat_get(0, 0);
                __ic_custom_fd_filestat_set_size(0, 0);
                __ic_custom_fd_filestat_set_times(0, 0, 0, 0);
                __ic_custom_fd_pread(0, 0, 0, 0, 0);
                __ic_custom_fd_pwrite(0, 0, 0, 0, 0);
                
                __ic_custom_fd_readdir(0, 0, 0, 0, 0);
                __ic_custom_fd_renumber(0, 0);
                __ic_custom_fd_seek(0, 0, 0, 0);
                __ic_custom_fd_sync(0);
                __ic_custom_fd_tell(0, 0);
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
