use std::{
    fs::{OpenOptions,File}
    ,io::{Seek,SeekFrom}
};

use memmap2::MmapMut;

pub struct FileMmap{
    file:File
    ,mmap:MmapMut
    ,len:u64
}

impl FileMmap{
    pub fn new(path:&str,inital_size:u64)->Result<Self,std::io::Error>{
        let mut file=OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?
        ;
        let mut len=file.metadata()?.len();
        if len==0{
            file.set_len(if inital_size==0{
                1   //If the size is 0, it seems to fail, so if it is 0, specify 1 byte for the time being
            }else{
                inital_size
            })?;
            len=inital_size;
        }
        file.seek(SeekFrom::Start(len))?;
        let mmap=unsafe{
            MmapMut::map_mut(&file)?
        };
        Ok(FileMmap{
            file
            ,mmap
            ,len
        })
    }
    pub fn len(&self)->u64{
        self.len
    }
    pub fn as_ptr(&self)->*const i64{
        self.mmap.as_ptr() as *const i64
    }
    pub fn offset(&self,addr:isize)->*const i8{
        unsafe{
            self.mmap.as_ptr().offset(addr) as *const i8
        }
    }
    pub fn bytes(&self,addr:isize,len:usize)->&[u8]{
        unsafe{
            std::slice::from_raw_parts(self.mmap.as_ptr().offset(addr),len)
        }
    }
    pub fn set_len(&mut self,len:u64)->std::io::Result<()>{
        self.len=len;
        self.file.set_len(len)
    }
    pub fn append(&mut self,bytes:&[u8])->Result<u64,std::io::Error>{
        let addr=self.len;
        self.set_len(self.len+bytes.len() as u64)?;
        self.write(addr,bytes);
        Ok(addr)
    }
    pub fn write(&mut self,addr:u64,bytes:&[u8]){
        let len=bytes.len();
        unsafe{
            std::ptr::copy(
                bytes.as_ptr()
                ,self.mmap.as_ptr().offset(addr as isize) as *mut u8
                ,len
            );
        }
    }
    pub fn write_0(&mut self,addr:isize,len:u64){
        unsafe{
            std::ptr::write_bytes(
                self.mmap.as_ptr().offset(addr) as *mut u8
                ,0
                ,len as usize
            );
        }
    }
}