/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};

// Local Imports
use super::execution::{
    ExecutionEvidenceReader, PyExecutionEvidenceStream, PyExecutionRequestHandle,
    PyTestExecutionHandle,
};
use dokime::client::{ResponseStream, TestCycleHandle, wire};
use dokime::core::{CaseId, PromptId, TestExecutionId};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[pyclass(name = "TestCycleHandle", module = "dokime._native.client", frozen)]
pub(crate) struct PyTestCycleHandle {
    pub(super) inner: TestCycleHandle,
}

#[pyclass(name = "PromptStream", module = "dokime._native.client", frozen)]
pub(crate) struct PyPromptStream {
    inner: ResponseStream<wire::PromptInstance>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[pymethods]
impl PyTestCycleHandle {
    fn pending_prompts<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let inner = inner.pending_prompts().map_err(super::into_pyerr)?;
            Ok(PyPromptStream { inner })
        })
    }

    fn prompt<'py>(&self, py: Python<'py>, prompt_id: String) -> PyResult<Bound<'py, PyAny>> {
        let id = PromptId::from_uuid(prompt_id)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let inner = self.inner.clone();
        super::response(py, async move { inner.prompt(&id).await })
    }

    fn respond_prompt<'py>(
        &self,
        py: Python<'py>,
        prompt_id: String,
        action: String,
        values: &[u8],
        accepted: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let id = PromptId::from_uuid(prompt_id)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let values = super::decode_request::<wire::ValueObject>(values)?;
        let inner = self.inner.clone();
        super::response(py, async move {
            inner.respond_prompt(&id, action, values, accepted).await
        })
    }

    #[pyo3(signature = (*, streams=Vec::new(), execution_id=None, until=None, page_size=500, poll_interval=None))]
    fn samples<'py>(
        &self,
        py: Python<'py>,
        streams: Vec<String>,
        execution_id: Option<String>,
        until: Option<PyRef<'_, PyExecutionRequestHandle>>,
        page_size: u32,
        poll_interval: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let execution_id = execution_id
            .map(TestExecutionId::from_uuid)
            .transpose()
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let interval = super::rpc_timeout(poll_interval)?;
        let until = until.map(|request| request.inner.clone());
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let reader = inner
                .samples(streams, execution_id, until.as_ref(), page_size, interval)
                .map_err(super::into_pyerr)?;
            Ok(PyExecutionEvidenceStream {
                inner: ExecutionEvidenceReader::Samples(reader),
            })
        })
    }

    #[pyo3(signature = (*, record_types=Vec::new(), execution_id=None, until=None))]
    fn records<'py>(
        &self,
        py: Python<'py>,
        record_types: Vec<String>,
        execution_id: Option<String>,
        until: Option<PyRef<'_, PyExecutionRequestHandle>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let execution_id = execution_id
            .map(TestExecutionId::from_uuid)
            .transpose()
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let until = until.map(|request| request.inner.clone());
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let reader = inner
                .records(record_types, execution_id, until.as_ref())
                .map_err(super::into_pyerr)?;
            Ok(PyExecutionEvidenceStream {
                inner: ExecutionEvidenceReader::Records(reader),
            })
        })
    }

    #[getter]
    fn id(&self) -> &str {
        self.inner.id().as_str()
    }

    fn snapshot<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        super::response(py, async move { inner.snapshot().await })
    }

    fn run<'py>(&self, py: Python<'py>, request_key: String) -> PyResult<Bound<'py, PyAny>> {
        dokime::core::command::validate_idempotency_key(&request_key)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let inner = self.inner.clone();
        super::response(py, async move { inner.run(&request_key).await })
    }

    fn executions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let handles = inner.executions().await.map_err(super::into_pyerr)?;
            Ok(handles
                .into_iter()
                .map(|inner| PyTestExecutionHandle { inner })
                .collect::<Vec<_>>())
        })
    }

    fn append<'py>(&self, py: Python<'py>, cases: Vec<String>) -> PyResult<Bound<'py, PyAny>> {
        let cases = case_ids(cases)?;
        let inner = self.inner.clone();
        super::response(py, async move { inner.append(&cases).await })
    }
}

#[pymethods]
impl PyPromptStream {
    fn message<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let row = inner.message().await.map_err(super::into_pyerr)?;
            Python::attach(|py| Ok(row.map(|row| PyBytes::new(py, &row.encode_to_vec()).unbind())))
        })
    }

    fn close<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        inner.cancel();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.close().await;
            Ok(())
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(super) fn case_ids(values: Vec<String>) -> PyResult<Vec<CaseId>> {
    values
        .into_iter()
        .map(|value| {
            CaseId::from_wire(value).map_err(|error| PyValueError::new_err(error.to_string()))
        })
        .collect()
}

/* === Tests === */
