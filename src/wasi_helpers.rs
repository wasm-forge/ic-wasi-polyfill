use stable_fs::{
    error::Error,
    fs::{Fd, FileSystem},
    storage::types::DirEntryIndex,
};

use crate::wasi;


pub fn get_file_name<'a>(path: *const u8, path_len: wasi::Size) -> &'a str {
    let path_bytes = unsafe{std::slice::from_raw_parts(path, path_len as wasi::Size)};
    let file_name = unsafe {std::str::from_utf8_unchecked(path_bytes)};

    return file_name;
}

pub fn into_errno(error: Error) -> i32 {
    let errno = match error {
        stable_fs::error::Error::NotFound => wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidOffset => wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidFileType => wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidFileDescriptor => wasi::ERRNO_BADF,
        stable_fs::error::Error::InvalidBufferLength => wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidOpenFlags => wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidFdFlags => wasi::ERRNO_INVAL,
        stable_fs::error::Error::FileAlreadyExists => wasi::ERRNO_EXIST,
        stable_fs::error::Error::NameTooLong => wasi::ERRNO_NAMETOOLONG,
        stable_fs::error::Error::DirectoryNotEmpty => wasi::ERRNO_NOTEMPTY,
        stable_fs::error::Error::ExpectedToRemoveFile => wasi::ERRNO_ISDIR,
        stable_fs::error::Error::ExpectedToRemoveDirectory => wasi::ERRNO_NOTDIR,
        stable_fs::error::Error::CannotRemovedOpenedNode => wasi::ERRNO_BUSY,
    };

    errno.raw() as i32
}

pub fn into_wasi_filetype(file_type: stable_fs::storage::types::FileType) -> wasi::Filetype {
    match file_type {
        stable_fs::storage::types::FileType::Directory => wasi::FILETYPE_DIRECTORY,
        stable_fs::storage::types::FileType::RegularFile => wasi::FILETYPE_REGULAR_FILE,
        stable_fs::storage::types::FileType::SymbolicLink => wasi::FILETYPE_SYMBOLIC_LINK,
    }
}

pub fn _into_stable_fs_filetype(
    file_type: wasi::Filetype,
) -> Result<stable_fs::storage::types::FileType, stable_fs::error::Error> {
    match file_type {
        wasi::FILETYPE_DIRECTORY => Ok(stable_fs::storage::types::FileType::Directory),
        wasi::FILETYPE_REGULAR_FILE => Ok(stable_fs::storage::types::FileType::RegularFile),
        wasi::FILETYPE_SYMBOLIC_LINK => Ok(stable_fs::storage::types::FileType::SymbolicLink),
        _ => Err(stable_fs::error::Error::InvalidFileType),
    }
}

pub fn fd_readdir(
    fs: &FileSystem,
    fd: i32,
    cookie: i64,
    bytes: *mut u8,
    bytes_len: i32,
    res: *mut wasi::Size,
) -> i32 {
    let fd = fd as Fd;
    let bytes_len = bytes_len as usize;
    let mut result = 0usize;

    let buf = unsafe { std::slice::from_raw_parts_mut(bytes, bytes_len) };

    let meta = fs.metadata(fd);

    match meta {
        Ok(meta) => {
            let mut entry_index = if cookie == 0 {
                meta.first_dir_entry
            } else {
                Some(cookie as DirEntryIndex)
            };

            while let Some(index) = entry_index {
                let entry = fs.get_direntry(fd, index);
                if let Err(err) = entry {
                    return into_errno(err);
                }
                let entry = entry.unwrap();

                let put_result = put_single_entry(fs, fd, index, &mut buf[result..]);
                if let Err(err) = put_result {
                    return into_errno(err);
                }
                let put_result = put_result.unwrap();

                result += put_result;

                entry_index = entry.next_entry;

                if result == bytes_len {
                    break;
                }
            }

            unsafe {
                *res = result;
            }

            wasi::ERRNO_SUCCESS.raw() as i32
        }
        Err(err) => into_errno(err),
    }
}

pub fn put_single_entry(
    fs: &FileSystem,
    fd: Fd,
    index: DirEntryIndex,
    buf: &mut [u8],
) -> Result<usize, Error> {
    let direntry = fs.get_direntry(fd, index)?;
    let file_type = fs.metadata_from_node(direntry.node)?.file_type;
    let wasi_dirent = wasi::Dirent {
        d_next: direntry
            .next_entry
            .map(|x| x as u64)
            .unwrap_or(u64::MAX)
            .to_le(),
        d_ino: direntry.node.to_le(),
        d_namlen: (direntry.name.length as wasi::Dirnamlen).to_le(),
        d_type: into_wasi_filetype(file_type),
    };

    let result = fill_buffer(wasi_dirent, buf, &direntry.name);
    Ok(result)
}

fn fill_buffer(
    wasi_dirent: wasi::Dirent,
    buf: &mut [u8],
    filename: &stable_fs::storage::types::FileName,
) -> usize {
    use std::mem;
    use std::slice;

    let p: *const wasi::Dirent = &wasi_dirent;
    let p: *const u8 = p as *const u8;

    let s: &[u8] = unsafe { slice::from_raw_parts(p, mem::size_of::<wasi::Dirent>() - 3) };

    let result = usize::min(s.len(), buf.len());
    buf[0..result].copy_from_slice(&s[0..result]);

    let buf_len = buf.len();
    let buf = &mut buf[result..buf_len];

    eprintln!("filename: {}", filename.length);

    let filename = &filename.bytes[0..filename.length as usize];

    let result2 = usize::min(filename.len(), buf.len());
    buf[0..result2].copy_from_slice(&filename[0..result2]);
    result + result2
}

