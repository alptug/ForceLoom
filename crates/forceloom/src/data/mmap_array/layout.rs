/// Maps logical coordinates into densely packed physical storage.
/// Implementations should return a unique offset below rows * cols for each
/// valid coordinate. All access remains bounds checked even for custom layouts.
pub trait Layout2D {
    fn offset(rows: usize, cols: usize, row: usize, col: usize) -> Option<usize>;
}

/// Contiguous rows, with columns varying fastest.
#[derive(Debug, Clone, Copy, Default)]
pub struct RowMajor;
impl Layout2D for RowMajor {
    fn offset(rows: usize, cols: usize, row: usize, col: usize) -> Option<usize> {
        if row >= rows || col >= cols {
            return None;
        }
        row.checked_mul(cols)?.checked_add(col)
    }
}
