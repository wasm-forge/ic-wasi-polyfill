use std::{mem, slice, str};

use crate::{init, test::common::libc, wasi};

const BUF_LEN: usize = 256;

struct DirEntry {
    dirent: wasi::Dirent,
    name: String,
}

// Manually reading the output from fd_readdir is tedious and repetitive,
// so encapsulate it into an iterator
struct ReadDir<'a> {
    buf: &'a [u8],
}

impl<'a> ReadDir<'a> {
    fn from_slice(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

impl Iterator for ReadDir<'_> {
    type Item = DirEntry;

    fn next(&mut self) -> Option<DirEntry> {
        unsafe {
            if self.buf.len() < mem::size_of::<wasi::Dirent>() {
                return None;
            }

            // Read the data
            let dirent_ptr = self.buf.as_ptr() as *const wasi::Dirent;
            let dirent = dirent_ptr.read_unaligned();

            if self.buf.len() < mem::size_of::<wasi::Dirent>() + dirent.d_namlen as usize {
                return None;
            }

            let name_ptr = dirent_ptr.offset(1) as *const u8;
            // NOTE Linux syscall returns a NUL-terminated name, but WASI doesn't
            let namelen = dirent.d_namlen as usize;
            let slice = slice::from_raw_parts(name_ptr, namelen);
            let name = str::from_utf8(slice).expect("invalid utf8").to_owned();

            // Update the internal state
            let delta = mem::size_of_val(&dirent) + namelen;
            self.buf = &self.buf[delta..];

            DirEntry { dirent, name }.into()
        }
    }
}

/// Return the entries plus a bool indicating EOF.
unsafe fn exec_fd_readdir(fd: wasi::Fd, cookie: wasi::Dircookie) -> (Vec<DirEntry>, bool) {
    let mut buf: [u8; BUF_LEN] = [0; BUF_LEN];
    let bufused =
        wasi::fd_readdir(fd, buf.as_mut_ptr(), BUF_LEN, cookie).expect("failed fd_readdir");
    assert!(bufused <= BUF_LEN);

    let sl = slice::from_raw_parts(buf.as_ptr(), bufused);
    let dirs: Vec<_> = ReadDir::from_slice(sl).collect();
    let eof = bufused < BUF_LEN;
    (dirs, eof)
}

unsafe fn assert_empty_dir(dir_fd: wasi::Fd) {
    let stat = wasi::fd_filestat_get(dir_fd).expect("failed filestat");

    let (mut dirs, eof) = exec_fd_readdir(dir_fd, 0);
    assert!(eof, "expected to read the entire directory");
    dirs.sort_by_key(|d| d.name.clone());
    assert_eq!(dirs.len(), 2, "expected two entries in an empty directory");
    let mut dirs = dirs.into_iter();

    // the first entry should be `.`
    let dir = dirs.next().expect("first entry is None");
    assert_eq!(dir.name, ".", "first name");
    assert_eq!(dir.dirent.d_type, wasi::FILETYPE_DIRECTORY, "first type");
    assert_eq!(dir.dirent.d_namlen, 1);
    assert_eq!(dir.dirent.d_ino, stat.ino);

    // the second entry should be `..`
    let dir = dirs.next().expect("second entry is None");
    assert_eq!(dir.name, "..", "second name");
    assert_eq!(dir.dirent.d_type, wasi::FILETYPE_DIRECTORY, "second type");
    assert_eq!(dir.dirent.d_namlen, 2);

    assert!(
        dirs.next().is_none(),
        "the directory should be seen as empty"
    );
}

#[test]
fn test_fd_readdir() {
    init(&[], &[]);

    let dir_fd: wasi::Fd = 3;

    unsafe {
        // Check the behavior in an empty directory
        assert_empty_dir(dir_fd);

        // Add a file and check the behavior
        let file_fd = wasi::path_open(
            dir_fd,
            0,
            "file",
            wasi::OFLAGS_CREAT,
            wasi::RIGHTS_FD_READ | wasi::RIGHTS_FD_WRITE,
            0,
            0,
        )
        .expect("failed to create file");
        assert!(
            file_fd > libc::STDERR_FILENO as wasi::Fd,
            "file descriptor range check",
        );

        let file_stat = wasi::fd_filestat_get(file_fd).expect("failed filestat");
        wasi::fd_close(file_fd).expect("closing a file");

        wasi::path_create_directory(dir_fd, "nested").expect("create a directory");
        let nested_fd = wasi::path_open(dir_fd, 0, "nested", 0, 0, 0, 0)
            .expect("failed to open nested directory");
        let nested_stat = wasi::fd_filestat_get(nested_fd).expect("failed filestat");

        // Execute another readdir
        let (mut dirs, eof) = exec_fd_readdir(dir_fd, 0);
        assert!(eof, "expected to read the entire directory");
        assert_eq!(dirs.len(), 4, "expected four entries");
        // Save the data about the last entry. We need to do it before sorting.
        let lastfile_cookie = dirs[2].dirent.d_next;
        let lastfile_name = dirs[3].name.clone();
        dirs.sort_by_key(|d| d.name.clone());
        let mut dirs = dirs.into_iter();

        let dir = dirs.next().expect("first entry is None");
        assert_eq!(dir.name, ".", "first name");
        let dir = dirs.next().expect("second entry is None");
        assert_eq!(dir.name, "..", "second name");
        let dir = dirs.next().expect("third entry is None");
        // check the file info
        assert_eq!(dir.name, "file", "file name doesn't match");
        assert_eq!(
            dir.dirent.d_type,
            wasi::FILETYPE_REGULAR_FILE,
            "type for the real file"
        );
        assert_eq!(dir.dirent.d_ino, file_stat.ino);
        let dir = dirs.next().expect("fourth entry is None");
        // check the directory info
        assert_eq!(dir.name, "nested", "nested directory name doesn't match");
        assert_eq!(
            dir.dirent.d_type,
            wasi::FILETYPE_DIRECTORY,
            "type for the nested directory"
        );
        assert_eq!(dir.dirent.d_ino, nested_stat.ino);

        // check if cookie works as expected
        let (dirs, eof) = exec_fd_readdir(dir_fd, lastfile_cookie);
        assert!(eof, "expected to read the entire directory");
        assert_eq!(dirs.len(), 1, "expected one entry");
        assert_eq!(dirs[0].name, lastfile_name, "name of the only entry");

        // check if nested directory shows up as empty
        assert_empty_dir(nested_fd);
        wasi::fd_close(nested_fd).expect("closing a nested directory");

        wasi::path_unlink_file(dir_fd, "file").expect("removing a file");
        wasi::path_remove_directory(dir_fd, "nested").expect("removing a nested directory");
    }
}

#[test]
fn test_fd_readdir_lots() {
    init(&[], &[]);

    let dir_fd: wasi::Fd = 3;

    unsafe {
        // TODO: make sure, it is "fast enough" for a 1000 files
        // Add a file and check the behavior
        for count in 0..300 {
            let file_fd = wasi::path_open(
                dir_fd,
                0,
                &format!("file.{count}"),
                wasi::OFLAGS_CREAT,
                wasi::RIGHTS_FD_READ | wasi::RIGHTS_FD_WRITE,
                0,
                0,
            )
            .expect("failed to create file");
            assert!(
                file_fd > libc::STDERR_FILENO as wasi::Fd,
                "file descriptor range check",
            );
            wasi::fd_close(file_fd).expect("closing a file");
        }

        // Count the entries to ensure that we see the correct number.
        let mut total = 0;
        let mut cookie = 0;
        loop {
            let (dirs, eof) = exec_fd_readdir(dir_fd, cookie);
            total += dirs.len();
            if eof {
                break;
            }
            cookie = dirs[dirs.len() - 1].dirent.d_next;
        }
        assert_eq!(total, 302, "expected all entries plus . and ..");

        for count in 0..300 {
            wasi::path_unlink_file(dir_fd, &format!("file.{count}")).expect("removing a file");
        }

        // make sure we have an empty folder again
        assert_empty_dir(dir_fd);
    }
}

#[test]
fn test_fd_readdir_unicode_boundary() {
    init(&[], &[]);

    let dir_fd: wasi::Fd = 3;

    unsafe {
        let filename = "Действие";
        let file_fd = wasi::path_open(
            dir_fd,
            0,
            filename,
            wasi::OFLAGS_CREAT,
            wasi::RIGHTS_FD_READ | wasi::RIGHTS_FD_WRITE,
            0,
            0,
        )
        .expect("failed to create file");
        assert!(
            file_fd > libc::STDERR_FILENO as wasi::Fd,
            "file descriptor range check",
        );
        wasi::fd_close(file_fd).expect("closing a file");

        let mut buf = Vec::new();
        'outer: loop {
            let len = wasi::fd_readdir(dir_fd, buf.as_mut_ptr(), buf.capacity(), 0).unwrap();
            buf.set_len(len);

            for entry in ReadDir::from_slice(&buf) {
                if entry.name == filename {
                    break 'outer;
                }
            }
            buf = Vec::with_capacity(buf.capacity() + 1);
        }

        wasi::path_unlink_file(dir_fd, filename).expect("removing a file");
    }
}
