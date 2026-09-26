use forceloom::data::mmap_array::{Layout2D, MmapArrayBuilder};

#[test]
fn build_freeze_and_view() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let mut b = MmapArrayBuilder::<f64>::new_in(2, 3, dir.path())?;
    assert_eq!(b.data_mut(), &[0.0; 6]);
    b.row_mut(0).unwrap().copy_from_slice(&[1., 2., 3.]);
    *b.get_mut(1, 2).unwrap() = 9.;
    assert!(b.get_mut(2, 0).is_none());
    assert!(b.row_mut(2).is_none());
    let a = b.finish()?;
    assert_eq!(a.shape(), (2, 3));
    assert_eq!(a.row(0), Some([1., 2., 3.].as_slice()));
    assert_eq!(a.get(1, 2), Some(&9.));
    assert_eq!(a.view().get(1, 2), Some(&9.));
    assert!(a.get(0, 3).is_none());
    assert!(a.view().row(2).is_none());
    std::thread::scope(|s| {
        s.spawn(|| assert_eq!(a.get(1, 2), Some(&9.)));
    });
    drop(a);
    assert_eq!(std::fs::read_dir(dir.path())?.count(), 0);
    Ok(())
}
#[test]
fn empty_and_invalid_sizes() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    for shape in [(0, 3), (3, 0), (0, 0)] {
        let mut b = MmapArrayBuilder::<u8>::new_in(shape.0, shape.1, dir.path())?;
        assert!(b.data_mut().is_empty());
        let a = b.finish()?;
        assert_eq!(a.shape(), shape);
        assert!(a.data().is_empty());
        assert!(a.get(0, 0).is_none());
    }
    assert!(MmapArrayBuilder::<u64>::new_in(usize::MAX, 2, dir.path()).is_err());
    assert!(MmapArrayBuilder::<()>::new_in(1, 1, dir.path()).is_err());
    assert!(MmapArrayBuilder::<u8>::new_in(isize::MAX as usize + 1, 1, dir.path()).is_err());
    Ok(())
}
struct ColumnMajor;
impl Layout2D for ColumnMajor {
    fn offset(rows: usize, _: usize, row: usize, col: usize) -> Option<usize> {
        col.checked_mul(rows)?.checked_add(row)
    }
}
#[test]
fn custom_layout() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let mut b = MmapArrayBuilder::<u32, ColumnMajor>::new_in(2, 3, dir.path())?;
    *b.get_mut(1, 2).unwrap() = 42;
    let a = b.finish()?;
    assert_eq!(a.data(), &[0, 0, 0, 0, 0, 42]);
    assert_eq!(a.view().get(1, 2), Some(&42));
    assert!(a.get(2, 0).is_none());
    Ok(())
}
