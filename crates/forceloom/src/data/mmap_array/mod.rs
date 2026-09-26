//! Temporary, raw-element arrays. No persistence format or importer dependencies.
mod array;
mod builder;
mod layout;
mod view;
pub use array::MmapArray2D;
pub use builder::MmapArrayBuilder;
pub use layout::{Layout2D, RowMajor};
pub use view::ArrayView2D;
