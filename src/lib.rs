use memmap2::MmapRaw;
use std::{
    fs,
    io::{self, Seek},
    mem::ManuallyDrop,
    ptr,
};

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
    pub fn new(path: &str) -> io::Result<Self> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        file.seek(io::SeekFrom::End(0))?;
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
    pub unsafe fn bytes(&self, addr: isize, len: usize) -> &[u8] {
        std::slice::from_raw_parts(self.mmap.as_ptr().offset(addr), len)
    }
    pub fn set_len(&mut self, len: u64) -> io::Result<()> {
        let current_len = self.file.metadata()?.len();
        if len > current_len && current_len > 0 {
            self.file.set_len(len)?;
        } else {
            unsafe { ManuallyDrop::drop(&mut self.mmap) };
            self.file.set_len(len)?;
            self.mmap = ManuallyDrop::new(Box::new(MmapRaw::map_raw(&self.file).unwrap()));
        }
        Ok(())
    }
    pub fn append(&mut self, bytes: &[u8]) -> io::Result<u64> {
        let addr = self.file.metadata()?.len();
        self.set_len(addr + bytes.len() as u64)?;
        unsafe {
            self.write(addr as isize, bytes);
        }
        Ok(addr)
    }
    pub unsafe fn write(&mut self, addr: isize, bytes: &[u8]) {
        let len = bytes.len();
        ptr::copy(
            bytes.as_ptr(),
            self.mmap.as_ptr().offset(addr) as *mut u8,
            len,
        );
    }
    pub unsafe fn write_0(&mut self, addr: isize, len: usize) {
        ptr::write_bytes(self.mmap.as_ptr().offset(addr) as *mut u8, 0, len);
    }
}
