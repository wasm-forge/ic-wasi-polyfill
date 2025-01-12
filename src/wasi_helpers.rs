use stable_fs::{
    error::Error,
    fs::{Fd, FileSystem},
    storage::types::DirEntryIndex,
};

#[cfg(target_arch = "wasm32")]
use crate::wasi;
#[cfg(not(all(target_arch = "wasm32")))]
use crate::wasi_mock as wasi;

pub fn get_file_name<'a>(path: *const u8, path_len: wasi::Size) -> &'a str {
    let path_bytes = unsafe { std::slice::from_raw_parts(path, path_len as wasi::Size) };
    let file_name = unsafe { std::str::from_utf8_unchecked(path_bytes) };

    file_name
}

pub const DIRENT_SIZE: usize = std::mem::size_of::<wasi::Dirent>();

pub fn into_errno(error: Error) -> i32 {
    let errno = match error {
        stable_fs::error::Error::ArgumentListTooLong => wasi::ERRNO_2BIG,
        stable_fs::error::Error::PermissionDenied => wasi::ERRNO_ACCES,
        stable_fs::error::Error::AddressInUse => wasi::ERRNO_ADDRINUSE,
        stable_fs::error::Error::AddressNotAvailable => wasi::ERRNO_ADDRNOTAVAIL,
        stable_fs::error::Error::AddressFamilyNotSupported => wasi::ERRNO_AFNOSUPPORT,
        stable_fs::error::Error::ResourceUnavailableOrOperationWouldBlock => wasi::ERRNO_AGAIN,
        stable_fs::error::Error::ConnectionAlreadyInProgress => wasi::ERRNO_ALREADY,
        stable_fs::error::Error::BadFileDescriptor => wasi::ERRNO_BADF,
        stable_fs::error::Error::BadMessage => wasi::ERRNO_BADMSG,
        stable_fs::error::Error::DeviceOrResourceBusy => wasi::ERRNO_BUSY,
        stable_fs::error::Error::OperationCanceled => wasi::ERRNO_CANCELED,
        stable_fs::error::Error::NoChildProcesses => wasi::ERRNO_CHILD,
        stable_fs::error::Error::ConnectionAborted => wasi::ERRNO_CONNABORTED,
        stable_fs::error::Error::ConnectionRefused => wasi::ERRNO_CONNREFUSED,
        stable_fs::error::Error::ConnectionReset => wasi::ERRNO_CONNRESET,
        stable_fs::error::Error::ResourceDeadlockWouldOccur => wasi::ERRNO_DEADLK,
        stable_fs::error::Error::DestinationAddressRequired => wasi::ERRNO_DESTADDRREQ,
        stable_fs::error::Error::MathematicsArgumentOutOfDomainOfFunction => wasi::ERRNO_DOM,
        stable_fs::error::Error::Reserved19 => wasi::ERRNO_DQUOT,
        stable_fs::error::Error::FileExists => wasi::ERRNO_EXIST,
        stable_fs::error::Error::BadAddress => wasi::ERRNO_FAULT,
        stable_fs::error::Error::FileTooLarge => wasi::ERRNO_FBIG,
        stable_fs::error::Error::HostIsUnreachable => wasi::ERRNO_HOSTUNREACH,
        stable_fs::error::Error::IdentifierRemoved => wasi::ERRNO_IDRM,
        stable_fs::error::Error::IllegalByteSequence => wasi::ERRNO_ILSEQ,
        stable_fs::error::Error::OperationInProgress => wasi::ERRNO_INPROGRESS,
        stable_fs::error::Error::InterruptedFunction => wasi::ERRNO_INTR,
        stable_fs::error::Error::InvalidArgument => wasi::ERRNO_INVAL,
        stable_fs::error::Error::IOError => wasi::ERRNO_IO,
        stable_fs::error::Error::SocketIsConnected => wasi::ERRNO_ISCONN,
        stable_fs::error::Error::IsDirectory => wasi::ERRNO_ISDIR,
        stable_fs::error::Error::TooManyLevelsOfSymbolicLinks => wasi::ERRNO_LOOP,
        stable_fs::error::Error::FileDescriptorValueTooLarge => wasi::ERRNO_MFILE,
        stable_fs::error::Error::TooManyLinks => wasi::ERRNO_MLINK,
        stable_fs::error::Error::MessageTooLarge => wasi::ERRNO_MSGSIZE,
        stable_fs::error::Error::Reserved36 => wasi::ERRNO_MULTIHOP,
        stable_fs::error::Error::FilenameTooLong => wasi::ERRNO_NAMETOOLONG,
        stable_fs::error::Error::NetworkIsDown => wasi::ERRNO_NETDOWN,
        stable_fs::error::Error::ConnectionAbortedByNetwork => wasi::ERRNO_NETRESET,
        stable_fs::error::Error::NetworkUnreachable => wasi::ERRNO_NETUNREACH,
        stable_fs::error::Error::TooManyFilesOpenInSystem => wasi::ERRNO_NFILE,
        stable_fs::error::Error::NoBufferSpaceAvailable => wasi::ERRNO_NOBUFS,
        stable_fs::error::Error::NoSuchDevice => wasi::ERRNO_NODEV,
        stable_fs::error::Error::NoSuchFileOrDirectory => wasi::ERRNO_NOENT,
        stable_fs::error::Error::ExecutableFileFormatError => wasi::ERRNO_NOEXEC,
        stable_fs::error::Error::NoLocksAvailable => wasi::ERRNO_NOLCK,
        stable_fs::error::Error::Reserved47 => wasi::ERRNO_NOLINK,
        stable_fs::error::Error::NotEnoughSpace => wasi::ERRNO_NOMEM,
        stable_fs::error::Error::NoMessageOfTheDesiredType => wasi::ERRNO_NOMSG,
        stable_fs::error::Error::ProtocolNotAvailable => wasi::ERRNO_NOPROTOOPT,
        stable_fs::error::Error::NoSpaceLeftOnDevice => wasi::ERRNO_NOSPC,
        stable_fs::error::Error::FunctionNotSupported => wasi::ERRNO_NOSYS,
        stable_fs::error::Error::SocketNotConnected => wasi::ERRNO_NOTCONN,
        stable_fs::error::Error::NotADirectoryOrSymbolicLink => wasi::ERRNO_NOTDIR,
        stable_fs::error::Error::DirectoryNotEmpty => wasi::ERRNO_NOTEMPTY,
        stable_fs::error::Error::StateNotRecoverable => wasi::ERRNO_NOTRECOVERABLE,
        stable_fs::error::Error::NotASocket => wasi::ERRNO_NOTSOCK,
        stable_fs::error::Error::NotSupportedOrOperationNotSupportedOnSocket => wasi::ERRNO_NOTSUP,
        stable_fs::error::Error::InappropriateIOControlOperation => wasi::ERRNO_NOTTY,
        stable_fs::error::Error::NoSuchDeviceOrAddress => wasi::ERRNO_NXIO,
        stable_fs::error::Error::ValueTooLargeToBeStoredInDataType => wasi::ERRNO_OVERFLOW,
        stable_fs::error::Error::PreviousOwnerDied => wasi::ERRNO_OWNERDEAD,
        stable_fs::error::Error::OperationNotPermitted => wasi::ERRNO_PERM,
        stable_fs::error::Error::BrokenPipe => wasi::ERRNO_PIPE,
        stable_fs::error::Error::ProtocolError => wasi::ERRNO_PROTO,
        stable_fs::error::Error::ProtocolNotSupported => wasi::ERRNO_PROTONOSUPPORT,
        stable_fs::error::Error::ProtocolWrongTypeForSocket => wasi::ERRNO_PROTOTYPE,
        stable_fs::error::Error::ResultTooLarge => wasi::ERRNO_RANGE,
        stable_fs::error::Error::ReadOnlyFileSystem => wasi::ERRNO_ROFS,
        stable_fs::error::Error::InvalidSeek => wasi::ERRNO_SPIPE,
        stable_fs::error::Error::NoSuchProcess => wasi::ERRNO_SRCH,
        stable_fs::error::Error::Reserved72 => wasi::ERRNO_STALE,
        stable_fs::error::Error::ConnectionTimedOut => wasi::ERRNO_TIMEDOUT,
        stable_fs::error::Error::TextFileBusy => wasi::ERRNO_TXTBSY,
        stable_fs::error::Error::CrossDeviceLink => wasi::ERRNO_XDEV,
        stable_fs::error::Error::ExtensionCapabilitiesInsufficient => wasi::ERRNO_NOTCAPABLE,
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
        _ => Err(stable_fs::error::Error::InvalidArgument),
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
    if cookie == -1 {
        unsafe {
            *res = 0;
        }
        return wasi::ERRNO_SUCCESS.raw() as i32;
    }

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
                *res = std::cmp::min(result, bytes_len);
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

    let d_next = direntry.next_entry.map(|x| x as u64).unwrap_or(u64::MAX);

    let wasi_dirent = wasi::Dirent {
        d_next,
        d_ino: (index as u64),
        d_namlen: (direntry.name.length as wasi::Dirnamlen),
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
    use std::slice;

    let p: *const wasi::Dirent = &wasi_dirent;
    let p: *const u8 = p as *const u8;

    let s: &[u8] = unsafe { slice::from_raw_parts(p, DIRENT_SIZE) };

    let result = usize::min(s.len(), buf.len());
    buf[0..result].copy_from_slice(&s[0..result]);

    let buf_len = buf.len();
    let buf = &mut buf[result..buf_len];

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

#[cfg(test)]
mod tests {
    use crate::{wasi, wasi_helpers::put_single_entry, DIRENT_SIZE};
    use ic_stable_structures::DefaultMemoryImpl;
    use stable_fs::{
        fs::{FdStat, FileSystem, OpenFlags},
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
            name: FileName::new("test.txt".as_bytes()).unwrap(),
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
            123, 0, 0, 0, 0, 0, 0, 0, 234, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 4, 243, 243, 243, 116,
            101, 115, 116, 46, 116, 120, 116,
        ];

        let mut buf = [0u8; 100];
        let len = fill_buffer(wasi_dirent, &mut buf, &direntry.name);

        // stabilize test, the three bytes can take random value here...
        buf[DIRENT_SIZE - 3] = 243;
        buf[DIRENT_SIZE - 2] = 243;
        buf[DIRENT_SIZE - 1] = 243;

        assert_eq!(&expected[0..len], &buf[0..len]);
        assert_eq!(len, expected.len());

        let mut buf = [0u8; 27];
        let len = fill_buffer(wasi_dirent, &mut buf, &direntry.name);
        // stabilize test, the three bytes can take random value here...
        buf[DIRENT_SIZE - 3] = 243;
        buf[DIRENT_SIZE - 2] = 243;
        buf[DIRENT_SIZE - 1] = 243;

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
            .open(dir_fd, "test.txt", FdStat::default(), OpenFlags::CREATE, 0)
            .unwrap();
        let _fd2 = fs
            .open(dir_fd, "test2.txt", FdStat::default(), OpenFlags::CREATE, 0)
            .unwrap();
        let _fd3 = fs
            .open(dir_fd, "test3.txt", FdStat::default(), OpenFlags::CREATE, 0)
            .unwrap();
        let _fd4 = fs
            .open(dir_fd, "test4.txt", FdStat::default(), OpenFlags::CREATE, 0)
            .unwrap();

        let meta = fs.metadata(dir_fd);

        let first_entry = meta.unwrap().first_dir_entry.unwrap();

        let expected = [
            2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 4, 243, 245, 246, 116, 101,
            115, 116, 46, 116, 120, 116,
        ];

        let mut buf = [0u8; 100];

        let len = put_single_entry(&fs, dir_fd, first_entry as DirEntryIndex, &mut buf).unwrap();

        // stabilize test, the three bytes can take random value here...
        buf[DIRENT_SIZE - 3] = 243;
        buf[DIRENT_SIZE - 2] = 245;
        buf[DIRENT_SIZE - 1] = 246;

        assert_eq!(&expected[0..len], &buf[0..len]);
        assert_eq!(len, expected.len());
    }

    #[test]
    fn test_fd_readdir() {
        let mut fs = test_fs();

        let dir_fd = fs.root_fd();

        let _fd1 = fs
            .open(dir_fd, "test.txt", FdStat::default(), OpenFlags::CREATE, 0)
            .unwrap();

        let _fd2 = fs
            .open(dir_fd, "test2.txt", FdStat::default(), OpenFlags::CREATE, 0)
            .unwrap();
        let _fd3 = fs
            .open(dir_fd, "test3.txt", FdStat::default(), OpenFlags::CREATE, 0)
            .unwrap();
        let _fd4 = fs
            .open(dir_fd, "test4.txt", FdStat::default(), OpenFlags::CREATE, 0)
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

        assert_eq!(result, 0);
        assert_eq!(bytes_used, 99);
    }
}
