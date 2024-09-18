use std::ptr::{null, null_mut};

use crate::wasi;
use crate::*;

#[cfg(test)]
fn create_test_file_with_content(parent_fd: Fd, file_name: &str, content: Vec<String>) -> Fd {
    let new_file_name = String::from(file_name);

    let mut file_fd = 0;

    let res = unsafe {
        __ic_custom_path_open(
            parent_fd as i32,
            0,
            new_file_name.as_ptr(),
            new_file_name.len() as i32,
            1 + 4 + 8,
            0,
            0,
            0,
            (&mut file_fd) as *mut i32,
        )
    };
    assert!(res == 0);

    let mut src = Vec::new();

    for str in content.iter() {
        src.push(wasi::Ciovec {
            buf: str.as_ptr(),
            buf_len: str.len(),
        });
    }

    let mut bytes_written: wasi::Size = 0;

    unsafe {
        __ic_custom_fd_write(
            file_fd,
            src.as_ptr(),
            src.len() as i32,
            (&mut bytes_written) as *mut wasi::Size,
        )
    };

    file_fd as Fd
}

#[cfg(test)]
fn create_test_file(parent_fd: Fd, file_name: &str) -> Fd {
    create_test_file_with_content(
        parent_fd,
        file_name,
        vec![
            String::from("This is a sample text."),
            String::from("1234567890"),
        ],
    )
}

#[cfg(test)]
fn read_directory(root_fd: Fd) -> Vec<String> {
    let len = 1000;
    let mut bytes: Vec<u8> = Vec::with_capacity(len);

    let mut new_length: wasi::Size = 0;

    let res = __ic_custom_fd_readdir(
        root_fd as i32,
        bytes.as_mut_ptr(),
        len as i32,
        0,
        (&mut new_length) as *mut wasi::Size,
    );

    assert!(res == 0);

    unsafe { bytes.set_len(new_length) };

    let mut folders: Vec<String> = Vec::new();

    let mut idx = 0usize;

    loop {
        unsafe {
            let d_namlen = bytes[idx + 16] as usize;

            let bytes_ptr = bytes.as_mut_ptr().add(idx + DIRENT_SIZE);

            let name_slice = std::slice::from_raw_parts(bytes_ptr, d_namlen);

            let name = std::str::from_utf8(name_slice)
                .expect("Failed to convert bytes to string")
                .to_string();

            folders.push(name);

            idx += DIRENT_SIZE + d_namlen;
        };

        if idx >= bytes.len() {
            break;
        }
    }

    folders
}

#[test]
fn test_environ_get() {
    let init_env = [
        ("PATH", "/usr/bin"),
        ("UID", "1028"),
        ("HOME", "/home/user"),
    ];

    let mut lines = Vec::new();

    for pair in init_env.iter() {
        lines.push(String::from(pair.0) + "=" + pair.1 + "\0");
    }

    let lines = lines;

    init(&[12, 3, 54, 1], &init_env);

    // get environment sizes
    let mut entry_count: wasi::Size = 0;
    let mut buffer_size: wasi::Size = 0;

    let ret = unsafe {
        __ic_custom_environ_sizes_get(
            (&mut entry_count) as *mut wasi::Size,
            (&mut buffer_size) as *mut wasi::Size,
        )
    };

    assert!(ret == 0);
    assert!(entry_count == lines.len());

    let mut expected_buffer_size = 0;
    for i in lines.iter().map(|x: &String| x.len()) {
        expected_buffer_size += i;
    }

    assert!(buffer_size == expected_buffer_size);

    let mut entry_table: Vec<wasi::Size> = Vec::with_capacity(entry_count);
    unsafe {
        entry_table.set_len(entry_count);
    }

    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_size);
    unsafe {
        buffer.set_len(buffer_size);
    }

    // get environment values
    let ret = unsafe {
        __ic_custom_environ_get(
            entry_table.as_mut_ptr() as *mut *mut u8,
            buffer.as_mut_ptr(),
        )
    };

    assert!(ret == 0);

    let computed_string = String::from_utf8(buffer).unwrap();

    let mut expected_string = String::new();
    for env_string in lines.iter() {
        expected_string += env_string;
    }

    assert!(computed_string == expected_string);
}