pub fn into_stable_fs_wence(whence: u8) -> stable_fs::fs::Whence {
    if whence == wasi::WHENCE_SET.raw() {
        return stable_fs::fs::Whence::SET;
    }
    if whence == wasi::WHENCE_CUR.raw() {
        return stable_fs::fs::Whence::CUR;
    }
    if whence == wasi::WHENCE_END.raw() {
        return stable_fs::fs::Whence::END;
    }

    panic!("Unsupported whence type!");
}

pub unsafe fn forward_to_debug(iovs: *const wasi::Ciovec, len: i32, res: *mut wasi::Size) -> i32 {
    let iovs = std::slice::from_raw_parts(iovs, len as usize);

    let mut written = 0;

    for iov in iovs {
        let buf = std::slice::from_raw_parts(iov.buf, iov.buf_len);
        let str = std::str::from_utf8(buf).unwrap_or("");
        ic_cdk::api::print(str);
        written += iov.buf_len;
    }

    *res = written;

    wasi::ERRNO_SUCCESS.raw() as i32
}

#[cfg(test)]
mod tests {
    use crate::{wasi, wasi_helpers::put_single_entry};
    use ic_stable_structures::DefaultMemoryImpl;
    use stable_fs::{
        fs::{FdStat, FileSystem},
        storage::{
            stable::StableStorage,
            types::{DirEntry, DirEntryIndex, FileName, Node},
        },
    };

    use super::{fd_readdir, fill_buffer};

    fn test_fs() -> FileSystem {
        FileSystem::new(Box::new(StableStorage::new(DefaultMemoryImpl::default()))).unwrap()
    }

    #[test]
    fn test_fill_buffer_normal_and_trimmed() {
        let direntry = DirEntry {
            name: FileName::new("test.txt").unwrap(),
            node: 45 as Node,
            next_entry: None,
            prev_entry: None,
        };

        let wasi_dirent = wasi::Dirent {
            d_next: 123 as wasi::Dircookie,
            d_ino: 234 as wasi::Inode,
            d_namlen: direntry.name.length as wasi::Dirnamlen,
            d_type: wasi::FILETYPE_REGULAR_FILE,
        };

        let expected = [
            123, 0, 0, 0, 0, 0, 0, 0, 234, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 4, 116, 101, 115, 116,
            46, 116, 120, 116,
        ];

        let mut buf = [0u8; 100];
        let len = fill_buffer(wasi_dirent, &mut buf, &direntry.name);

        assert_eq!(&expected[0..len], &buf[0..len]);
        assert_eq!(len, expected.len());

        let mut buf = [0u8; 27];
        let len = fill_buffer(wasi_dirent, &mut buf, &direntry.name);

        assert_eq!(&expected[0..len], &buf[0..len]);
        assert_eq!(len, buf.len());

        let mut buf = [0u8; 3];
        let len = fill_buffer(wasi_dirent, &mut buf, &direntry.name);

        assert_eq!(&expected[0..len], &buf[0..len]);
        assert_eq!(len, buf.len());
    }

    #[test]
    fn test_put_single_entry() {
        let mut fs = test_fs();

        let dir_fd = fs.root_fd();

        let _fd1 = fs
            .create_file(dir_fd, "test.txt", FdStat::default())
            .unwrap();
        let _fd2 = fs
            .create_file(dir_fd, "test2.txt", FdStat::default())
            .unwrap();
        let _fd3 = fs
            .create_file(dir_fd, "test3.txt", FdStat::default())
            .unwrap();
        let _fd4 = fs
            .create_file(dir_fd, "test4.txt", FdStat::default())
            .unwrap();

        let meta = fs.metadata(dir_fd);

        let first_entry = meta.unwrap().first_dir_entry.unwrap();

        let expected = [
            2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 4, 116, 101, 115, 116, 46,
            116, 120, 116,
        ];

        let mut buf = [0u8; 100];

        let len = put_single_entry(&fs, dir_fd, first_entry as DirEntryIndex, &mut buf).unwrap();

        assert_eq!(&expected[0..len], &buf[0..len]);
        assert_eq!(len, expected.len());
    }

    #[test]
    fn test_fd_readdir() {
        let mut fs = test_fs();

        let dir_fd = fs.root_fd();

        let _fd1 = fs
            .create_file(dir_fd, "test.txt", FdStat::default())
            .unwrap();
        let _fd2 = fs
            .create_file(dir_fd, "test2.txt", FdStat::default())
            .unwrap();
        let _fd3 = fs
            .create_file(dir_fd, "test3.txt", FdStat::default())
            .unwrap();
        let _fd4 = fs
            .create_file(dir_fd, "test4.txt", FdStat::default())
            .unwrap();

        let mut buf = [0u8; 200];

        let p = &mut buf as *mut u8;

        let mut bytes_used: wasi::Size = 0usize;

        let result = fd_readdir(
            &fs,
            fs.root_fd() as i32,
            2,
            p,
            buf.len() as i32,
            &mut bytes_used as *mut wasi::Size,
        );
        println!("{buf:?} result = {result} bytes_used = {bytes_used}");
        assert_eq!(result, 0);
        assert_eq!(bytes_used, 90);
    }
}
