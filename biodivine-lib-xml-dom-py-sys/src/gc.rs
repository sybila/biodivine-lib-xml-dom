//! [`GcStats`]: what one run of the document collector did.

use biodivine_lib_xml_dom::GcStats;
use pyo3::prelude::*;

/// What one run of the garbage collector did.
///
/// The object is a snapshot: it describes the arena at the moment the collection finished.
#[pyclass(
    name = "GcStats",
    module = "biodivine_lib_xml_dom._sys",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PyGcStats {
    pub(crate) inner: GcStats,
}

impl From<GcStats> for PyGcStats {
    fn from(inner: GcStats) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyGcStats {
    /// The arena capacity after the collection, in slots.
    #[getter]
    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }

    /// The number of slots that hold a node after the collection.
    #[getter]
    pub fn live(&self) -> usize {
        self.inner.live
    }

    /// The number of slots that can be allocated without growing the arena.
    #[getter]
    pub fn free(&self) -> usize {
        self.inner.free
    }

    /// The number of nodes this collection reclaimed.
    #[getter]
    pub fn collected(&self) -> usize {
        self.inner.collected
    }

    /// The fraction of the capacity that is free, in `[0.0, 1.0]`.
    pub fn free_fraction(&self) -> f64 {
        self.inner.free_fraction()
    }

    /// Equality with another snapshot: all four numbers agree.
    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }

    /// A debug representation for interactive use, not a serialization of the value.
    pub fn __repr__(&self) -> String {
        format!(
            "GcStats(capacity={}, live={}, free={}, collected={})",
            self.inner.capacity, self.inner.live, self.inner.free, self.inner.collected
        )
    }
}