#[test]
fn test_random_get() {
    let init_env = [
        ("PATH", "/usr/bin"),
        ("UID", "1028"),
        ("HOME", "/home/user"),
    ];

    let seed = [12, 3, 54, 21];

    init(&seed, &init_env);

    let buf_len: wasi::Size = 14usize;

    let mut random_buf1: Vec<u8> = Vec::with_capacity(buf_len);

    let res = unsafe { __ic_custom_random_get(random_buf1.as_mut_ptr(), buf_len) };

    assert!(res == 0);

    unsafe { random_buf1.set_len(buf_len) };

    init_seed(&seed);

    let mut random_buf2: Vec<u8> = Vec::with_capacity(buf_len);

    let res = unsafe { __ic_custom_random_get(random_buf2.as_mut_ptr(), buf_len) };

    assert!(res == 0);

    unsafe { random_buf2.set_len(buf_len) };

    assert!(random_buf1 == random_buf2)
}

#[test]
#[should_panic]
fn test_proc_exit() {
    init(&[], &[]);

    __ic_custom_proc_exit(5);
}

#[test]
fn test_args_get() {
    init(&[], &[]);

    let mut entry_count: wasi::Size = 0;
    let mut buffer_size: wasi::Size = 0;

    let ret = unsafe {
        __ic_custom_args_sizes_get(
            (&mut entry_count) as *mut wasi::Size,
            (&mut buffer_size) as *mut wasi::Size,
        )
    };

    assert!(ret == 0);
    assert!(entry_count == 0);

    let expected_buffer_size = 0;

    assert!(buffer_size == expected_buffer_size);

    let mut entry_table: Vec<wasi::Size> = Vec::with_capacity(entry_count);
    unsafe {
        entry_table.set_len(entry_count);
    }

    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_size);
    unsafe {
        buffer.set_len(buffer_size);
    }

    // get environment values
    let ret = __ic_custom_args_get(
        entry_table.as_mut_ptr() as *mut *mut u8,
        buffer.as_mut_ptr(),
    );

    assert!(ret == 0);

    let computed_string = String::from_utf8(buffer).unwrap();

    let expected_string = String::new();

    assert!(computed_string == expected_string);
}

#[test]
fn test_clock_res_get_clock_time_get() {
    init(&[], &[]);

    let mut resolution: u64 = 0;

    let res = unsafe { __ic_custom_clock_res_get(0, (&mut resolution) as *mut u64) };

    assert!(res == 0);
    assert!(resolution == 1_000_000_000);

    let res =
        unsafe { __ic_custom_clock_time_get(0, 1_000_000_000, (&mut resolution) as *mut u64) };

    assert!(res == 0);
    assert!(resolution > 0);
}

#[test]
fn test_fd_prestat_init_fd_prestat_dir_name() {
    init(&[], &[]);

    let mut root_fd = 0;
    let mut prestat = wasi::Prestat {
        tag: 0,
        u: wasi::PrestatU {
            dir: wasi::PrestatDir { pr_name_len: 0 },
        },
    };

    // find working fd
    loop {
        let res =
            unsafe { __ic_custom_fd_prestat_get(root_fd, (&mut prestat) as *mut wasi::Prestat) };

        if root_fd > 10 {
            panic!();
        }

        if res == 0 {
            break;
        }

        root_fd += 1;
    }

    let root_fd = root_fd;

    let un_dir = unsafe { prestat.u.dir };
    let root_path_len = un_dir.pr_name_len;

    assert!(root_path_len > 0);

    // our default root_fd is known
    assert!(root_fd == 3);

    // find root folder
    let mut path: Vec<u8> = Vec::with_capacity(root_path_len);

    unsafe {
        path.set_len(root_path_len);
    }

    let res = unsafe {
        __ic_custom_fd_prestat_dir_name(root_fd, path.as_mut_ptr(), root_path_len as i32)
    };

    assert!(res == 0);

    let root_path = String::from_utf8(path).unwrap();

    // our default path is known
    assert!(root_path == "/");
}

