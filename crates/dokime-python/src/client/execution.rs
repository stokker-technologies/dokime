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
use super::response;
use dokime::client::{ExecutionRequestHandle, ResponseStream, TestExecutionHandle, wire};
use dokime::core::{CommandId, StepId};
use dokime::execution::{ExecutionMode, request::StepCommand};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[pyclass(name = "TestExecutionHandle", module = "dokime._native.client", frozen)]
pub(crate) struct PyTestExecutionHandle {
    pub(super) inner: TestExecutionHandle,
}

#[pyclass(
    name = "ExecutionRequestHandle",
    module = "dokime._native.client",
    frozen
)]
pub(crate) struct PyExecutionRequestHandle {
    pub(super) inner: ExecutionRequestHandle,
}

#[pyclass(
    name = "ExecutionRequestUpdates",
    module = "dokime._native.client",
    frozen
)]
pub(crate) struct PyExecutionRequestUpdates {
    inner: ResponseStream<wire::RunRequestReceipt>,
}

#[pyclass(
    name = "ExecutionEvidenceStream",
    module = "dokime._native.client",
    frozen
)]
pub(crate) struct PyExecutionEvidenceStream {
    pub(super) inner: ExecutionEvidenceReader,
}

/* === Enums === */
#[derive(Clone)]
pub(super) enum ExecutionEvidenceReader {
    Records(ResponseStream<wire::RecordPage>),
    Samples(ResponseStream<wire::TelemetrySamplesReply>),
    SavedRecords(ResponseStream<wire::CheckpointRecordPage>),
    SavedSamples(ResponseStream<wire::CheckpointTelemetryPage>),
}

/* === Traits === */

/* === Implementations === */
#[pymethods]
impl PyExecutionRequestHandle {
    #[getter]
    fn id(&self) -> &str {
        self.inner.id().as_str()
    }

    fn status<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        response(py, async move { inner.status().await })
    }

    #[pyo3(signature = (*, poll_interval=None))]
    fn updates<'py>(
        &self,
        py: Python<'py>,
        poll_interval: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let interval = super::rpc_timeout(poll_interval)?;
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let inner = inner.updates(interval).map_err(super::into_pyerr)?;
            Ok(PyExecutionRequestUpdates { inner })
        })
    }

    #[pyo3(signature = (*, timeout=None, poll_interval=None))]
    fn wait<'py>(
        &self,
        py: Python<'py>,
        timeout: Option<f64>,
        poll_interval: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let timeout = super::rpc_timeout(timeout)?;
        let interval = super::rpc_timeout(poll_interval)?;
        let inner = self.inner.clone();
        response(py, async move { inner.wait(timeout, interval).await })
    }
}

#[pymethods]
impl PyExecutionRequestUpdates {
    fn message<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let receipt = inner.message().await.map_err(super::into_pyerr)?;
            let bytes = receipt.map(|receipt| receipt.encode_to_vec());
            Python::attach(|py| Ok(bytes.map(|bytes| PyBytes::new(py, &bytes).unbind())))
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

#[pymethods]
impl PyTestExecutionHandle {
    #[getter]
    fn id(&self) -> &str {
        self.inner.id().as_str()
    }

    fn snapshot<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        response(py, async move { inner.snapshot().await })
    }

