use crate::test::common::{create_test_file, libc};
use crate::wasi_mock::{Clockid, CLOCKID_MONOTONIC};
use crate::{init, wasi};

#[test]
fn test_interesting_paths() {
    init(&[], &[]);

    let dir_fd = 3;

    unsafe {
        // Create a directory in the scratch directory.
        wasi::path_create_directory(dir_fd, "dir").expect("creating dir");

        // Create a directory in the directory we just created.
        wasi::path_create_directory(dir_fd, "dir/nested").expect("creating a nested dir");

        // Create a file in the nested directory.
        let file_fd = create_test_file(dir_fd, "dir/nested/file");
        wasi::fd_close(file_fd).expect("closing a file");

        // Now open it with an absolute path.
        assert_eq!(
            wasi::path_open(dir_fd, 0, "/dir/nested/file", 0, 0, 0, 0)
                .expect_err("opening a file with an absolute path"),
            wasi::ERRNO_PERM
        );

        // Now open it with a path containing "..".
        let mut file_fd = wasi::path_open(
            dir_fd,
            0,
            "dir/.//nested/../../dir/nested/../nested///./file",
            0,
            0,
            0,
            0,
        )
        .expect("opening a file with \"..\" in the path");
        assert!(
            file_fd > libc::STDERR_FILENO as wasi::Fd,
            "file descriptor range check",
        );
        wasi::fd_close(file_fd).expect("closing a file");

        // TODO: trailing slashes enforce expect directory
        /*
        // Now open it with a trailing slash.
        assert_eq!(
            wasi::path_open(dir_fd, 0, "dir/nested/file/", 0, 0, 0, 0)
                .expect_err("opening a file with a trailing slash should fail"),
            wasi::ERRNO_NOTDIR
        );

        // Now open it with trailing slashes.
        assert_eq!(
            wasi::path_open(dir_fd, 0, "dir/nested/file///", 0, 0, 0, 0)
                .expect_err("opening a file with trailing slashes should fail"),
            wasi::ERRNO_NOTDIR
        );
        */

        // Now open the directory with a trailing slash.
        file_fd = wasi::path_open(dir_fd, 0, "dir/nested/", 0, 0, 0, 0)
            .expect("opening a directory with a trailing slash");
        assert!(
            file_fd > libc::STDERR_FILENO as wasi::Fd,
            "file descriptor range check",
        );
        wasi::fd_close(file_fd).expect("closing a file");

        // Now open the directory with trailing slashes.
        file_fd = wasi::path_open(dir_fd, 0, "dir/nested///", 0, 0, 0, 0)
            .expect("opening a directory with trailing slashes");
        assert!(
            file_fd > libc::STDERR_FILENO as wasi::Fd,
            "file descriptor range check",
        );
        wasi::fd_close(file_fd).expect("closing a file");

        // Now open it with a path containing too many ".."s.
        let bad_path = "dir/nested/../../../some_path/dir/nested/file";
        assert_eq!(
            wasi::path_open(dir_fd, 0, bad_path, 0, 0, 0, 0)
                .expect_err("opening a file with too many \"..\"s in the path should fail"),
            wasi::ERRNO_PERM
        );

        wasi::path_unlink_file(dir_fd, "dir/nested/file")
            .expect("unlink_file on a symlink should succeed");

        wasi::path_remove_directory(dir_fd, "dir/nested")
            .expect("remove_directory on a directory should succeed");

        wasi::path_remove_directory(dir_fd, "dir")
            .expect("remove_directory on a directory should succeed");
    }
}

