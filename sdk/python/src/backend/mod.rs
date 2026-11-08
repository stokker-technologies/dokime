/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Embedded Backend Ownership And Startup Refusals

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

// Third Party Imports
use async_trait::async_trait;
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_async_runtimes::tokio::{future_into_py, get_runtime};
use tokio::sync::Mutex;

// Local Imports
use dokime::runtime::EmbeddedProvider;
use dokime_server::host::{Args, Error, NetworkArgs, Session};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[pyclass(name = "Backend", module = "dokime._native", frozen)]
pub struct PyBackend {
    inner: Arc<Mutex<Session>>,
    accepting: AtomicBool,
    endpoint: String,
    root: PathBuf,
}

// A Metadata-Only Provider Is Useful Before Any Templates Are Declared.
#[pyclass(name = "EmbeddedProvider", module = "dokime._native", frozen)]
pub struct PyEmbeddedProvider {
    pub(crate) inner: Arc<dyn EmbeddedProvider>,
}

struct CatalogProvider {
    catalog: dokime::InstalledPlugin,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[pymethods]
impl PyEmbeddedProvider {
    #[new]
    fn new(id: String, version: String, description: String) -> PyResult<Self> {
        dokime::Identifier::new(&id).map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;
        let catalog = dokime::InstalledPlugin {
            source: id,
            version,
            description,
            enabled: true,
            ..Default::default()
        };
        Ok(Self {
            inner: Arc::new(CatalogProvider { catalog }),
        })
    }
}

#[async_trait]
impl EmbeddedProvider for CatalogProvider {
    fn catalog(&self) -> dokime::InstalledPlugin {
        self.catalog.clone()
    }
    fn streams(&self) -> Vec<dokime::harmos::sidecar::StreamDeclaration> {
        Vec::new()
    }
    async fn start(&self) -> dokime::runtime::Result<Vec<Arc<dyn dokime::execution::Runnable>>> {
        Ok(Vec::new())
    }
    async fn stop(&self) -> dokime::runtime::Result<()> {
        Ok(())
    }
}

#[pymethods]
impl PyBackend {
    #[staticmethod]
    #[pyo3(signature = (port, root, application_root, plugins))]
    fn start<'py>(
        py: Python<'py>, port: Option<u16>, root: Option<PathBuf>, application_root: Option<PathBuf>,
        plugins: Vec<PyRef<'_, PyEmbeddedProvider>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let providers = plugins.iter().map(|plugin| plugin.inner.clone()).collect();
        future_into_py(py, async move {
            let args = Args {
                shutdown_stdin: false,
                dev_control: false,
                network: NetworkArgs {
                    host: std::net::Ipv4Addr::LOCALHOST.into(),
                    port,
                    message_limit_mib: None,
                },
                title: None,
                label: None,
                output_root: root,
                application_root,
                env_file: None,
                plugins: Vec::new(),
                plugins_root: None,
            };
            let address = "127.0.0.1";
            let config = args.configuration().map_err(|failure| error(failure, address))?;
            let root = config.output_root.clone();
            let network = args.network.config().map_err(|failure| error(failure, address))?;
            let address = network.bind_addr().to_string();
            let session = Session::start(&args, config, network, providers, |_| {}).await.map_err(|failure| error(failure, &address))?;
            let endpoint = session.endpoint();
            Ok(Self {
                inner: Arc::new(Mutex::new(session)),
                accepting: AtomicBool::new(true),
                endpoint,
                root,
            })
        })
    }

    #[getter]
    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    #[getter]
    fn root(&self) -> PathBuf {
        self.root.clone()
    }

    fn export_pairing(&self, py: Python<'_>, path: PathBuf, label: String) -> PyResult<()> {
        let result = py.detach(|| {
            get_runtime().block_on(async {
                let session = self.inner.lock().await;
                if self.accepting.load(Ordering::Acquire) { session.export_pairing(&path, label).await } else { Err(Error::NotAccepting) }
            })
        });
        result.map_err(|failure| error(failure, &self.endpoint))
    }

    fn close<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.accepting.store(false, Ordering::Release);
        let inner = self.inner.clone();
        let address = self.endpoint.clone();
        future_into_py(py, async move {
            let result = inner.lock().await.shutdown().await;
            result.map_err(|failure| error(failure, &address))
        })
    }
}

/* === Public Functions === */

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyBackend>()?;
    module.add_class::<PyEmbeddedProvider>()
}

/* === Private Functions === */

fn error(failure: Error, address: &str) -> PyErr {
    Python::attach(|py| {
        let create = || -> PyResult<PyErr> {
            let errors = py.import("dokime.backend")?;
            let message = failure.to_string();
            let mut source: Option<&(dyn std::error::Error + 'static)> = Some(&failure);
            let mut causes = Vec::new();
            let mut exception = None;
            while let Some(cause) = source {
                causes.push(cause.to_string());
                if exception.is_none() {
                    exception = match cause.downcast_ref::<Error>() {
                        Some(Error::BackendRunning { root }) => Some(errors.getattr("BackendRunning")?.call1((&message, root))?),
                        Some(Error::NotAccepting) => Some(errors.getattr("BackendClosed")?.call1((&message,))?),
                        _ => None,
                    };
                }
                if exception.is_none()
                    && let Some(dokime::runtime::Error::PluginIdConflict { plugin }) = cause.downcast_ref::<dokime::runtime::Error>()
                {
                    exception = Some(errors.getattr("PluginIdConflict")?.call1((&message, plugin))?);
                }
                if exception.is_none()
                    && let Some(io) = cause.downcast_ref::<std::io::Error>()
                    && matches!(io.kind(), std::io::ErrorKind::AddrInUse | std::io::ErrorKind::AddrNotAvailable)
                {
                    exception = Some(errors.getattr("BackendAddressUnavailable")?.call1((&message, address))?);
                }
                source = next_source(cause);
            }
            let exception = exception.unwrap_or(errors.getattr("BackendError")?.call1((&message,))?);
            let exception = PyErr::from_value(exception);
            if causes.len() > 1 {
                exception.set_cause(py, Some(PyRuntimeError::new_err(causes[1..].join(": "))));
            }
            Ok(exception)
        };
        create().unwrap_or_else(|failure| failure)
    })
}

// Transparent Variants And Shared Errors Can Hide Their Own Concrete Type From source().
fn next_source<'a>(error: &'a (dyn std::error::Error + 'static)) -> Option<&'a (dyn std::error::Error + 'static)> {
    if let Some(Error::Runtime(inner)) = error.downcast_ref::<Error>() {
        return Some(inner);
    }
    if let Some(inner) = error.downcast_ref::<Arc<dyn std::error::Error + Send + Sync>>() {
        return Some(inner.as_ref());
    }
    if let Some(inner) = error.downcast_ref::<Arc<dokime::harmos::Error>>() {
        return Some(inner.as_ref());
    }
    error.source()
}

/* === Tests === */
