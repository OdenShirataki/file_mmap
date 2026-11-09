use file_mmap::FileMmap;

/// Appends small chunks across many page boundaries, so both the remap path
/// and the grow-within-the-same-page path are exercised, then checks the file
/// content after reopening.
#[test]
fn append_across_page_boundaries() {
    let dir = std::env::temp_dir().join("file_mmap-grow-test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("grow.data");
    let _ = std::fs::remove_file(&path);

    let mut expected = Vec::new();
    {
        let mut m = FileMmap::new(&path).unwrap();
        for i in 0..5000u32 {
            let chunk = format!("{i:07}|");
            let addr = m.append(chunk.as_bytes()).unwrap();
            assert_eq!(addr as usize, expected.len());
            expected.extend_from_slice(chunk.as_bytes());
            assert_eq!(
                unsafe { m.bytes(addr as isize, chunk.len()) },
                chunk.as_bytes()
            );
        }
        assert_eq!(m.len() as usize, expected.len());
    }
    assert_eq!(std::fs::read(&path).unwrap(), expected);

    let m = FileMmap::new(&path).unwrap();
    assert_eq!(m.len() as usize, expected.len());
    assert_eq!(unsafe { m.bytes(0, expected.len()) }, &expected[..]);
}
