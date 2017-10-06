use std::default::Default;
use std::ptr;
use std::slice;

use libc::off_t;
use libarchive3_sys::ffi;

use archive::{Entry, Handle};
use error::{ArchiveResult, ArchiveError};
use super::ArchiveHandle;

mod builder;
pub use self::builder::Builder;

mod file_reader;
pub use self::file_reader::FileReader;

mod stream_reader;
pub use self::stream_reader::StreamReader;

pub trait Reader : Handle {
    fn entry(&mut self) -> &mut ReaderEntry;

    fn header_position(&self) -> i64 {
        unsafe { ffi::archive_read_header_position(self.handle()) }
    }

    fn next_header(&mut self) -> Option<&mut ReaderEntry> {
        let res = unsafe { ffi::archive_read_next_header(self.handle(), &mut self.entry().handle) };
        if res == 0 {
            Some(self.entry())
        } else {
            None
        }
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

pub struct ReaderEntry {
    handle: *mut ffi::Struct_archive_entry,
}

impl ReaderEntry {
    pub fn new(handle: *mut ffi::Struct_archive_entry) -> Self {
        ReaderEntry { handle: handle }
    }
}

impl Default for ReaderEntry {
    fn default() -> Self {
        ReaderEntry { handle: ptr::null_mut() }
    }
}

impl Entry for ReaderEntry {
    unsafe fn entry(&self) -> *mut ffi::Struct_archive_entry {
        self.handle
    }
}