    fn checkpoint<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        response(py, async move { inner.checkpoint().await })
    }

    fn document_at<'py>(&self, py: Python<'py>, checkpoint: &[u8]) -> PyResult<Bound<'py, PyAny>> {
        let checkpoint = super::decode_request::<wire::ExecutionCheckpoint>(checkpoint)?;
        let inner = self.inner.clone();
        response(py, async move { inner.document_at(&checkpoint).await })
    }

    fn context_at<'py>(&self, py: Python<'py>, checkpoint: &[u8]) -> PyResult<Bound<'py, PyAny>> {
        let checkpoint = super::decode_request::<wire::ExecutionCheckpoint>(checkpoint)?;
        let inner = self.inner.clone();
        response(py, async move { inner.context_at(&checkpoint).await })
    }

    fn evidence_at<'py>(&self, py: Python<'py>, checkpoint: &[u8]) -> PyResult<Bound<'py, PyAny>> {
        let checkpoint = super::decode_request::<wire::ExecutionCheckpoint>(checkpoint)?;
        let inner = self.inner.clone();
        response(py, async move { inner.evidence_at(&checkpoint).await })
    }

    fn records_at<'py>(&self, py: Python<'py>, checkpoint: &[u8]) -> PyResult<Bound<'py, PyAny>> {
        let checkpoint = super::decode_request::<wire::ExecutionCheckpoint>(checkpoint)?;
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let reader = inner.records_at(checkpoint).map_err(super::into_pyerr)?;
            Ok(PyExecutionEvidenceStream {
                inner: ExecutionEvidenceReader::SavedRecords(reader),
            })
        })
    }

    fn samples_at<'py>(&self, py: Python<'py>, checkpoint: &[u8]) -> PyResult<Bound<'py, PyAny>> {
        let checkpoint = super::decode_request::<wire::ExecutionCheckpoint>(checkpoint)?;
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let reader = inner.samples_at(checkpoint).map_err(super::into_pyerr)?;
            Ok(PyExecutionEvidenceStream {
                inner: ExecutionEvidenceReader::SavedSamples(reader),
            })
        })
    }

    #[pyo3(signature = (*, timeout=None, poll_interval=None))]
    fn wait<'py>(
        &self,
        py: Python<'py>,
        timeout: Option<f64>,
        poll_interval: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let timeout = super::rpc_timeout(timeout)?;
        let interval = super::rpc_timeout(poll_interval)?;
        let inner = self.inner.clone();
        response(py, async move { inner.wait(timeout, interval).await })
    }

    #[pyo3(signature = (*, record_types=Vec::new(), follow=false, poll_interval=None))]
    fn records<'py>(
        &self,
        py: Python<'py>,
        record_types: Vec<String>,
        follow: bool,
        poll_interval: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let interval = super::rpc_timeout(poll_interval)?;
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let reader = inner
                .records(record_types, follow, interval)
                .map_err(super::into_pyerr)?;
            Ok(PyExecutionEvidenceStream {
                inner: ExecutionEvidenceReader::Records(reader),
            })
        })
    }

    #[pyo3(signature = (*, streams=Vec::new(), follow=false, page_size=500, poll_interval=None))]
    fn samples<'py>(
        &self,
        py: Python<'py>,
        streams: Vec<String>,
        follow: bool,
        page_size: u32,
        poll_interval: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let interval = super::rpc_timeout(poll_interval)?;
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let reader = inner
                .samples(streams, follow, page_size, interval)
                .map_err(super::into_pyerr)?;
            Ok(PyExecutionEvidenceStream {
                inner: ExecutionEvidenceReader::Samples(reader),
            })
        })
    }

    #[pyo3(signature = (request_key, *, mode="automatic"))]
    fn start<'py>(
        &self,
        py: Python<'py>,
        request_key: String,
        mode: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        dokime::core::command::validate_idempotency_key(&request_key)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let mode = match mode {
            "automatic" => ExecutionMode::Automatic,
            "interactive" => ExecutionMode::Interactive,
            _ => {
                return Err(PyValueError::new_err(
                    "mode must be automatic or interactive",
                ));
            }
        };
        let inner = self.inner.clone();
        response(py, async move { inner.start(&request_key, mode).await })
    }

    #[pyo3(signature = (request_id, command_id, action, *, step_id=None))]
    fn command<'py>(
        &self,
        py: Python<'py>,
        request_id: String,
        command_id: String,
        action: &str,
        step_id: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request_id = CommandId::from_uuid(request_id)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let command_id = CommandId::from_uuid(command_id)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let command = match (action, step_id) {
            ("next", None) => StepCommand::Next,
            ("finish", None) => StepCommand::Finish,
            ("run_step", Some(step_id)) => StepCommand::RunStep {
                step_id: StepId::from_uuid(step_id)
                    .map_err(|error| PyValueError::new_err(error.to_string()))?,
            },
            _ => {
                return Err(PyValueError::new_err(
                    "next/finish take no step_id; run_step requires a step UUID",
                ));
            }
        };
        let inner = self.inner.clone();
        response(py, async move {
            inner.command(&request_id, &command_id, &command).await
        })
    }

    fn command_receipt<'py>(
        &self,
        py: Python<'py>,
        request_id: String,
        command_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request_id = CommandId::from_uuid(request_id)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let command_id = CommandId::from_uuid(command_id)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let inner = self.inner.clone();
        response(py, async move {
            inner.command_receipt(&request_id, &command_id).await
        })
    }

    fn pause<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        response(py, async move { inner.pause().await })
    }

    fn resume<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        response(py, async move { inner.resume().await })
    }

    fn stop<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        response(py, async move { inner.stop().await })
    }
}

#[pymethods]
impl PyExecutionEvidenceStream {
    fn message<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let bytes = match inner {
                ExecutionEvidenceReader::Records(reader) => reader
                    .message()
                    .await
                    .map(|page| page.map(|page| page.encode_to_vec())),
                ExecutionEvidenceReader::Samples(reader) => reader
                    .message()
                    .await
                    .map(|page| page.map(|page| page.encode_to_vec())),
                ExecutionEvidenceReader::SavedRecords(reader) => reader
                    .message()
                    .await
                    .map(|page| page.map(|page| page.encode_to_vec())),
                ExecutionEvidenceReader::SavedSamples(reader) => reader
                    .message()
                    .await
                    .map(|page| page.map(|page| page.encode_to_vec())),
            }
            .map_err(super::into_pyerr)?;
            Python::attach(|py| Ok(bytes.map(|bytes| PyBytes::new(py, &bytes).unbind())))
        })
    }

    fn close<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        match &inner {
            ExecutionEvidenceReader::Records(reader) => reader.cancel(),
            ExecutionEvidenceReader::Samples(reader) => reader.cancel(),
            ExecutionEvidenceReader::SavedRecords(reader) => reader.cancel(),
            ExecutionEvidenceReader::SavedSamples(reader) => reader.cancel(),
        }
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            match inner {
                ExecutionEvidenceReader::Records(reader) => reader.close().await,
                ExecutionEvidenceReader::Samples(reader) => reader.close().await,
                ExecutionEvidenceReader::SavedRecords(reader) => reader.close().await,
                ExecutionEvidenceReader::SavedSamples(reader) => reader.close().await,
            }
            Ok(())
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
/* === Tests === */
