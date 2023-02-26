
/// Read command-line argument data.
/// The size of the array should match that returned by `args_sizes_get`.
/// Each argument is expected to be `\0` terminated.
#[no_mangle]
#[export_name = "__ic_custom_args_get"]
pub extern "C" fn __ic_custom_args_get(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Return command-line argument data sizes.
#[no_mangle]
#[export_name = "__ic_custom_args_sizes_get"]
pub extern "C" fn __ic_custom_args_sizes_get1(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Read environment variable data.
/// The sizes of the buffers should match that returned by `environ_sizes_get`.
/// Key/value pairs are expected to be joined with `=`s, and terminated with `\0`s.
#[no_mangle]
pub extern "C" fn __ic_custom_environ_get(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Return environment variable data sizes.
#[no_mangle]
pub extern "C" fn __ic_custom_environ_sizes_get(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Return the resolution of a clock.
/// Implementations are required to provide a non-zero value for supported clocks. For unsupported clocks,
/// return `errno::inval`.
/// Note: This is similar to `clock_getres` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_clock_res_get(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Return the time value of a clock.
/// Note: This is similar to `clock_gettime` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_clock_time_get(_arg0: i32, _arg1: i64, _arg2: i32) -> i32 {
    0
}

/// Provide file advisory information on a file descriptor.
/// Note: This is similar to `posix_fadvise` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_advise(_arg0: i32, _arg1: i64, _arg2: i64, _arg3: i32) -> i32 {
    0
}

/// Force the allocation of space in a file.
/// Note: This is similar to `posix_fallocate` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_allocate(_arg0: i32, _arg1: i64, _arg2: i64) -> i32 {
    0
}

/// Close a file descriptor.
/// Note: This is similar to `close` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_close(_arg0: i32) -> i32 {
    0
}

/// Synchronize the data of a file to disk.
/// Note: This is similar to `fdatasync` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_datasync(_arg0: i32) -> i32 {
    0
}

/// Get the attributes of a file descriptor.
/// Note: This returns similar flags to `fsync(fd, F_GETFL)` in POSIX, as well as additional fields.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_fdstat_get(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Adjust the flags associated with a file descriptor.
/// Note: This is similar to `fcntl(fd, F_SETFL, flags)` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_fdstat_set_flags(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Adjust the rights associated with a file descriptor.
/// This can only be used to remove rights, and returns `errno::notcapable` if called in a way that would attempt to add rights
#[no_mangle]
pub extern "C" fn __ic_custom_fd_fdstat_set_rights(_arg0: i32, _arg1: i64, _arg2: i64) -> i32 {
    0
}

/// Return the attributes of an open file.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_filestat_get(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Adjust the size of an open file. If this increases the file's size, the extra bytes are filled with zeros.
/// Note: This is similar to `ftruncate` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_filestat_set_size(_arg0: i32, _arg1: i64) -> i32 {
    0
}

/// Adjust the timestamps of an open file or directory.
/// Note: This is similar to `futimens` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_filestat_set_times(_arg0: i32, _arg1: i64, _arg2: i64, _arg3: i32) -> i32 {
    0
}

/// Read from a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `preadv` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_pread(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32) -> i32 {
    0
}

/// Return a description of the given preopened file descriptor.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_prestat_get(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Return a description of the given preopened file descriptor.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_prestat_dir_name(_arg0: i32, _arg1: i32, _arg2: i32) -> i32 {
    0
}

/// Write to a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `pwritev` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_pwrite(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32) -> i32 {
    0
}


/// Read from a file descriptor.
/// Note: This is similar to `readv` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_read(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32) -> i32 {
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
pub extern "C" fn __ic_custom_fd_readdir(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32) -> i32 {
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
pub extern "C" fn __ic_custom_fd_renumber(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Move the offset of a file descriptor.
/// Note: This is similar to `lseek` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_seek(_arg0: i32, _arg1: i64, _arg2: i32, _arg3: i32) -> i32 {
    0
}

/// Synchronize the data and metadata of a file to disk.
/// Note: This is similar to `fsync` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_sync(_arg0: i32) -> i32 {
    0
}

/// Return the current offset of a file descriptor.
/// Note: This is similar to `lseek(fd, 0, SEEK_CUR)` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_tell(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Write to a file descriptor.
/// Note: This is similar to `writev` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_fd_write(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32) -> i32 {
    0
}

/// Create a directory.
/// Note: This is similar to `mkdirat` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_path_create_directory(_arg0: i32, _arg1: i32, _arg2: i32) -> i32 {
    0
}

/// Return the attributes of a file or directory.
/// Note: This is similar to `stat` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_path_filestat_get(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32) -> i32 {
    0
}

