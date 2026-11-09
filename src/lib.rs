use std::{
    fs,
    io::{self, Write},
    mem::ManuallyDrop,
    ops::Deref,
    path::Path,
};

use memmap2::MmapRaw;

/// Lower bound of the OS page size, used to skip remapping when the file grows
/// within the page that is already mapped.
///
/// This only has to be less than or equal to the real page size: a smaller value
/// just causes extra remaps, never a missing one. 4 KiB is the smallest page size
/// on mainstream platforms (x86 is 4 KiB, Apple Silicon 16 KiB, some ARM 64 KiB).
const PAGE_SIZE: usize = 4096;

pub struct FileMmap {
    file: fs::File,
    mmap: ManuallyDrop<MmapRaw>,
    len: u64,
}

impl Drop for FileMmap {
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
    /// Opens the file and creates the mmap.
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        let len = file.metadata()?.len();
        let mmap = ManuallyDrop::new(MmapRaw::map_raw(&file)?);
        Ok(FileMmap { file, mmap, len })
    }

    /// Gets the length of the file.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns true if the file is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Gets a byte slice from the mmap'd region.
    ///
    /// # Safety
    /// Make sure addr+len does not exceed the size of the file.
    pub unsafe fn bytes(&self, addr: isize, len: usize) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr().offset(addr), len) }
    }

    /// Sets the length of the file.
    pub fn set_len(&mut self, len: u64) -> io::Result<()> {
        let current_len = self.len;
        if current_len > len
            || current_len == 0
            || ((current_len as usize - 1) / PAGE_SIZE != len as usize / PAGE_SIZE)
        {
            unsafe { ManuallyDrop::drop(&mut self.mmap) };
            self.file.set_len(len)?;
            self.len = len;
            self.mmap = ManuallyDrop::new(MmapRaw::map_raw(&self.file)?);
            Ok(())
        } else {
            self.file.set_len(len)?;
            self.len = len;
            Ok(())
        }
    }

    /// Appends a byte slice to the end of the file.
    pub fn append(&mut self, bytes: &[u8]) -> io::Result<u64> {
        let addr = self.len;
        self.set_len(addr + bytes.len() as u64)?;
        // The file was just extended to cover addr..addr+bytes.len().
        unsafe { self.write_unchecked(addr as isize, bytes) }?;
        Ok(addr)
    }

    /// Writes a byte slice to the mmap'd region.
    ///
    /// Returns an [io::ErrorKind::InvalidInput] error if `addr` is negative
    /// or `addr + bytes.len()` exceeds the length of the file.
    pub fn write(&mut self, addr: isize, bytes: &[u8]) -> io::Result<()> {
        let in_range = u64::try_from(addr)
            .ok()
            .and_then(|addr| addr.checked_add(bytes.len() as u64))
            .is_some_and(|end| end <= self.len);
        if !in_range {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "write out of range of the file",
            ));
        }
        unsafe { self.write_unchecked(addr, bytes) }
    }

    /// Writes a byte slice to the mmap'd region without bounds checking.
    ///
    /// # Safety
    /// `addr` must be non-negative and `addr + bytes.len()` must not exceed the length of the file.
    pub unsafe fn write_unchecked(&mut self, addr: isize, bytes: &[u8]) -> io::Result<()> {
        unsafe {
            let mut memory =
                std::slice::from_raw_parts_mut(self.as_mut_ptr().offset(addr), bytes.len());
            memory.write_all(bytes)
        }
    }
}
