use libarchive3_sys::ffi;
use super::Entry;

pub struct OwnedEntry {
    handle: *mut ffi::Struct_archive_entry,
}

impl OwnedEntry {
    pub fn new() -> Option<Self> {
        unsafe { Self::from_raw( ffi::archive_entry_new() ) }
    }

    unsafe fn from_raw(p: *mut ffi::Struct_archive_entry) -> Option<Self> {
        p.as_mut().map(|p| OwnedEntry { handle: p } )
    }
}

impl Drop for OwnedEntry {
    fn drop(&mut self) {
        unsafe { ffi::archive_entry_free(self.handle); }
    }
}

impl Default for OwnedEntry {
    fn default() -> Self {
        Self::new().expect("Allocation error")
    }
}

impl Entry for OwnedEntry {
    unsafe fn entry(&self) -> *mut ffi::Struct_archive_entry {
        self.handle
    }
}
