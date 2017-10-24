use std::default::Default;
use std::error::Error;
use std::ffi::CString;
use std::io::{self, Read};

use libc::{c_void, ssize_t};
use libarchive3_sys::ffi;

use archive::{ArchiveHandle, Handle};
use entry::BorrowedEntry;
use error::{ArchiveResult, ArchiveError};
use super::{Builder, Reader};

pub struct StreamReader<T> {
    handle: ArchiveHandle,
    entry: BorrowedEntry,
    _pipe: Box<Pipe<T>>,
}

struct Pipe<T> {
    reader: T,
    buffer: Vec<u8>,
}

impl<T> Pipe<T> {
    fn new(src: T) -> Self {
        Pipe {
            reader: src,
            buffer: vec![0; 8192],
        }
    }

    fn read_bytes(&mut self) -> io::Result<usize> where T: Read {
        self.reader.read(&mut self.buffer[..])
    }
}

impl<T> StreamReader<T> {
    pub fn open(builder: Builder, src: T) -> ArchiveResult<Self> where T: Read {
        unsafe {
            let mut pipe = Box::new(Pipe::new(src));
            let pipe_ptr: *mut c_void = &mut *pipe as *mut Pipe<T> as *mut c_void;
            match ffi::archive_read_open(builder.handle(),
                                         pipe_ptr,
                                         None,
                                         Some(stream_read_callback::<T>),
                                         None) {
                ffi::ARCHIVE_OK => {
                    let reader = StreamReader {
                        handle: builder.into(),
                        entry: BorrowedEntry::default(),
                        _pipe: pipe,
                    };
                    Ok(reader)
                }
                _ => {
                    Err(ArchiveError::from(&builder as &Handle))
                }
            }
        }
    }
}

impl<T> Handle for StreamReader<T> {
    unsafe fn handle(&self) -> &mut ffi::Struct_archive {
        self.handle.handle()
    }
}

impl<T> Reader for StreamReader<T> {
    fn entry(&mut self) -> &mut BorrowedEntry {
        &mut self.entry
    }
}

unsafe extern "C" fn stream_read_callback<T: Read>(handle: *mut ffi::Struct_archive,
                                                   data: *mut c_void,
                                                   buff: *mut *const c_void)
                                                   -> ssize_t {
    let pipe: &mut Pipe<T> = &mut *(data as *mut Pipe<T>);
    *buff = pipe.buffer.as_mut_ptr() as *mut c_void;
    match pipe.read_bytes() {
        Ok(size) => size as ssize_t,
        Err(e) => {
            let desc = CString::new(e.description()).unwrap();
            ffi::archive_set_error(handle, e.raw_os_error().unwrap_or(0), desc.as_ptr());
            -1 as ssize_t
        }
    }
}
