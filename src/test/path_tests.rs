use crate::test::common::{create_test_file, libc};
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
