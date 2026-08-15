use hearth_env::{HearthEnv as RustHearthEnv, MatchConfig};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

fn runtime_error(error: impl ToString) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

fn to_json(value: &impl serde::Serialize) -> PyResult<String> {
    serde_json::to_string(value).map_err(runtime_error)
}

/// Thin Python transport for the framework-neutral Rust adapter.
///
/// JSON keeps the ABI stable and inspectable during environment development.
/// The training layer can later replace this transport with NumPy buffers
/// without changing `hearth-env` or the game engine.
#[pyclass(unsendable, name = "HearthEnv")]
struct PyHearthEnv {
    inner: RustHearthEnv,
}

#[pymethods]
impl PyHearthEnv {
    #[new]
    #[pyo3(signature = (data_path, match_config_json, seed=0, max_steps=1000))]
    fn new(
        data_path: &str,
        match_config_json: &str,
        seed: u64,
        max_steps: usize,
    ) -> PyResult<Self> {
        let match_config: MatchConfig = serde_json::from_str(match_config_json)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let inner =
            RustHearthEnv::load(data_path, match_config, seed, max_steps).map_err(runtime_error)?;
        Ok(Self { inner })
    }

    fn decision_json(&self) -> PyResult<Option<String>> {
        self.inner.decision().map(to_json).transpose()
    }

    fn reset_json(&mut self, seed: u64) -> PyResult<String> {
        let decision = self.inner.reset(seed).map_err(runtime_error)?;
        to_json(decision)
    }

    fn step_json(&mut self, decision_id: u64, action_index: usize) -> PyResult<String> {
        let transition = self
            .inner
            .step(decision_id, action_index)
            .map_err(runtime_error)?;
        to_json(&transition)
    }

    fn pack_hash(&self) -> PyResult<String> {
        self.inner
            .pack_hash()
            .map(str::to_owned)
            .map_err(runtime_error)
    }

    fn card_ids(&self) -> PyResult<Vec<String>> {
        self.inner.card_ids().map_err(runtime_error)
    }

    fn replay_json(&self) -> PyResult<String> {
        to_json(&self.inner.replay().map_err(runtime_error)?)
    }
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyHearthEnv>()?;
    Ok(())
}
