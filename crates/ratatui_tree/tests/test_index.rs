use ratatui_tree::index2::TreeIndex;

#[test]
fn test_new() {
    assert!(TreeIndex::new(&[]).is_none());
    assert!(TreeIndex::new(&[0]).is_some());
    assert!(TreeIndex::new(&[0, 1]).is_some());
}

#[test]
fn test_first() {
    let index = TreeIndex::new(&[0]).unwrap();
    assert_eq!(index.first(), 0);
    let index = TreeIndex::new(&[0, 1]).unwrap();
    assert_eq!(index.first(), 0);
}

#[test]
fn test_last() {
    let index = TreeIndex::new(&[0]).unwrap();
    assert_eq!(index.last(), 0);
    let index = TreeIndex::new(&[0, 1]).unwrap();
    assert_eq!(index.last(), 1);
}
