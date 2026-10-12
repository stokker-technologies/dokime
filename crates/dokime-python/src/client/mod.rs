/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod backend;
mod cycle;
mod execution;
mod rpc;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;

// Third Party Imports
use prost::Message;
use pyo3::{
    exceptions::{PyOSError, PyValueError},
    prelude::*,
    types::PyBytes,
};

// Local Imports
use backend::{PyBackend, PyLimits};
use cycle::{PyPromptStream, PyTestCycleHandle};
use dokime::client::{Client, Error, ResponseStream, Target, wire};
use execution::{
    PyExecutionEvidenceStream, PyExecutionRequestHandle, PyExecutionRequestUpdates,
    PyTestExecutionHandle,
};

/* === Constants === */

/* === Macros === */
pyo3::create_exception!(client, ClientError, pyo3::exceptions::PyException);

/* === Type Aliases === */

/* === Structs === */
#[pyclass(name = "Target", module = "dokime._native.client", frozen)]
pub(crate) struct PyTarget {
    inner: Target,
}

#[pyclass(name = "Client", module = "dokime._native.client", frozen)]
pub(crate) struct PyClient {
    inner: Client,
}

#[pyclass(name = "ResponseStream", module = "dokime._native.client", frozen)]
pub(crate) struct PyResponseStream {
    inner: rpc::Response,
}

#[pyclass(name = "SuiteStream", module = "dokime._native.client", frozen)]
pub(crate) struct PySuiteStream {
    inner: ResponseStream<wire::SuiteDefinition>,
}

#[pyclass(name = "RequestStream", module = "dokime._native.client", frozen)]
pub(crate) struct PyRequestStream {
    inner: rpc::Upload,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[pymethods]
impl PyTarget {
    #[new]
    #[pyo3(signature = (endpoint, *, ca_certificate_pem=None))]
    fn new(endpoint: String, ca_certificate_pem: Option<String>) -> PyResult<Self> {
        let inner = Target::new(endpoint, ca_certificate_pem)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        Ok(Self { inner })
    }

    #[getter]
    fn is_remote(&self) -> bool {
        self.inner.is_remote()
    }