#[test]
fn test_create_dirs_and_file_in_it() {
    init(&[], &[]);

    let root_fd = 3;
    let new_file_name = String::from("file.txt");

    // create dirs
    let new_folder_name1 = String::from("test_folder1");
    let res = unsafe {
        __ic_custom_path_create_directory(
            root_fd,
            new_folder_name1.as_ptr(),
            new_folder_name1.len() as i32,
        )
    };
    assert!(res == 0);

    let new_folder_name2 = String::from("test_folder2");
    let res = unsafe {
        __ic_custom_path_create_directory(
            root_fd,
            new_folder_name2.as_ptr(),
            new_folder_name2.len() as i32,
        )
    };
    assert!(res == 0);

    let new_folder_name3 = String::from("test_folder3");
    let res = unsafe {
        __ic_custom_path_create_directory(
            root_fd,
            new_folder_name3.as_ptr(),
            new_folder_name3.len() as i32,
        )
    };
    assert!(res == 0);

    // open first dir
    let mut parent_folder_fd = 0;

    let res = unsafe {
        __ic_custom_path_open(
            root_fd,
            0,
            new_folder_name1.as_ptr(),
            new_folder_name1.len() as i32,
            2,
            0,
            0,
            0,
            (&mut parent_folder_fd) as *mut i32,
        )
    };
    assert!(res == 0);

    assert!(parent_folder_fd > 0);

    //
    let mut new_file_fd = 0;

    let res = unsafe {
        __ic_custom_path_open(
            parent_folder_fd,
            0,
            new_file_name.as_ptr(),
            new_file_name.len() as i32,
            1 + 4 + 8,
            0,
            0,
            0,
            (&mut new_file_fd) as *mut i32,
        )
    };

    assert!(res == 0);

    // delete the second directory
    let ret = __ic_custom_path_remove_directory(
        root_fd,
        new_folder_name2.as_ptr(),
        new_folder_name2.len() as i32,
    );
    assert!(ret == 0);

    // check there are now 2 directories in the root folder and 1 file in the first directory

    let len = 1000;
    let mut bytes: Vec<u8> = Vec::with_capacity(len);

    let mut new_length: wasi::Size = 0;

    let res = __ic_custom_fd_readdir(
        root_fd,
        bytes.as_mut_ptr(),
        len as i32,
        0,
        (&mut new_length) as *mut wasi::Size,
    );

    assert!(res == 0);

    unsafe { bytes.set_len(new_length) };

    let mut folders: Vec<String> = Vec::new();

    let mut idx = 0usize;

    loop {
        unsafe {
            let d_namlen = bytes[idx + 16] as usize;

            let bytes_ptr = bytes.as_mut_ptr().add(idx + DIRENT_SIZE);

            let name_slice = std::slice::from_raw_parts(bytes_ptr, d_namlen);

            let name = std::str::from_utf8(name_slice)
                .expect("Failed to convert bytes to string")
                .to_string();

            folders.push(name);

            idx += DIRENT_SIZE + d_namlen;
        };

        if idx >= bytes.len() {
            break;
        }
    }

    assert!(folders.len() == 2);
    assert!(folders[0] == "test_folder1");
    assert!(folders[1] == "test_folder3");
}

#[test]
fn test_writing_and_reading() {
    init(&[], &[]);

    let root_fd = 3i32;
    let new_file_name = String::from("file.txt");

    let mut file_fd = create_test_file(root_fd as Fd, &new_file_name) as i32;

    __ic_custom_fd_close(file_fd);

    // open file for reading
    let res = unsafe {
        __ic_custom_path_open(
            root_fd,
            0,
            new_file_name.as_ptr(),
            new_file_name.len() as i32,
            0,
            0,
            0,
            0,
            (&mut file_fd) as *mut i32,
        )
    };
    assert!(res == 0);

    let mut buf_to_read1 = String::from("................");
    let mut buf_to_read2 = String::from("................");

    let mut read_buf = vec![
        wasi::Ciovec {
            buf: buf_to_read1.as_mut_ptr(),
            buf_len: buf_to_read1.len(),
        },
        wasi::Ciovec {
            buf: buf_to_read2.as_mut_ptr(),
            buf_len: buf_to_read2.len(),
        },
    ];

    let mut bytes_read: wasi::Size = 0;

    let res = unsafe {
        __ic_custom_fd_read(
            file_fd,
            read_buf.as_mut_ptr(),
            read_buf.len() as i32,
            (&mut bytes_read) as *mut wasi::Size,
        )
    };
    assert!(res == 0);

    assert!(buf_to_read1 == "This is a sample");
    assert!(buf_to_read2 == " text.1234567890");
}

