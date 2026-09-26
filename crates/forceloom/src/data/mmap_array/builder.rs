use super::{Layout2D, MmapArray2D, RowMajor};
use bytemuck::Pod;
use directories::ProjectDirs;
use memmap2::MmapMut;
use std::{
    fs::{self, File},
    io,
    marker::PhantomData,
    mem::size_of,
    path::Path,
};

/// Zero-initialized, mutable construction phase. Consumed by `finish`.
/// Zero-sized element types are rejected; zero-length arrays are supported.
pub struct MmapArrayBuilder<T: Pod, L: Layout2D = RowMajor> {
    // Declaration order ensures the mapping is dropped before the file.
    mmap: Option<MmapMut>,
    file: File,
    rows: usize,
    cols: usize,
    marker: PhantomData<(T, L)>,
}
impl<T: Pod, L: Layout2D> MmapArrayBuilder<T, L> {
    /// Uses the OS-appropriate ForceLoom application cache directory.
    pub fn new(rows: usize, cols: usize) -> io::Result<Self> {
        let dirs = ProjectDirs::from("org", "ForceLoom", "ForceLoom")
            .ok_or_else(|| io::Error::other("Cannot determine ForceLoom cache directory"))?;
        Self::new_in(rows, cols, dirs.cache_dir())
    }
    /// Uses a chosen scratch directory, creating it if necessary.
    pub fn new_in(rows: usize, cols: usize, directory: impl AsRef<Path>) -> io::Result<Self> {
        let invalid = || {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported element type or array size",
            )
        };
        if size_of::<T>() == 0 {
            return Err(invalid());
        }
        let bytes = rows
            .checked_mul(cols)
            .and_then(|n| n.checked_mul(size_of::<T>()))
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or_else(invalid)?;
        let length = u64::try_from(bytes).map_err(|_| invalid())?;
        fs::create_dir_all(directory.as_ref())?;
        let file = tempfile::tempfile_in(directory)?;
        file.set_len(length)?;
        let mmap = if bytes == 0 {
            None
        } else {
            // SAFETY: This fresh temporary file is exclusively owned here. Its
            // handle/path is never exposed, cloned, resized, or otherwise written
            // while mapped. Mutation occurs only through exclusive builder borrows.
            // Extending a fresh file provides initialized zero bytes; Pod accepts
            // every bit pattern. The file is retained until after unmapping.
            let mmap = unsafe { MmapMut::map_mut(&file)? };
            bytemuck::try_cast_slice::<u8, T>(&mmap).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Mapping cannot satisfy element alignment",
                )
            })?;
            Some(mmap)
        };
        Ok(Self {
            mmap,
            file,
            rows,
            cols,
            marker: PhantomData,
        })
    }
    pub fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }
    /// Elements in physical layout order.
    pub fn data_mut(&mut self) -> &mut [T] {
        match &mut self.mmap {
            Some(m) => bytemuck::cast_slice_mut(m),
            None => &mut [],
        }
    }
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut T> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        let offset = L::offset(self.rows, self.cols, row, col)?;
        self.data_mut().get_mut(offset)
    }
    /// Changes mapping protection to read-only without copying the array.
    /// This is temporary storage, not a durability or persistence operation.
    pub fn finish(self) -> io::Result<MmapArray2D<T, L>> {
        Ok(MmapArray2D {
            mmap: self.mmap.map(MmapMut::make_read_only).transpose()?,
            _file: self.file,
            rows: self.rows,
            cols: self.cols,
            marker: PhantomData,
        })
    }
}
impl<T: Pod> MmapArrayBuilder<T, RowMajor> {
    pub fn row_mut(&mut self, row: usize) -> Option<&mut [T]> {
        if row >= self.rows {
            return None;
        }
        let start = row.checked_mul(self.cols)?;
        let end = start.checked_add(self.cols)?;
        self.data_mut().get_mut(start..end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backing_file_contains_only_elements() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let mut b = MmapArrayBuilder::<u32>::new_in(2, 3, dir.path())?;
        b.data_mut().copy_from_slice(&[1, 2, 3, 4, 5, 6]);
        assert_eq!(b.file.metadata()?.len(), 24);
        b.mmap.as_ref().unwrap().flush()?;
        use std::io::{Read, Seek, SeekFrom};
        b.file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        b.file.read_to_end(&mut bytes)?;
        assert_eq!(bytes, bytemuck::cast_slice::<u32, u8>(&[1, 2, 3, 4, 5, 6]));
        Ok(())
    }
}
