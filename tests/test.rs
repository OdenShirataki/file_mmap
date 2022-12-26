use file_mmap::FileMmap;

#[test]
fn test(){
    let path="D:\\test.data";
    let mut filemmap=FileMmap::new(path).unwrap();
    filemmap.append(b"test").unwrap();
}