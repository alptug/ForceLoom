use super::{ArrayView2D, Layout2D, RowMajor};
use bytemuck::Pod;
use memmap2::Mmap;
use std::{fs::File, marker::PhantomData};

/// Immutable temporary array. Shape and layout are held only in memory.
/// The backing file contains exactly the native-endian raw elements.
pub struct MmapArray2D<T: Pod, L: Layout2D = RowMajor> {
    pub(super) mmap: Option<Mmap>,
    pub(super) _file: File,
    pub(super) rows: usize,
    pub(super) cols: usize,
    pub(super) marker: PhantomData<(T, L)>,
}
impl<T: Pod, L: Layout2D> MmapArray2D<T, L> {
    pub fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }
    pub fn data(&self) -> &[T] {
        self.mmap.as_ref().map_or(&[], |m| bytemuck::cast_slice(m))
    }
    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        self.data().get(L::offset(self.rows, self.cols, row, col)?)
    }
    pub fn view(&self) -> ArrayView2D<'_, T, L> {
        ArrayView2D {
            data: self.data(),
            rows: self.rows,
            cols: self.cols,
            marker: PhantomData,
        }
    }
}
impl<T: Pod> MmapArray2D<T, RowMajor> {
    pub fn row(&self, row: usize) -> Option<&[T]> {
        self.view().row(row)
    }
}
