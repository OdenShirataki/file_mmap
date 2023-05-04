use std::{fs, io, mem::ManuallyDrop, path::Path};

use file_offset::FileExt;
use memmap2::MmapRaw;
use once_cell::sync::Lazy;

static PAGE_SIZE: Lazy<usize> = Lazy::new(|| sysconf::page::pagesize());

pub struct FileMmap {
    file: fs::File,
    mmap: ManuallyDrop<Box<MmapRaw>>,
}

impl Drop for FileMmap {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.mmap) };
    }
}

impl FileMmap {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let mmap = ManuallyDrop::new(Box::new(MmapRaw::map_raw(&file)?));
        Ok(FileMmap { file, mmap })
    }
    pub fn len(&self) -> io::Result<u64> {
        Ok(self.file.metadata()?.len())
    }
    pub fn as_ptr(&self) -> *const u8 {
        self.mmap.as_ptr()
    }
    pub unsafe fn offset(&self, addr: isize) -> *const u8 {
        self.mmap.as_ptr().offset(addr)
    }
    pub unsafe fn bytes(&self, addr: isize, len: usize) -> &'static [u8] {
        std::slice::from_raw_parts(self.mmap.as_ptr().offset(addr), len)
    }
    pub fn set_len(&mut self, len: u64) -> io::Result<()> {
        let current_len = self.file.metadata()?.len();
        if current_len > len
            || current_len == 0
            || (current_len as usize / *PAGE_SIZE != len as usize / *PAGE_SIZE)
        {
            unsafe { ManuallyDrop::drop(&mut self.mmap) };
            self.file.set_len(len)?;
            self.mmap = ManuallyDrop::new(Box::new(MmapRaw::map_raw(&self.file)?));
            Ok(())
        } else {
            self.file.set_len(len)
        }
    }
    pub fn append(&mut self, bytes: &[u8]) -> io::Result<u64> {
        unsafe { ManuallyDrop::drop(&mut self.mmap) };
        let addr = self.file.metadata()?.len();
        self.file.set_len(addr + bytes.len() as u64)?;
        self.file.write_offset(bytes, addr)?;
        self.mmap = ManuallyDrop::new(Box::new(MmapRaw::map_raw(&self.file)?));
        Ok(addr)
    }
    pub fn write(&mut self, addr: isize, bytes: &[u8]) -> io::Result<usize> {
        self.file.write_offset(bytes, addr as u64)
    }
    pub fn write_0(&mut self, addr: isize, len: usize) -> io::Result<usize> {
        self.write(addr, &vec![0; len])
    }
}