#[test]
fn test_writing_and_reading_from_a_stationary_pointer() {
    init(&[], &[]);

    let root_fd = 3;
    let new_file_name = String::from("file.txt");

    let mut file_fd = 0;

    let res = unsafe {
        __ic_custom_path_open(
            root_fd,
            0,
            new_file_name.as_ptr(),
            new_file_name.len() as i32,
            1 + 4 + 8,
            0,
            0,
            0,
            (&mut file_fd) as *mut i32,
        )
    };
    assert!(res == 0);

    let text_to_write1 = String::from("This is a sample text.");
    let text_to_write2 = String::from("1234567890");

    let src = [
        wasi::Ciovec {
            buf: text_to_write1.as_ptr(),
            buf_len: text_to_write1.len(),
        },
        wasi::Ciovec {
            buf: text_to_write2.as_ptr(),
            buf_len: text_to_write2.len(),
        },
    ];

    let mut bytes_written: wasi::Size = 0;

    unsafe {
        __ic_custom_fd_pwrite(
            file_fd,
            src.as_ptr(),
            src.len() as i32,
            0,
            (&mut bytes_written) as *mut wasi::Size,
        )
    };

    __ic_custom_fd_close(file_fd);

    // open file for reading
    let res = unsafe {
        __ic_custom_path_open(
            root_fd,
            0,
            new_file_name.as_ptr(),
            new_file_name.len() as i32,
            0,
            0,
            0,
            0,
            (&mut file_fd) as *mut i32,
        )
    };
    assert!(res == 0);

    let mut buf_to_read1 = String::from("................");
    let mut buf_to_read2 = String::from("................");

    let mut read_buf = [
        wasi::Ciovec {
            buf: buf_to_read1.as_mut_ptr(),
            buf_len: buf_to_read1.len(),
        },
        wasi::Ciovec {
            buf: buf_to_read2.as_mut_ptr(),
            buf_len: buf_to_read2.len(),
        },
    ];

    let mut bytes_read: wasi::Size = 0;

    let res = unsafe {
        __ic_custom_fd_pread(
            file_fd,
            read_buf.as_mut_ptr(),
            (read_buf.len()) as i32,
            2,
            (&mut bytes_read) as *mut wasi::Size,
        )
    };

    assert!(res == 0);

    assert!(buf_to_read1 == "is is a sample t");
    assert!(buf_to_read2 == "ext.1234567890..");
}

#[test]
fn test_writing_and_reading_file_stats() {
    init(&[], &[]);

    let root_fd = 3;
    let new_file_name = String::from("file.txt");

    let mut file_fd = 0;

    unsafe {
        __ic_custom_path_open(
            root_fd,
            0,
            new_file_name.as_ptr(),
            new_file_name.len() as i32,
            1 + 4 + 8,
            0,
            0,
            0,
            (&mut file_fd) as *mut i32,
        )
    };

    let text_to_write1 = String::from("This is a sample text.");
    let text_to_write2 = String::from("1234567890");

    let src = vec![
        wasi::Ciovec {
            buf: text_to_write1.as_ptr(),
            buf_len: text_to_write1.len(),
        },
        wasi::Ciovec {
            buf: text_to_write2.as_ptr(),
            buf_len: text_to_write2.len(),
        },
    ];

    let mut bytes_written: wasi::Size = 0;

    unsafe {
        __ic_custom_fd_write(
            file_fd,
            src.as_ptr(),
            src.len() as i32,
            (&mut bytes_written) as *mut wasi::Size,
        )
    };

    __ic_custom_fd_sync(file_fd);

    let mut file_stat: wasi::Filestat = wasi::Filestat {
        dev: 0,
        ino: 0,
        filetype: wasi::FILETYPE_UNKNOWN,
        nlink: 0,
        size: 0,
        atim: 0,
        mtim: 0,
        ctim: 0,
    };

    unsafe {
        __ic_custom_fd_filestat_get(file_fd, &mut file_stat as *mut wasi::Filestat);
    }

    let mtime = ic_time();
    let atime = ic_time();

    __ic_custom_fd_filestat_set_times(file_fd, 1i64, 2i64, 1 + 2 + 4 + 8);

    unsafe {
        __ic_custom_fd_filestat_get(file_fd, &mut file_stat as *mut wasi::Filestat);
    }

    assert!(file_stat.filetype == wasi::FILETYPE_REGULAR_FILE);
    assert!(file_stat.nlink == 1);
    assert!(file_stat.size == 32);
    assert!(file_stat.atim > file_stat.ctim);
    assert!(file_stat.mtim > file_stat.ctim);

    unsafe {
        __ic_custom_fd_write(
            file_fd,
            src.as_ptr(),
            src.len() as i32,
            (&mut bytes_written) as *mut wasi::Size,
        )
    };

    __ic_custom_fd_filestat_set_times(file_fd, atime as i64, mtime as i64, 1 + 4);

    unsafe {
        __ic_custom_fd_filestat_get(file_fd, &mut file_stat as *mut wasi::Filestat);
    }

    assert!(file_stat.filetype == wasi::FILETYPE_REGULAR_FILE);
    assert!(file_stat.nlink == 1);
    assert!(file_stat.size == 64);
    assert!(file_stat.atim == atime);
    assert!(file_stat.mtim == mtime);
}

