use std::default::Default;
use std::error::Error;
use std::ffi::CString;
use std::io::{self, Read};

use libc::{c_void, ssize_t};
use libarchive3_sys::ffi;

use archive::Handle;
use entry::BorrowedEntry;
use error::{ArchiveResult, ArchiveError};
use super::{ArchiveHandle, Builder, Reader};

pub struct StreamReader {
    handle: ArchiveHandle,
    entry: BorrowedEntry,
    _pipe: Box<Pipe>,
}

struct Pipe {
    reader: Box<Read>,
    buffer: Vec<u8>,
}

impl Pipe {
    fn new<T: 'static + Read>(src: T) -> Self {
        Pipe {
            reader: Box::new(src),
            buffer: vec![0; 8192],
        }
    }

    fn read_bytes(&mut self) -> io::Result<usize> {
        self.reader.read(&mut self.buffer[..])
    }
}

impl StreamReader {
    pub fn open<T: 'static + Read>(builder: Builder, src: T) -> ArchiveResult<Self> {
        unsafe {
            let mut pipe = Box::new(Pipe::new(src));
            let pipe_ptr: *mut c_void = &mut *pipe as *mut Pipe as *mut c_void;
            match ffi::archive_read_open(builder.handle(),
                                         pipe_ptr,
                                         None,
                                         Some(stream_read_callback),
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

impl Handle for StreamReader {
    unsafe fn handle(&self) -> &mut ffi::Struct_archive {
        self.handle.handle()
    }
}

impl Reader for StreamReader {
    fn entry(&mut self) -> &mut BorrowedEntry {
        &mut self.entry
    }
}

unsafe extern "C" fn stream_read_callback(handle: *mut ffi::Struct_archive,
                                          data: *mut c_void,
                                          buff: *mut *const c_void)
                                          -> ssize_t {
    let pipe: &mut Pipe = &mut *(data as *mut Pipe);
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