    #[getter]
    fn endpoint(&self) -> &str {
        self.inner.endpoint()
    }
}

#[pymethods]
impl PyClient {
    #[staticmethod]
    #[pyo3(signature = (target, *, token=None, expected_backend_id=None, timeout=None))]
    fn connect<'py>(
        py: Python<'py>,
        target: &PyTarget,
        token: Option<String>,
        expected_backend_id: Option<String>,
        timeout: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let timeout = rpc_timeout(timeout)?;
        let target = target.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let connect = Client::connect(target, token.as_deref(), expected_backend_id.as_deref());
            let inner = if let Some(timeout) = timeout {
                tokio::time::timeout(timeout, connect)
                    .await
                    .map_err(|_| deadline_error())?
            } else {
                connect.await
            }
            .map_err(into_pyerr)?;
            Ok(Self { inner })
        })
    }

    #[getter]
    fn backend(&self) -> PyBackend {
        PyBackend {
            inner: self.inner.backend().clone(),
        }
    }

    #[pyo3(signature = (cycle_id, *, timeout=None))]
    fn cycle(&self, cycle_id: String, timeout: Option<f64>) -> PyResult<PyTestCycleHandle> {
        let id = dokime::core::TestCycleId::from_uuid(cycle_id)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let client = match rpc_timeout(timeout)? {
            Some(timeout) => self.inner.with_timeout(timeout),
            None => self.inner.clone(),
        };
        Ok(PyTestCycleHandle {
            inner: client.cycle(id),
        })
    }

    #[pyo3(signature = (suite, template, parameters, *, label=None, timeout=None))]
    fn plan_cycle<'py>(
        &self,
        py: Python<'py>,
        suite: String,
        template: String,
        parameters: &[u8],
        label: Option<String>,
        timeout: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let parameters = decode_request::<dokime::client::wire::ValueObject>(parameters)?;
        let client = match rpc_timeout(timeout)? {
            Some(timeout) => self.inner.with_timeout(timeout),
            None => self.inner.clone(),
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let inner = client
                .plan_cycle(&suite, &template, parameters, label)
                .await
                .map_err(into_pyerr)?;
            Ok(PyTestCycleHandle { inner })
        })
    }

    #[pyo3(signature = (plan, key, *, timeout=None))]
    fn register_selection<'py>(
        &self,
        py: Python<'py>,
        plan: &[u8],
        key: String,
        timeout: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let plan = decode_request::<dokime::client::wire::SelectionPlan>(plan)?;
        let client = match rpc_timeout(timeout)? {
            Some(timeout) => self.inner.with_timeout(timeout),
            None => self.inner.clone(),
        };
        response(
            py,
            async move { client.register_selection(plan, &key).await },
        )
    }

    #[pyo3(signature = (case_ids, metadata, *, continue_on_failure=false, timeout=None))]
    fn create_cycle<'py>(
        &self,
        py: Python<'py>,
        case_ids: Vec<String>,
        metadata: &[u8],
        continue_on_failure: bool,
        timeout: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let cases = cycle::case_ids(case_ids)?;
        let metadata = decode_request::<dokime::client::wire::Metadata>(metadata)?;
        let client = match rpc_timeout(timeout)? {
            Some(timeout) => self.inner.with_timeout(timeout),
            None => self.inner.clone(),
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let inner = client
                .create_cycle(&cases, metadata, continue_on_failure)
                .await
                .map_err(into_pyerr)?;
            Ok(PyTestCycleHandle { inner })
        })
    }

    #[pyo3(signature = (*, timeout=None))]
    fn suites<'py>(&self, py: Python<'py>, timeout: Option<f64>) -> PyResult<Bound<'py, PyAny>> {
        let client = match rpc_timeout(timeout)? {
            Some(timeout) => self.inner.with_timeout(timeout),
            None => self.inner.clone(),
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let inner = client.suites().map_err(into_pyerr)?;
            Ok(PySuiteStream { inner })
        })
    }

    #[pyo3(signature = (identifier, *, timeout=None))]
    fn suite<'py>(
        &self,
        py: Python<'py>,
        identifier: String,
        timeout: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = match rpc_timeout(timeout)? {
            Some(timeout) => self.inner.with_timeout(timeout),
            None => self.inner.clone(),
        };
        response(py, async move { client.suite(&identifier).await })
    }

    #[pyo3(signature = (request_id, *, cycle_id=None, timeout=None))]
    fn execution_request(
        &self,
        request_id: String,
        cycle_id: Option<String>,
        timeout: Option<f64>,
    ) -> PyResult<PyExecutionRequestHandle> {
        let id = dokime::core::CommandId::from_uuid(request_id)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let client = match rpc_timeout(timeout)? {
            Some(timeout) => self.inner.with_timeout(timeout),
            None => self.inner.clone(),
        };
        let inner = match cycle_id {
            Some(cycle) => {
                let cycle = dokime::core::TestCycleId::from_uuid(cycle)
                    .map_err(|error| PyValueError::new_err(error.to_string()))?;
                client.cycle(cycle).request(id)
            }
            None => client.execution_request(id),
        };
        Ok(PyExecutionRequestHandle { inner })
    }

    #[pyo3(signature = (execution_id, *, timeout=None))]
    fn execution(
        &self,
        execution_id: String,
        timeout: Option<f64>,
    ) -> PyResult<PyTestExecutionHandle> {
        let id = dokime::core::TestExecutionId::from_uuid(execution_id)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let client = match rpc_timeout(timeout)? {
            Some(timeout) => self.inner.with_timeout(timeout),
            None => self.inner.clone(),
        };
        Ok(PyTestExecutionHandle {
            inner: client.execution(id),
        })
    }

    #[pyo3(signature = (method, request, *, timeout=None))]
    fn unary<'py>(
        &self,
        py: Python<'py>,
        method: String,
        request: &Bound<'_, PyBytes>,
        timeout: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = request.as_bytes();
        if request.len() > 16 * 1024 * 1024 {
            return Err(PyValueError::new_err(
                "RPC request exceeds the client message limit",
            ));
        }
        let request = request.to_vec();
        let inner = match rpc_timeout(timeout)? {
            Some(timeout) => self.inner.with_timeout(timeout),
            None => self.inner.clone(),
        };

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = rpc::unary(&inner, &method, &request).await?;
            Python::attach(|py| Ok(PyBytes::new(py, &response).unbind()))
        })
    }

    fn stream<'py>(
        &self,
        py: Python<'py>,
        method: String,
        request: &Bound<'_, PyBytes>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = request.as_bytes();
        if request.len() > 16 * 1024 * 1024 {
            return Err(PyValueError::new_err(
                "RPC request exceeds the client message limit",
            ));
        }
        let request = request.to_vec();
        let inner = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            rpc::stream(&inner, &method, &request)
                .await
                .map(|inner| PyResponseStream { inner })
        })
    }

    fn download_artifact<'py>(
        &self,
        py: Python<'py>,
        request: &[u8],
        destination: std::path::PathBuf,
        expected_size: u64,
        expected_sha256: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request =
            decode_request::<dokime::client::wire::DownloadCheckpointArtifactRequest>(request)?;
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client
                .download_checkpoint_artifact(
                    request,
                    destination,
                    (expected_size, expected_sha256),
                )
                .await
                .map_err(download_error)?;
            Python::attach(|py| {
                Ok((
                    result.size_bytes,
                    result.sha256,
                    PyBytes::new(py, &result.metadata.encode_to_vec()).unbind(),
                ))
            })
        })
    }

    fn download_report<'py>(
        &self,
        py: Python<'py>,
        request: &[u8],
        destination: std::path::PathBuf,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = decode_request::<dokime::client::wire::DownloadCycleReportRequest>(request)?;
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client
                .download_cycle_report(request, destination)
                .await
                .map_err(download_error)?;
            Python::attach(|py| {
                Ok((
                    result.size_bytes,
                    result.sha256,
                    PyBytes::new(py, &result.metadata.encode_to_vec()).unbind(),
                ))
            })
        })
    }

    fn upload<'py>(&self, py: Python<'py>, method: String) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            rpc::upload(&inner, &method).map(|inner| PyRequestStream { inner })
        })
    }

    #[pyo3(signature = (*, live=true))]
    fn verify<'py>(&self, py: Python<'py>, live: bool) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.verify(live).await.map_err(into_pyerr)
        })
    }

    fn close<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        // Cancellation starts even if Python immediately cancels its waiter.
        inner.close();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.close_and_wait().await;
            Ok(())
        })
    }
}

