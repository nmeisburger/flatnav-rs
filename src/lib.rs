mod flatnav;

use numpy::{PyReadonlyArray1, PyUntypedArrayMethods};
use pyo3::{exceptions, prelude::*};
use pyo3::{pymodule, types::PyModule, Bound, PyResult};

#[pyclass(name = "IndexEuclideanF32")]
struct PyIndexEuclideanF32 {
    index: flatnav::IndexEuclideanF32,
}

#[pymethods]
impl PyIndexEuclideanF32 {
    #[new]
    fn new(max_nbrs: usize, data_dim: usize, capacity: usize) -> PyResult<Self> {
        Ok(Self {
            index: flatnav::IndexEuclideanF32::new(
                max_nbrs,
                data_dim,
                capacity,
                flatnav::EuclideanDistance,
            ),
        })
    }

    fn insert(
        &mut self,
        label: u64,
        data: PyReadonlyArray1<f32>,
        ef_construction: usize,
    ) -> PyResult<()> {
        if data.shape()[0] != self.index.data_dim() {
            return Err(PyErr::new::<exceptions::PyValueError, _>(format!(
                "invalid data dim: expected {}, got {}",
                data.shape()[0],
                self.index.data_dim()
            )));
        }

        self.index.insert(label, data.as_slice()?, ef_construction);
        Ok(())
    }

    fn query(
        &self,
        query: PyReadonlyArray1<f32>,
        ef_search: usize,
        topk: usize,
    ) -> PyResult<Vec<(u64, f32)>> {
        if query.shape()[0] != self.index.data_dim() {
            return Err(PyErr::new::<exceptions::PyValueError, _>(format!(
                "invalid data dim: expected {}, got {}",
                query.shape()[0],
                self.index.data_dim()
            )));
        }

        let results = self.index.query(query.as_slice()?, ef_search, topk);
        Ok(results)
    }

    fn __len__(&self) -> usize {
        self.index.len()
    }
}

#[pymodule(name = "flatnav")]
fn flatnav_lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIndexEuclideanF32>()?;

    Ok(())
}
