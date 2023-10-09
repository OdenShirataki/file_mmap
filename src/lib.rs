use std::{
    fs,
    io::{self, Write},
    mem::ManuallyDrop,
    ops::Deref,
    path::Path,
};

use memmap2::MmapRaw;
use once_cell::sync::Lazy;

static PAGE_SIZE: Lazy<usize> = Lazy::new(|| sysconf::page::pagesize());

pub struct FileMmap {
    file: fs::File,
    mmap: ManuallyDrop<MmapRaw>,
}

impl Drop for FileMmap {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.mmap) };
    }
}

impl Deref for FileMmap {
    type Target = MmapRaw;

    fn deref(&self) -> &Self::Target {
        &self.mmap
    }
}

impl FileMmap {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let mmap = ManuallyDrop::new(MmapRaw::map_raw(&file)?);
        Ok(FileMmap { file, mmap })
    }

    #[inline(always)]
    pub fn len(&self) -> u64 {
        self.file.metadata().unwrap().len()
    }

    #[inline(always)]
    pub unsafe fn bytes(&self, addr: isize, len: usize) -> &'static [u8] {
        std::slice::from_raw_parts(self.as_ptr().offset(addr), len)
    }

    #[inline(always)]
    pub fn set_len(&mut self, len: u64) -> io::Result<()> {
        let current_len = self.file.metadata()?.len();
        if current_len > len
            || current_len == 0
            || ((current_len as usize - 1) / *PAGE_SIZE != len as usize / *PAGE_SIZE)
        {
            unsafe { ManuallyDrop::drop(&mut self.mmap) };
            self.file.set_len(len)?;
            self.mmap = ManuallyDrop::new(MmapRaw::map_raw(&self.file)?);
            Ok(())
        } else {
            self.file.set_len(len)
        }
    }

    #[inline(always)]
    pub fn append(&mut self, bytes: &[u8]) -> io::Result<u64> {
        let addr = self.file.metadata()?.len();
        self.set_len(addr + bytes.len() as u64)?;
        self.write(addr as isize, bytes)?;
        Ok(addr)
    }

    #[inline(always)]
    pub fn write(&mut self, addr: isize, bytes: &[u8]) -> io::Result<()> {
        let mut memory =
            unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr().offset(addr), bytes.len()) };
        memory.write_all(bytes)
    }
}
