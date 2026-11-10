/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Awaitable Rust Client Handles

/* === Modules === */

mod messages;
mod pairing;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{error::Error as _, num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    sync::MutexExt,
    types::{PyBytes, PyTuple},
};
use pyo3_async_runtimes::tokio::future_into_py;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use tonic::metadata::{KeyAndValueRef, MetadataMap};

// Local Imports
use dokime::client::{Client, Error, RawStreaming, Target};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Opening = tokio::task::JoinHandle<dokime::client::Result<tonic::Response<RawStreaming<Vec<u8>>>>>;

/* === Structs === */

#[pyclass(name = "Client", module = "dokime._native", frozen)]
pub struct PyClient {
    inner: Client,
}

#[pyclass(name = "Stream", module = "dokime._native", frozen)]
pub struct PyStream {
    inner: Arc<Mutex<Option<Incoming>>>,
    opening: Arc<Mutex<Option<Opening>>>,
    client: Client,
    sender: Arc<std::sync::Mutex<Option<mpsc::Sender<Vec<u8>>>>>,
    cancellation: CancellationToken,
}

struct Incoming {
    stream: RawStreaming<Vec<u8>>,
    metadata: MetadataMap,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[pymethods]
impl PyClient {
    #[staticmethod]
    #[pyo3(signature = (endpoint, endpoints, settings, token=None, expected_id=None, ca=None))]
    fn connect<'py>(
        py: Python<'py>, endpoint: String, endpoints: Vec<String>, settings: String, token: Option<String>, expected_id: Option<String>,
        ca: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let settings = serde_json::from_str(&settings).map_err(value_error)?;
        let target = Target::new(endpoint, ca).and_then(|target| target.with_endpoints(&endpoints)).map_err(error)?.with_settings(settings);
        future_into_py(py, async move {
            let inner = Client::connect(target, token.as_deref(), expected_id.as_deref()).await.map_err(error)?;
            Ok(Self { inner })
        })
    }

    #[getter]
    fn backend(&self) -> PyResult<String> {
        serde_json::to_string(self.inner.backend()).map_err(value_error)
    }

    #[getter]
    fn endpoint(&self) -> &str {
        self.inner.endpoint()
    }

    fn verify<'py>(&self, py: Python<'py>, live: bool) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move { client.verify(live).await.map_err(error) })
    }

    fn call<'py>(&self, py: Python<'py>, path: String, request: Vec<u8>, timeout: f64) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let deadline = duration(timeout)?;
        future_into_py(py, async move {
            let response = client.call_raw(&path, request, deadline).await.map_err(error)?;
            Python::attach(|py| Ok(PyBytes::new(py, &response).unbind()))
        })
    }

    fn operations<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let operations = client.operations().await.map_err(|failure| match failure {
                dokime::client::operations::Error::Client(failure) => error(failure),
                failure @ dokime::client::operations::Error::CatalogTooLarge => {
                    error(Error::Rpc(tonic::Status::resource_exhausted(failure.to_string())))
                }
                _ => error(Error::IncompatibleApi),
            })?;
            serde_json::to_string(&operations).map_err(value_error)
        })
    }

    fn stream<'py>(&self, py: Python<'py>, path: String, opening: Vec<u8>, timeout: f64) -> PyResult<Bound<'py, PyAny>> {
        let client = if matches!(path.as_str(), "/dokime.v1.Reports/DownloadCycleReport" | "/dokime.v1.Reports/DownloadExecutionReport") {
            self.inner.with_frame_limit(NonZeroUsize::new(2_097_152).expect("nonzero report frame limit"))
        } else {
            self.inner.clone()
        };
        let deadline = duration(timeout)?;
        future_into_py(py, async move {
            let (sender, receiver) = mpsc::channel(1);
            sender.send(opening).await.map_err(value_error)?;
            let cancellation = CancellationToken::new();
            let cancelled = cancellation.clone();
            let connection = client.clone();
            let opening = tokio::spawn(async move {
                tokio::select! {
                    biased;
                    _ = cancelled.cancelled() => Err(Error::Closed),
                    incoming = connection.stream_raw_response(&path, ReceiverStream::new(receiver), deadline) => incoming,
                }
            });
            Ok(PyStream {
                inner: Arc::new(Mutex::new(None)),
                opening: Arc::new(Mutex::new(Some(opening))),
                client,
                sender: Arc::new(std::sync::Mutex::new(Some(sender))),
                cancellation,
            })
        })
    }

    fn close<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        // Close Synchronously So Cancellation Of The Drain Cannot Leave A Live Channel.
        let client = self.inner.clone();
        py.detach(|| client.close());
        future_into_py(py, async move {
            client.close_and_wait().await;
            Ok(())
        })
    }
}

