extern crate libc;
extern crate libarchive3_sys;

pub mod archive;
pub mod error;
pub mod reader;
pub mod writer;

use libarchive3_sys::ffi;

pub trait ArchiveHandleKind {
    unsafe fn free_handle(*mut ffi::Struct_archive);
}

pub struct ArchiveKindRead;
pub struct ArchiveKindWrite;

impl ArchiveHandleKind for ArchiveKindRead {
    unsafe fn free_handle(handle: *mut ffi::Struct_archive) {
        ffi::archive_read_free(handle);
    }
}

impl ArchiveHandleKind for ArchiveKindWrite {
    unsafe fn free_handle(handle: *mut ffi::Struct_archive) {
        ffi::archive_write_free(handle);
    }
}

pub struct ArchiveHandle<K: ArchiveHandleKind> {
    handle: *mut ffi::Struct_archive,
    _kind: ::std::marker::PhantomData<K>
}

impl<K: ArchiveHandleKind> ArchiveHandle<K> {
    unsafe fn from_raw(handle: *mut ffi::Struct_archive) -> Option<Self> {
        handle.as_mut().map(|handle| ArchiveHandle { handle: handle, _kind: ::std::marker::PhantomData } )
    }
}

impl<K: ArchiveHandleKind> Drop for ArchiveHandle<K> {
    fn drop(&mut self) {
        unsafe { K::free_handle(self.handle) }
    }
}

impl<K: ArchiveHandleKind> archive::Handle for ArchiveHandle<K> {
    unsafe fn handle(&self) -> &mut ffi::Struct_archive {
        &mut *self.handle
    }
}
