use std::ptr;
use std::slice;

use libc::{off_t, size_t};
use libarchive3_sys::ffi;

use archive::Handle;
use entry::BorrowedEntry;
use error::{ArchiveResult, ArchiveError};
use super::ArchiveHandle;

#[deprecated(note="Use BorrowedEntry directly instead.")]
pub use entry::BorrowedEntry as ReaderEntry;

mod builder;
pub use self::builder::Builder;

mod file_reader;
pub use self::file_reader::FileReader;

mod stream_reader;
pub use self::stream_reader::StreamReader;

pub trait Reader : Handle {
    fn entry(&mut self) -> &mut BorrowedEntry;

    fn header_position(&self) -> i64 {
        unsafe { ffi::archive_read_header_position(self.handle()) }
    }

    fn next_header(&mut self) -> Option<&mut BorrowedEntry> {
        let res = unsafe { ffi::archive_read_next_header(self.handle(), &mut self.entry().handle) };
        if res == 0 {
            Some(self.entry())
        } else {
            None
        }
    }

    fn read(&self, buffer: &mut [u8]) -> ArchiveResult<size_t> {
        let ret_val = unsafe { ffi::archive_read_data(self.handle(), buffer.as_mut_ptr() as *mut _, buffer.len()) };
        if ret_val >= 0 {
            return Ok(ret_val as size_t);
        }

        Err(ArchiveError::Sys(self.err_code(), self.err_msg()))
    }

    fn read_all(&self) -> ArchiveResult<Vec<u8>> {
        const INCREMENT : usize = 65536;
        let mut buf = Vec::with_capacity(INCREMENT);
        loop {
            let len = buf.len();
            let mut cap = buf.capacity();
            if len >= cap {
                buf.reserve(len + INCREMENT);
                cap = buf.capacity();
            }

            let res = self.read(unsafe { buf.get_unchecked_mut(len..cap) })?;
            if 0 == res {
                break; //EOF
            }
            unsafe { buf.set_len(len + res) };
        };
        Ok(buf)
    }

    fn read_block(&self) -> ArchiveResult<Option<(&[u8], off_t)>> {
        let mut buff = ptr::null();
        let mut size = 0;
        let mut offset = 0;

        unsafe {
            match ffi::archive_read_data_block(self.handle(), &mut buff, &mut size, &mut offset) {
                ffi::ARCHIVE_EOF => Ok(None),
                ffi::ARCHIVE_OK => Ok(Some((slice::from_raw_parts(buff as *const u8, size), offset))),
                _ => Err(ArchiveError::Sys(self.err_code(), self.err_msg())),
            }
        }
    }
}