#[pymethods]
impl PyStream {
    fn next<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let opening = self.opening.clone();
        let cancellation = self.cancellation.clone();
        let client = self.client.clone();
        future_into_py(py, async move {
            let mut metadata = None;
            let response = async {
                let mut incoming = inner.lock().await;
                let mut opening = opening.lock().await;
                if let Some(pending) = opening.as_mut() {
                    let result = tokio::select! {
                        biased;
                        _ = cancellation.cancelled() => { pending.abort(); return Ok(None); }
                        result = &mut *pending => result.map_err(Error::Task)?,
                    };
                    opening.take();
                    let response = result?;
                    *incoming = Some(Incoming {
                        metadata: response.metadata().clone(),
                        stream: response.into_inner(),
                    });
                }
                drop(opening);
                let Some(stream) = incoming.as_mut() else {
                    return Ok(None);
                };
                metadata = Some(stream.metadata.clone());
                let response = tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => { incoming.take(); return Ok(None); }
                    response = client.guard(stream.stream.message()) => response?.map_err(Error::Rpc)?,
                };
                if response.is_none() {
                    incoming.take();
                }
                Ok::<_, Error>(response)
            }
            .await
            .map_err(|failure| error_with_metadata(failure, metadata.as_ref()))?;
            Python::attach(|py| Ok(response.map(|bytes| PyBytes::new(py, &bytes).unbind())))
        })
    }

    fn send<'py>(&self, py: Python<'py>, frame: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let sender = self.sender.lock_py_attached(py).map_err(value_error)?.clone().ok_or_else(|| value_error("stream input is closed"))?;
        let cancellation = self.cancellation.clone();
        future_into_py(py, async move {
            tokio::select! {
                biased;
                _ = cancellation.cancelled() => Err(value_error("stream is closed")),
                result = sender.send(frame) => result.map_err(value_error),
            }
        })
    }

    fn finish(&self, py: Python<'_>) -> PyResult<()> {
        self.sender.lock_py_attached(py).map_err(value_error)?.take();
        Ok(())
    }

    fn cancel(&self, py: Python<'_>) -> PyResult<()> {
        self.cancellation.cancel();
        self.finish(py)?;
        // Drop Idle Incoming State Immediately; A Pending Next Observes Cancellation.
        if let Ok(mut incoming) = self.inner.try_lock() {
            incoming.take();
        }
        Ok(())
    }

    fn close<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.cancel(py)?;
        let inner = self.inner.clone();
        let opening = self.opening.clone();
        future_into_py(py, async move {
            inner.lock().await.take();
            if let Some(pending) = opening.lock().await.take() {
                pending.abort();
                let _ = pending.await;
            }
            Ok(())
        })
    }
}

impl Drop for PyStream {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

/* === Public Functions === */

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyClient>()?;
    module.add_class::<PyStream>()?;
    messages::register(module)?;
    pairing::register(module)
}

/* === Private Functions === */

fn duration(seconds: f64) -> PyResult<Duration> {
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err(value_error("timeout must be finite and positive"));
    }
    Duration::try_from_secs_f64(seconds).map_err(value_error)
}

fn value_error(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn error(failure: Error) -> PyErr {
    error_with_metadata(failure, None)
}

fn error_with_metadata(failure: Error, initial: Option<&MetadataMap>) -> PyErr {
    let mut causes = Vec::new();
    let mut source = failure.source();
    while let Some(cause) = source {
        causes.push(cause.to_string());
        source = cause.source();
    }
    Python::attach(|py| {
        let create = || -> PyResult<PyErr> {
            let errors = py.import("dokime.client.errors")?;
            let exception = match failure {
                Error::Closed => errors.getattr("ClientClosed")?.call1((failure.to_string(),))?,
                Error::IdentityChanged => errors.getattr("BackendChanged")?.call1((failure.to_string(),))?,
                Error::IncompatibleApi => errors.getattr("ProtocolError")?.call1((failure.to_string(),))?,
                Error::InvalidTarget { .. } => return Ok(value_error(failure)),
                Error::Rpc(status) => {
                    let code = format!("{:?}", status.code());
                    let code = code.chars().enumerate().fold(String::new(), |mut result, (index, character)| {
                        if index > 0 && character.is_uppercase() {
                            result.push('_');
                        }
                        result.extend(character.to_uppercase());
                        result
                    });
                    let exception = errors.getattr("RpcError")?.call1((code, status.message()))?;
                    let mut metadata = metadata_values(py, status.metadata());
                    if !status.details().is_empty() {
                        metadata.push(("grpc-status-details-bin".to_owned(), PyBytes::new(py, status.details()).into_any().unbind()));
                    }
                    exception.setattr("trailing_metadata", PyTuple::new(py, metadata)?)?;
                    if let Some(initial) = initial {
                        exception.setattr("initial_metadata", PyTuple::new(py, metadata_values(py, initial))?)?;
                    }
                    exception
                }
                failure => {
                    let code = match failure {
                        Error::AuthenticationRequired => "UNAUTHENTICATED",
                        Error::WaitTimedOut { .. } => "DEADLINE_EXCEEDED",
                        Error::Transport(_) | Error::BackendUnavailable => "UNAVAILABLE",
                        _ => "INTERNAL",
                    };
                    errors.getattr("RpcError")?.call1((code, failure.to_string()))?
                }
            };
            let exception = PyErr::from_value(exception);
            if !causes.is_empty() {
                exception.set_cause(py, Some(PyRuntimeError::new_err(causes.join(": "))));
            }
            Ok(exception)
        };
        create().unwrap_or_else(|failure| failure)
    })
}

fn metadata_values(py: Python<'_>, metadata: &MetadataMap) -> Vec<(String, Py<PyAny>)> {
    metadata
        .iter()
        .filter_map(|entry| match entry {
            KeyAndValueRef::Ascii(key, value) => value
                .to_str()
                .ok()
                .map(|value| (key.as_str().to_owned(), value.into_pyobject(py).expect("string conversion").into_any().unbind())),
            KeyAndValueRef::Binary(key, value) => {
                value.to_bytes().ok().map(|value| (key.as_str().to_owned(), PyBytes::new(py, &value).into_any().unbind()))
            }
        })
        .collect()
}

/* === Tests === */