#[test]
fn test_forward_to_debug_is_called() {
    init(&[], &[]);

    let text_to_write1 = String::from("This is a sample text.");
    let text_to_write2 = String::from("1234567890");

    let src = vec![
        wasi::Ciovec {
            buf: text_to_write1.as_ptr(),
            buf_len: text_to_write1.len(),
        },
        wasi::Ciovec {
            buf: text_to_write2.as_ptr(),
            buf_len: text_to_write2.len(),
        },
    ];

    let mut bytes_written: wasi::Size = 0;

    let res = unsafe {
        __ic_custom_fd_write(
            1,
            src.as_ptr(),
            src.len() as i32,
            (&mut bytes_written) as *mut wasi::Size,
        )
    };

    assert!(res == 0);
    assert!(bytes_written > 0);
}

#[test]
fn test_link_seek_tell() {
    init(&[], &[]);

    let root_fd = 3i32;
    let new_file_name = String::from("file.txt");

    let file_fd = create_test_file(root_fd as Fd, &new_file_name) as i32;

    // test seek and tell
    let mut position: wasi::Filesize = 0;

    unsafe { __ic_custom_fd_tell(file_fd, &mut position as *mut wasi::Filesize) };

    assert!(position == 32);

    let mut position_after_seek: wasi::Filesize = 0;
    unsafe {
        __ic_custom_fd_seek(
            file_fd,
            10,
            0,
            &mut position_after_seek as *mut wasi::Filesize,
        )
    };

    assert!(position_after_seek == 10);

    // create link
    let link_file_name = String::from("file_link.txt");

    let res = __ic_custom_path_link(
        root_fd,
        0,
        new_file_name.as_ptr(),
        new_file_name.len() as i32,
        root_fd,
        link_file_name.as_ptr(),
        link_file_name.len() as i32,
    );

    assert!(res == 0);

    let mut link_file_fd = 0;

    // open file for reading
    let res = unsafe {
        __ic_custom_path_open(
            root_fd,
            0,
            link_file_name.as_ptr(),
            link_file_name.len() as i32,
            0,
            0,
            0,
            0,
            (&mut link_file_fd) as *mut i32,
        )
    };
    assert!(res == 0);

    // the file should be on the position
    let mut position_link: wasi::Filesize = 0;
    unsafe {
        __ic_custom_fd_seek(
            link_file_fd,
            10,
            0,
            &mut position_link as *mut wasi::Filesize,
        )
    };

    assert!(position_link == 10);

    let mut buf_to_read1 = String::from("................");

    let mut read_buf = vec![wasi::Ciovec {
        buf: buf_to_read1.as_mut_ptr(),
        buf_len: buf_to_read1.len(),
    }];

    let mut bytes_read: wasi::Size = 0;

    let res = unsafe {
        __ic_custom_fd_read(
            link_file_fd,
            read_buf.as_mut_ptr(),
            (read_buf.len()) as i32,
            (&mut bytes_read) as *mut wasi::Size,
        )
    };

    assert!(res == 0);

    assert!(buf_to_read1 == "sample text.1234");
}

