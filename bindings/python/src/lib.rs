use pyo3::prelude::*;

/// ForceLoom's initial extension surface. Array APIs currently live in Rust.
#[pymodule]
fn _forceloom(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", forceloom::VERSION)?;
    Ok(())
}
