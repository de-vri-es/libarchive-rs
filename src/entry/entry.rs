use archive::FileType;

use std::ffi::{CStr, CString};
use std::path::PathBuf;
use std::str;

use libarchive3_sys::ffi;

pub trait Entry {
    unsafe fn entry(&self) -> *mut ffi::Struct_archive_entry;

    fn filetype(&self) -> FileType {
        unsafe {
            match ffi::archive_entry_filetype(self.entry()) as u32 {
                ffi::AE_IFBLK => FileType::BlockDevice,
                ffi::AE_IFCHR => FileType::CharacterDevice,
                ffi::AE_IFLNK => FileType::SymbolicLink,
                ffi::AE_IFDIR => FileType::Directory,
                ffi::AE_IFIFO => FileType::NamedPipe,
                ffi::AE_IFMT => FileType::Mount,
                ffi::AE_IFREG => FileType::RegularFile,
                ffi::AE_IFSOCK => FileType::Socket,
                0 => FileType::Unknown,
                code => unreachable!("undefined filetype: {}", code),
            }
        }
    }

    fn hardlink_raw(&self) -> Option<&[u8]> {
        let c_str: &CStr = unsafe {
            let ptr = ffi::archive_entry_hardlink(self.entry());
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr)
        };
        let buf: &[u8] = c_str.to_bytes();
        Some(buf)
    }

    fn hardlink(&self) -> Option<&str> {
        self.hardlink_raw().map(|buf| str::from_utf8(buf).unwrap())
    }

    fn pathname_raw(&self) -> &[u8] {
        let c_str: &CStr = unsafe { CStr::from_ptr(ffi::archive_entry_pathname(self.entry())) };
        let buf: &[u8] = c_str.to_bytes();
        buf
    }

    fn pathname(&self) -> &str {
        str::from_utf8(self.pathname_raw()).unwrap()
    }

    fn size(&self) -> i64 {
        unsafe { ffi::archive_entry_size(self.entry()) }
    }

    fn symlink_raw(&self) -> Option<&[u8]> {
        let c_str: &CStr = unsafe {
            let ptr = ffi::archive_entry_symlink(self.entry());
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr)
        };
        let buf: &[u8] = c_str.to_bytes();
        Some(buf)
    }

    fn symlink(&self) -> Option<&str> {
        self.symlink_raw().map(|buf| str::from_utf8(buf).unwrap())
    }

    fn set_filetype(&mut self, file_type: FileType) {
        unsafe {
            let file_type = match file_type {
                FileType::BlockDevice => ffi::AE_IFBLK,
                FileType::CharacterDevice => ffi::AE_IFCHR,
                FileType::SymbolicLink => ffi::AE_IFLNK,
                FileType::Directory => ffi::AE_IFDIR,
                FileType::NamedPipe => ffi::AE_IFIFO,
                FileType::Mount => ffi::AE_IFMT,
                FileType::RegularFile => ffi::AE_IFREG,
                FileType::Socket => ffi::AE_IFSOCK,
                FileType::Unknown => 0,
            };
            ffi::archive_entry_set_filetype(self.entry(), file_type);
        }
    }

    fn set_link(&mut self, path: &PathBuf) {
        unsafe {
            let c_str = CString::new(path.to_str().unwrap()).unwrap();
            ffi::archive_entry_set_link(self.entry(), c_str.as_ptr());
        }
    }

    fn set_pathname(&mut self, path: &PathBuf) {
        unsafe {
            let c_str = CString::new(path.to_str().unwrap()).unwrap();
            ffi::archive_entry_set_pathname(self.entry(), c_str.as_ptr());
        }
    }
}

pub fn entry_debug_fmt<E: Entry>(struct_name: &str, e: &E, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    f.debug_struct(struct_name)
        .field("type", &e.filetype())
        .field("pathname", &String::from_utf8_lossy(e.pathname_raw()))
        .finish()
}