#[test]
fn test_seek_types() {
    init(&[], &[]);

    let file_fd = create_test_file(3 as Fd, "file.txt") as i32;

    // test seek and tell
    let mut position: wasi::Filesize = 0;
    unsafe { __ic_custom_fd_tell(file_fd, &mut position as *mut wasi::Filesize) };
    assert!(position == 32);

    let mut position_after_seek: wasi::Filesize = 0;
    unsafe {
        __ic_custom_fd_seek(
            file_fd,
            10,
            0,
            &mut position_after_seek as *mut wasi::Filesize,
        )
    };
    assert!(position_after_seek == 10);

    let mut position_after_seek: wasi::Filesize = 0;
    unsafe {
        __ic_custom_fd_seek(
            file_fd,
            2,
            1,
            &mut position_after_seek as *mut wasi::Filesize,
        )
    };
    assert!(position_after_seek == 12);

    let mut position_after_seek: wasi::Filesize = 0;
    unsafe {
        __ic_custom_fd_seek(
            file_fd,
            -2,
            2,
            &mut position_after_seek as *mut wasi::Filesize,
        )
    };

    assert!(position_after_seek == 30);
}

#[test]
fn test_advice() {
    init(&[], &[]);

    let file_fd = create_test_file(3, "file.txt") as i32;

    assert!(__ic_custom_fd_advise(file_fd, 0, 500, 0) == 0);

    assert!(__ic_custom_fd_advise(file_fd + 10, 0, 500, 0) > 0);

    assert!(__ic_custom_fd_advise(file_fd, 0, 500, 10) > 0);
}

#[test]
fn test_allocate() {
    init(&[], &[]);

    let file_fd = create_test_file(3, "file.txt") as i32;

    assert!(__ic_custom_fd_allocate(file_fd, 0, 500) == 0);
    assert!(__ic_custom_fd_allocate(7, 0, 500) == wasi::ERRNO_BADF.raw() as i32);
}

#[test]
fn test_datasync() {
    init(&[], &[]);

    let file_fd = create_test_file(3, "file.txt") as i32;

    assert!(__ic_custom_fd_datasync(file_fd) == 0);
    assert!(__ic_custom_fd_datasync(7) == wasi::ERRNO_BADF.raw() as i32);
}

#[test]
fn test_rename_unlink() {
    init(&[], &[]);

    let filename1 = "file1.txt";
    let filename2 = "file2.txt";
    let filename3 = "file3.txt";
    let filename1_renamed = "file1_renamed.txt";

    let file_fd = create_test_file(3, filename1) as i32;
    __ic_custom_fd_close(file_fd);
    let file_fd = create_test_file(3, filename2) as i32;
    __ic_custom_fd_close(file_fd);
    let file_fd = create_test_file(3, filename3) as i32;
    __ic_custom_fd_close(file_fd);

    let res = __ic_custom_path_rename(
        3,
        filename1.as_ptr(),
        filename1.len() as i32,
        3,
        filename1_renamed.as_ptr(),
        filename1_renamed.len() as i32,
    );
    assert!(res == 0);

    let res = __ic_custom_path_unlink_file(3, filename3.as_ptr(), filename3.len() as i32);
    assert!(res == 0);

    // list files
    let persons = read_directory(3);

    assert_eq!(persons.len(), 2);

    assert!(persons.contains(&String::from("file1_renamed.txt")));
    assert!(persons.contains(&String::from("file2.txt")));
}

#[test]
#[should_panic]
fn test_unimplemented_path_readlink() {
    __ic_custom_path_readlink(0, null::<u8>(), 0, 0, 0, 0);
}

#[test]
#[should_panic]
fn test_unimplemented_path_symlink() {
    __ic_custom_path_symlink(0, 0, 0, 0, 0);
}

#[test]
#[should_panic]
fn test_unimplemented_poll_oneoff() {
    __ic_custom_poll_oneoff(
        null::<wasi::Subscription>(),
        null_mut::<wasi::Event>(),
        0,
        0,
    );
}

