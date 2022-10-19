use memmap2::*;
use std::{
    fs::{OpenOptions,File,metadata}
    ,io::{prelude::*}
};

pub struct FileMmap{
    file:File
    ,mmap:MmapMut
    ,len:u64
}

impl FileMmap{
    pub fn new(path:&str,initial_size:u64) -> Result<FileMmap,std::io::Error>{
        let mut len = match metadata(&path){
            Ok(md)=>md.len()
            ,Err(_)=>0
        };
        let mut file=OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?
        ;
        if len==0{
            file.set_len(if initial_size==0{
                1   //サイズが0の場合失敗するようなので0の場合はとりあえず1バイト指定しておく
            }else{
                initial_size
            })?;
            file.seek(std::io::SeekFrom::Start(initial_size))?;
            len=initial_size;
        }else{
            file.seek(std::io::SeekFrom::Start(len))?;
        }
        let mmap = unsafe {
            MmapOptions::new().map_mut(&file)?
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
    pub fn append(&mut self,bytes:&[u8])->Option<u64>{
        let addr=self.len;
        if let Ok(_)=self.set_len(self.len+bytes.len() as u64){
            self.write(addr,bytes);
            Some(addr)
        }else{
            None
        }
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
        self.write_0(addr as isize+len as isize,1);
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