/// Adjust the timestamps of a file or directory.
/// Note: This is similar to `utimensat` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_path_filestat_set_times(
    _arg0: i32,
    _arg1: i32,
    _arg2: i32,
    _arg3: i32,
    _arg4: i64,
    _arg5: i64,
    _arg6: i32,
) -> i32 {
    0
}

/// Create a hard link.
/// Note: This is similar to `linkat` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_path_link(
    _arg0: i32,
    _arg1: i32,
    _arg2: i32,
    _arg3: i32,
    _arg4: i32,
    _arg5: i32,
    _arg6: i32,
) -> i32 {
    0
}

/// Open a file or directory.
/// The returned file descriptor is not guaranteed to be the lowest-numbered
/// file descriptor not currently open; it is randomized to prevent
/// applications from depending on making assumptions about indexes, since this
/// is error-prone in multi-threaded contexts. The returned file descriptor is
/// guaranteed to be less than 2**31.
/// Note: This is similar to `openat` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_path_open(
    _arg0: i32,
    _arg1: i32,
    _arg2: i32,
    _arg3: i32,
    _arg4: i32,
    _arg5: i64,
    _arg6: i64,
    _arg7: i32,
    _arg8: i32,
) -> i32 {
    0
}

/// Read the contents of a symbolic link.
/// Note: This is similar to `readlinkat` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_path_readlink(
    _arg0: i32,
    _arg1: i32,
    _arg2: i32,
    _arg3: i32,
    _arg4: i32,
    _arg5: i32,
) -> i32 {
    0
}

/// Remove a directory.
/// Return `errno::notempty` if the directory is not empty.
/// Note: This is similar to `unlinkat(fd, path, AT_REMOVEDIR)` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_path_remove_directory(_arg0: i32, _arg1: i32, _arg2: i32) -> i32 {
    0
}

/// Rename a file or directory.
/// Note: This is similar to `renameat` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_path_rename(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32, _arg5: i32) -> i32 {
        0
}

/// Create a symbolic link.
/// Note: This is similar to `symlinkat` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_path_symlink(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32) -> i32 {
    0
}

/// Unlink a file.
/// Return `errno::isdir` if the path refers to a directory.
/// Note: This is similar to `unlinkat(fd, path, 0)` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_path_unlink_file(_arg0: i32, _arg1: i32, _arg2: i32) -> i32 {
    0
}

/// Concurrently poll for the occurrence of a set of events.
#[no_mangle]
pub extern "C" fn __ic_custom_poll_oneoff(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32) -> i32 {
    0
}

/// Terminate the process normally. An exit code of 0 indicates successful
/// termination of the program. The meanings of other values is dependent on
/// the environment.
#[no_mangle]
pub extern "C" fn __ic_custom_proc_exit(_arg0: i32) {
    
}

/// Send a signal to the process of the calling thread.
/// Note: This is similar to `raise` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_proc_raise(_arg0: i32) -> i32 {
    0
}

/// Temporarily yield execution of the calling thread.
/// Note: This is similar to `sched_yield` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_sched_yield() -> i32 {
    0
}

/// Write high-quality random data into a buffer.
/// This function blocks when the implementation is unable to immediately
/// provide sufficient high-quality random data.
/// This function may execute slowly, so when large mounts of random data are
/// required, it's advisable to use this function to seed a pseudo-random
/// number generator, rather than to provide the random data directly.
#[no_mangle]
pub extern "C" fn __ic_custom_random_get(_arg0: i32, _arg1: i32) -> i32 {
    0
}

/// Accept a new incoming connection.
/// Note: This is similar to `accept` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_sock_accept(_arg0: i32, _arg1: i32, _arg2: i32) -> i32 {
    0
}

/// Receive a message from a socket.
/// Note: This is similar to `recv` in POSIX, though it also supports reading
/// the data into multiple buffers in the manner of `readv`.
#[no_mangle]
pub extern "C" fn __ic_custom_sock_recv(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32, _arg5: i32) -> i32 {
    0
}

/// Send a message on a socket.
/// Note: This is similar to `send` in POSIX, though it also supports writing
/// the data from multiple buffers in the manner of `writev`.
#[no_mangle]
pub extern "C" fn __ic_custom_sock_send(_arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32) -> i32 {
    0
}

/// Shut down socket send and receive channels.
/// Note: This is similar to `shutdown` in POSIX.
#[no_mangle]
pub extern "C" fn __ic_custom_sock_shutdown(_arg0: i32, _arg1: i32) -> i32 {
    0
}


#[no_mangle]
pub extern "C" fn add(left: usize, right: usize) -> usize {
    left + right
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
