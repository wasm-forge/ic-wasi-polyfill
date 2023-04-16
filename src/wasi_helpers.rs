use crate::wasi;



pub fn into_errno(error: stable_fs::error::Error) -> i32 {

    let errno = match error {
        stable_fs::error::Error::NotFound => wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidOffset =>  wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidFileType =>  wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidFileDescriptor =>  wasi::ERRNO_BADF,
        stable_fs::error::Error::InvalidBufferLength => wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidOpenFlags =>  wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidFdFlags =>  wasi::ERRNO_INVAL,
        stable_fs::error::Error::FileAlreadyExists => wasi::ERRNO_EXIST,
        stable_fs::error::Error::NameTooLong =>  wasi::ERRNO_NAMETOOLONG,
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

pub fn _into_stable_fs_filetype(file_type: wasi::Filetype) -> Result<stable_fs::storage::types::FileType, stable_fs::error::Error > {

    match file_type {
        wasi::FILETYPE_DIRECTORY => Ok(stable_fs::storage::types::FileType::Directory),
        wasi::FILETYPE_REGULAR_FILE => Ok(stable_fs::storage::types::FileType::RegularFile),
        wasi::FILETYPE_SYMBOLIC_LINK => Ok(stable_fs::storage::types::FileType::SymbolicLink),
        _ => Err(stable_fs::error::Error::InvalidFileType),
     }
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