#[pymethods]
impl PyResponseStream {
    fn message<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let row = inner.message().await?;
            Python::attach(|py| Ok(row.map(|row| PyBytes::new(py, &row).unbind())))
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
impl PySuiteStream {
    fn message<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let row = inner.message().await.map_err(into_pyerr)?;
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

#[pymethods]
impl PyRequestStream {
    fn send<'py>(
        &self,
        py: Python<'py>,
        frame: &Bound<'_, PyBytes>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let frame = frame.as_bytes();
        if frame.len() > 16 * 1024 * 1024 {
            return Err(PyValueError::new_err(
                "RPC request exceeds the client message limit",
            ));
        }
        let frame = frame.to_vec();
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move { inner.send(&frame).await })
    }

    fn finish<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = inner.finish().await?;
            Python::attach(|py| Ok(PyBytes::new(py, &response).unbind()))
        })
    }

    fn end_input<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move { inner.end_input().await })
    }

    fn response<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = inner.response().await?;
            Python::attach(|py| Ok(PyBytes::new(py, &response).unbind()))
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
#[pymodule]
pub(crate) fn client(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyClient>()?;
    module.add_class::<PyTestCycleHandle>()?;
    module.add_class::<PyPromptStream>()?;
    module.add_class::<PyTestExecutionHandle>()?;
    module.add_class::<PyExecutionEvidenceStream>()?;
    module.add_class::<PyExecutionRequestHandle>()?;
    module.add_class::<PyExecutionRequestUpdates>()?;
    module.add_class::<PyTarget>()?;
    module.add_class::<PyBackend>()?;
    module.add_class::<PyLimits>()?;
    module.add_class::<PyResponseStream>()?;
    module.add_class::<PySuiteStream>()?;
    module.add_class::<PyRequestStream>()?;
    let exception = module.py().get_type::<ClientError>();
    exception.setattr("__module__", "dokime._native.client")?;
    module.add("ClientError", exception)?;
    Ok(())
}

