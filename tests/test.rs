use file_mmap::FileMmap;

#[test]
fn test() {
    let dir = std::env::temp_dir().join("file_mmap-test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.data");
    let _ = std::fs::remove_file(&path);

    let mut filemmap = FileMmap::new(&path).unwrap();
    assert_eq!(filemmap.append(b"test").unwrap(), 0);
    assert_eq!(filemmap.append(b"data").unwrap(), 4);
    drop(filemmap);

    assert_eq!(std::fs::read(&path).unwrap(), b"testdata");
}