#[test]
fn test_path_exists() {
    init(&[], &[]);

    let dir_fd = 3;

    unsafe {
        // Create a temporary directory
        wasi::path_create_directory(dir_fd, "subdir").expect("create directory");

        // Check directory exists:
        let file_stat = wasi::path_filestat_get(dir_fd, 0, "subdir").expect("reading file stats");
        assert_eq!(file_stat.filetype, wasi::FILETYPE_DIRECTORY);

        // Should still exist with symlink follow flag:
        let file_stat = wasi::path_filestat_get(dir_fd, wasi::LOOKUPFLAGS_SYMLINK_FOLLOW, "subdir")
            .expect("reading file stats");
        assert_eq!(file_stat.filetype, wasi::FILETYPE_DIRECTORY);

        // Create a file:
        let fd = create_test_file(dir_fd, "subdir/file");
        wasi::fd_close(fd).unwrap();

        // Check directory exists:
        let file_stat =
            wasi::path_filestat_get(dir_fd, 0, "subdir/file").expect("reading file stats");
        assert_eq!(file_stat.filetype, wasi::FILETYPE_REGULAR_FILE);

        // Should still exist with symlink follow flag:
        let file_stat =
            wasi::path_filestat_get(dir_fd, wasi::LOOKUPFLAGS_SYMLINK_FOLLOW, "subdir/file")
                .expect("reading file stats");
        assert_eq!(file_stat.filetype, wasi::FILETYPE_REGULAR_FILE);

        wasi::path_unlink_file(dir_fd, "subdir/file").expect("clean up");
        wasi::path_remove_directory(dir_fd, "subdir").expect("clean up");
    }
}

#[test]
fn test_path_filestat() {
    init(&[], &[]);

    let dir_fd = 3;

    unsafe {
        let precission = wasi::clock_res_get(CLOCKID_MONOTONIC).unwrap();

        let fdflags = wasi::FDFLAGS_APPEND;

        // Create a file in the scratch directory.
        let file_fd = wasi::path_open(
            dir_fd,
            0,
            "file",
            wasi::OFLAGS_CREAT,
            wasi::RIGHTS_FD_READ | wasi::RIGHTS_FD_WRITE,
            0,
            // Pass some flags for later retrieval
            fdflags,
        )
        .expect("opening a file");

        assert!(
            file_fd > libc::STDERR_FILENO as wasi::Fd,
            "file descriptor range check",
        );

        let fdstat = wasi::fd_fdstat_get(file_fd).expect("fd_fdstat_get");
        assert_eq!(
            fdstat.fs_flags & wasi::FDFLAGS_APPEND,
            wasi::FDFLAGS_APPEND,
            "file should have the APPEND fdflag used to create the file"
        );

        // Check file size
        let file_stat = wasi::path_filestat_get(dir_fd, 0, "file").expect("reading file stats");
        assert_eq!(file_stat.size, 0, "file size should be 0");

        // Check path_filestat_set_times
        let new_mtim = file_stat.mtim - 2 * precission;
        wasi::path_filestat_set_times(dir_fd, 0, "file", 0, new_mtim as u64, wasi::FSTFLAGS_MTIM)
            .expect("path_filestat_set_times should succeed");

        let modified_file_stat = wasi::path_filestat_get(dir_fd, 0, "file")
            .expect("reading file stats after path_filestat_set_times");

        assert_eq!(modified_file_stat.mtim, new_mtim, "mtim should change");

        assert_eq!(
            wasi::path_filestat_set_times(
                dir_fd,
                0,
                "file",
                0,
                new_mtim as u64,
                wasi::FSTFLAGS_MTIM | wasi::FSTFLAGS_MTIM_NOW,
            )
            .expect_err("MTIM and MTIM_NOW can't both be set"),
            wasi::ERRNO_INVAL
        );

        // check if the times were untouched
        let unmodified_file_stat = wasi::path_filestat_get(dir_fd, 0, "file")
            .expect("reading file stats after ERRNO_INVAL fd_filestat_set_times");

        assert_eq!(
            unmodified_file_stat.mtim, new_mtim,
            "mtim should not change"
        );

        // Invalid arguments to set_times:
        assert_eq!(
            wasi::path_filestat_set_times(
                dir_fd,
                0,
                "file",
                0,
                0,
                wasi::FSTFLAGS_ATIM | wasi::FSTFLAGS_ATIM_NOW,
            )
            .expect_err("ATIM & ATIM_NOW can't both be set"),
            wasi::ERRNO_INVAL
        );

        wasi::fd_close(file_fd).expect("closing a file");
        wasi::path_unlink_file(dir_fd, "file").expect("removing a file");
    }
}
