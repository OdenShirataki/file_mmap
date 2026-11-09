use file_mmap::FileMmap;

#[test]
fn write_out_of_range_is_rejected() {
    let dir = std::env::temp_dir().join("file_mmap-bounds-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut m = FileMmap::new(dir.join("b.data")).unwrap();
    assert_eq!(m.append(b"hello").unwrap(), 0);
    assert_eq!(m.len(), 5);

    assert!(m.write(0, b"HELLO").is_ok());
    assert!(m.write(3, b"LO").is_ok());
    assert!(m.write(3, b"LOX").is_err());
    assert!(m.write(-1, b"x").is_err());
    assert!(m.write(isize::MAX, b"x").is_err());
    assert!(m.write(1_000_000, b"").is_err());
    assert_eq!(unsafe { m.bytes(0, 5) }, b"HELLO");
}