/* === Private Functions === */
fn response<'py, M: Message + Send + 'static>(
    py: Python<'py>,
    operation: impl Future<Output = dokime::client::Result<M>> + Send + 'static,
) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let response = operation.await.map_err(into_pyerr)?;
        let bytes = response.encode_to_vec();
        Python::attach(|py| Ok(PyBytes::new(py, &bytes).unbind()))
    })
}

fn decode_request<T: Message + Default>(request: &[u8]) -> PyResult<T> {
    if request.len() > 16 * 1024 * 1024 {
        return Err(PyValueError::new_err(
            "RPC request exceeds the client message limit",
        ));
    }
    T::decode(request).map_err(|error| PyValueError::new_err(error.to_string()))
}

fn download_error(error: dokime::client::DownloadError) -> PyErr {
    match error {
        dokime::client::DownloadError::Client(error) => into_pyerr(error),
        dokime::client::DownloadError::Invalid(message) => PyValueError::new_err(message),
        error => PyOSError::new_err(error.to_string()),
    }
}

fn rpc_timeout(timeout: Option<f64>) -> PyResult<Option<std::time::Duration>> {
    timeout
        .map(|seconds| {
            if !seconds.is_finite() || seconds <= 0.0 {
                return Err(PyValueError::new_err(
                    "RPC timeout must be positive and finite",
                ));
            }
            std::time::Duration::try_from_secs_f64(seconds)
                .map_err(|_| PyValueError::new_err("RPC timeout is out of range"))
        })
        .transpose()
}

fn deadline_error() -> PyErr {
    let exception = ClientError::new_err("connection deadline exceeded");
    Python::attach(|py| {
        let value = exception.value(py);
        value.setattr("kind", "rpc")?;
        value.setattr("code", 4)?;
        value.setattr("details", PyBytes::new(py, &[]))?;
        Ok::<_, PyErr>(())
    })
    .err()
    .unwrap_or(exception)
}

fn into_pyerr(error: Error) -> PyErr {
    let (kind, code, details) = match &error {
        Error::InvalidTarget { .. } => ("invalid_target", None, Vec::new()),
        Error::AuthenticationRequired => ("authentication_required", None, Vec::new()),
        Error::Transport(_) => ("transport", None, Vec::new()),
        Error::Rpc(status) => ("rpc", Some(status.code() as i32), status.details().to_vec()),
        Error::ExecutionRejected { rejection } => {
            ("execution_rejected", None, rejection.encode_to_vec())
        }
        Error::IncompatibleApi => ("incompatible_api", None, Vec::new()),
        Error::IdentityChanged => ("identity_changed", None, Vec::new()),
        Error::BackendUnavailable => ("backend_unavailable", None, Vec::new()),
        Error::Task(_) => ("worker", None, Vec::new()),
        Error::Closed => ("closed", None, Vec::new()),
        Error::WaitTimedOut { .. } => ("wait_timeout", Some(4), Vec::new()),
    };
    let message = match &error {
        Error::Rpc(status) => status.message().to_owned(),
        _ => error.to_string(),
    };
    let exception = ClientError::new_err(message);

    Python::attach(|py| {
        let value = exception.value(py);
        value.setattr("kind", kind)?;
        value.setattr("code", code)?;
        value.setattr("details", PyBytes::new(py, &details))?;
        Ok::<_, PyErr>(())
    })
    .err()
    .unwrap_or(exception)
}

/* === Tests === */
