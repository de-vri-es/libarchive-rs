use super::Entry;
use libarchive3_sys::ffi;
use std::ptr;

pub struct BorrowedEntry {
    pub(crate) handle: *mut ffi::archive_entry,
}

impl BorrowedEntry {
    pub fn new(handle: *mut ffi::archive_entry) -> Self {
        BorrowedEntry { handle: handle }
    }
}

impl Default for BorrowedEntry {
    fn default() -> Self {
        BorrowedEntry {
            handle: ptr::null_mut(),
        }
    }
}

impl Entry for BorrowedEntry {
    unsafe fn entry(&self) -> *mut ffi::archive_entry {
        self.handle
    }
}

impl ::std::fmt::Debug for BorrowedEntry {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        super::entry::entry_debug_fmt("BorrowedEntry", self, f)
    }
}
