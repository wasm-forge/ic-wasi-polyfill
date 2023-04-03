use crate::wasi;



pub fn into_errno(error: stable_fs::error::Error) -> i32 {


    let errno = match error {
        stable_fs::error::Error::NotFound => wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidOffset =>  wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidFileType =>  wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidFileDescriptor =>  wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidBufferLength => wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidOpenFlags =>  wasi::ERRNO_INVAL,
        stable_fs::error::Error::InvalidFdFlags =>  wasi::ERRNO_INVAL,
        stable_fs::error::Error::FileAlreadyExists => wasi::ERRNO_EXIST,
        stable_fs::error::Error::NameTooLong =>  wasi::ERRNO_NAMETOOLONG,
    };

    errno.raw() as i32
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