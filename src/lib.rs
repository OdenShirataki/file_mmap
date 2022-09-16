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
        match OpenOptions::new()
           .read(true)
           .write(true)
           .create(true)
           .open(&path)
        {
            Ok(mut file)=>{
                if len==0{
                    if let Err(e)=file.set_len(if initial_size==0{
                        1
                    }else{
                        initial_size
                    }){   //サイズが0の場合失敗するようなので0の場合はとりあえず1バイト指定しておく
                        return Err(e);
                    }
                    if let Err(e)=file.seek(std::io::SeekFrom::Start(initial_size)){
                        return Err(e);
                    }
                    len=initial_size;
                }else{
                    file.seek(std::io::SeekFrom::Start(len))?;
                }
                let mmap = unsafe {
                    MmapOptions::new().map_mut(&file).unwrap()
                };
                Ok(FileMmap{
                    file
                    ,mmap
                    ,len
                })
            }
            ,Err(e)=>{
                Err(e)
            }
        }
    }
    pub fn len(&self)->u64{
        self.len
    }
    pub fn as_ptr(&self)->*mut i64{
        self.mmap.as_ptr() as *mut i64
    }
    pub fn as_mut_ptr(&mut self)->*mut i64{
        self.mmap.as_mut_ptr() as *mut i64
    }
    pub fn offset(&self,addr:isize)->*const i8{
        unsafe{
            self.mmap.as_ptr().offset(addr) as *const i8
        }
    }
    pub fn set_len(&mut self,len:u64)->std::io::Result<()>{
        self.len=len;
        match self.file.set_len(len){
            Err(e)=>{
                Err(e)
            }
            ,Ok(())=>{
                Ok(())
            }
        }
    }
    pub fn append(&mut self,bytes:&[u8])->Option<u64>{
        let addr=self.len;
        if let Ok(_)=self.set_len(self.len+bytes.len() as u64 + 1){
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