#[test]
#[should_panic]
fn test_unimplemented_proc_raise() {
    __ic_custom_proc_raise(0);
}

#[test]
#[should_panic]
fn test_unimplemented_sock_accept() {
    __ic_custom_sock_accept(0, 0, 0);
}

#[test]
#[should_panic]
fn test_unimplemented_sock_recv() {
    __ic_custom_sock_recv(0, 0, 0, 0, 0, 0);
}

#[test]
#[should_panic]
fn test_unimplemented_sock_send() {
    __ic_custom_sock_send(0, 0, 0, 0, 0);
}

#[test]
#[should_panic]
fn test_unimplemented_sock_shutdown() {
    __ic_custom_sock_shutdown(0, 0);
}

#[test]
fn test_fd_stat_get_fd_stat_set_flags() {
    init(&[], &[]);

    let mut stat: wasi::Fdstat = wasi::Fdstat {
        fs_filetype: wasi::FILETYPE_UNKNOWN,
        fs_flags: 0,
        fs_rights_base: wasi::RIGHTS_FD_READ,
        fs_rights_inheriting: wasi::RIGHTS_FD_READ,
    };

    let file_fd = create_test_file(3, "file.txt") as i32;

    let ret = unsafe { __ic_custom_fd_fdstat_get(file_fd, (&mut stat) as *mut wasi::Fdstat) };
    assert!(ret == 0);

    let mut fs_flags = stat.fs_flags;
    assert!(fs_flags == 0);

    fs_flags = 4;

    __ic_custom_fd_fdstat_set_flags(file_fd, fs_flags as i32);

    let ret = unsafe { __ic_custom_fd_fdstat_get(file_fd, (&mut stat) as *mut wasi::Fdstat) };
    assert!(ret == 0);

    assert!(stat.fs_flags == fs_flags);
}

#[test]
fn test_path_filestat_get_set_times() {
    init(&[], &[]);

    let filename = "file.txt";

    let file_fd = create_test_file(3, filename) as i32;
    __ic_custom_fd_close(file_fd);

    let mut filestat: wasi::Filestat = wasi::Filestat {
        dev: 0,
        ino: 0,
        filetype: wasi::FILETYPE_UNKNOWN,
        nlink: 0,
        size: 0,
        atim: 0,
        mtim: 0,
        ctim: 0,
    };

    unsafe {
        __ic_custom_path_filestat_get(
            3,
            0,
            filename.as_ptr(),
            filename.len() as i32,
            &mut filestat as *mut wasi::Filestat,
        )
    };

    __ic_custom_path_filestat_set_times(
        3,
        0,
        filename.as_ptr(),
        filename.len() as i32,
        123,
        456,
        1 + 4,
    );

    let mut filestat2: wasi::Filestat = wasi::Filestat {
        dev: 0,
        ino: 0,
        filetype: wasi::FILETYPE_UNKNOWN,
        nlink: 0,
        size: 0,
        atim: 0,
        mtim: 0,
        ctim: 0,
    };

    unsafe {
        __ic_custom_path_filestat_get(
            3,
            0,
            filename.as_ptr(),
            filename.len() as i32,
            &mut filestat2 as *mut wasi::Filestat,
        )
    };

    assert!(filestat2.atim == 123);
    assert!(filestat2.mtim == 456);
}

#[test]
fn test_fd_renumber_over_opened_file() {
    init(&[], &[]);

    let filename = "file.txt";

    let file_fd = create_test_file(3, filename) as i32;

    let filename2 = "file2.txt";
    let second_file_fd =
        create_test_file_with_content(3, filename2, vec![String::from("12345")]) as i32;

    let mut position: wasi::Filesize = 0 as wasi::Filesize;
    unsafe { __ic_custom_fd_tell(second_file_fd, &mut position as *mut wasi::Filesize) };

    assert!(position == 5);

    __ic_custom_fd_renumber(file_fd, second_file_fd);

    unsafe { __ic_custom_fd_tell(second_file_fd, &mut position as *mut wasi::Filesize) };

    // we expect the create_test_file to leave the cursor at the position 32
    assert!(position == 32);
}
