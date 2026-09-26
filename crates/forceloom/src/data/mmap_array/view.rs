use super::{Layout2D, RowMajor};
use bytemuck::Pod;
use std::marker::PhantomData;

/// Borrowed immutable view; cannot outlive its owning array.
pub struct ArrayView2D<'a, T: Pod, L: Layout2D = RowMajor> {
    pub(super) data: &'a [T],
    pub(super) rows: usize,
    pub(super) cols: usize,
    pub(super) marker: PhantomData<L>,
}
impl<'a, T: Pod, L: Layout2D> ArrayView2D<'a, T, L> {
    pub fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }
    pub fn data(&self) -> &'a [T] {
        self.data
    }
    pub fn get(&self, row: usize, col: usize) -> Option<&'a T> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        self.data.get(L::offset(self.rows, self.cols, row, col)?)
    }
}
impl<'a, T: Pod> ArrayView2D<'a, T, RowMajor> {
    pub fn row(&self, row: usize) -> Option<&'a [T]> {
        if row >= self.rows {
            return None;
        }
        let start = row.checked_mul(self.cols)?;
        self.data.get(start..start.checked_add(self.cols)?)
    }
}
