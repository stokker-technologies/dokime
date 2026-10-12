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
use std::{path::PathBuf, sync::Arc};
// Third Party Imports
use pyo3::prelude::*;
use tokio::sync::Mutex;
// Local Imports
use crate::{PyRuntime, authoring::Provider, to_value_error};
use dokime_app::{Runtime, config::ApplicationOptions};
use dokime_server::grpc::Server;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct State {
    options: ApplicationOptions,
    provider: Option<Arc<Provider>>,
    runtime: Option<Runtime>,
    failed_start: Option<dokime_app::Error>,
    server: Option<Server>,
    closed: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[pymethods]
impl PyRuntime {
    #[new]
    #[pyo3(signature = (storage_root = None, application_root = None, plugins_root = None))]
    fn new(
        storage_root: Option<PathBuf>,
        application_root: Option<PathBuf>,
        plugins_root: Option<PathBuf>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(State {
                options: ApplicationOptions {
                    application_root,
                    storage_root,
                    plugins_root,
                },
                provider: None,
                runtime: None,
                failed_start: None,
                server: None,
                closed: false,
            })),
        }
    }

    fn register<'py>(&self, py: Python<'py>, provider: Py<PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let provider = Arc::new(Provider::prepare(py, provider)?);
        let state = self.inner.clone();
        let task = pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
            let mut state = state.lock().await;
            state.open()?;
            if let Some(runtime) = state.runtime.as_mut() {
                runtime.register(provider).await.map_err(to_value_error)?;
            } else {
                state.provider = Some(provider);
            }

            Ok(())
        });
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            task.await.map_err(to_value_error)?
        })
    }

    #[pyo3(signature = (server_config = None))]
    fn start<'py>(
        &self,
        py: Python<'py>,
        server_config: Option<&crate::server::config::PyServerConfig>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = pyo3_async_runtimes::tokio::get_current_loop(py)?;
        let server_config = server_config
            .map(|config| config.take())
            .transpose()?
            .unwrap_or_else(dokime_server::grpc::Config::local);
        let state = self.inner.clone();
        let task = pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
            let mut state = state.lock().await;
            state.open()?;
            if state.runtime.is_some() {
                return Err(to_value_error("runtime is already started"));
            }
            let mut config = state.options.clone().resolve().map_err(to_value_error)?;
            if let Some(provider) = state.provider.take() {
                config = config.register(provider);
            }
            match dokime_app::app::launch(config).await {
                Ok(runtime) => state.runtime = Some(runtime),
                Err(error @ dokime_app::Error::StartupCleanup { .. }) => {
                    let message = error.to_string();
                    state.failed_start = Some(error);
                    return Err(to_value_error(message));
                }
                Err(error) => return Err(to_value_error(error)),
            }
            let handle = state.runtime.as_ref().expect("started runtime").handle();
            match Server::start(handle, server_config).await {
                Ok(server) => {
                    let endpoint = server.local_endpoint();
                    state.server = Some(server);
                    Ok(endpoint)
                }
                Err(error) => {
                    let failure = error.to_string();
                    state.close().await?;
                    Err(to_value_error(failure))
                }
            }
        });
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            task.await.map_err(to_value_error)?
        })
    }

    fn close<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let _ = pyo3_async_runtimes::tokio::get_current_loop(py)?;
        let state = self.inner.clone();
        // The operation remains owned if the Python awaiting task is cancelled.
        let task = pyo3_async_runtimes::tokio::get_runtime()
            .spawn(async move { state.lock().await.close().await });
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            task.await.map_err(to_value_error)?
        })
    }
}

impl State {
    fn open(&self) -> PyResult<()> {
        if self.closed {
            Err(to_value_error("runtime is closed"))
        } else if self.failed_start.is_some() {
            Err(to_value_error(
                "runtime initialization cleanup is pending; close the runtime",
            ))
        } else {
            Ok(())
        }
    }

    async fn close(&mut self) -> PyResult<()> {
        self.closed = true;
        self.provider = None;
        let mut failures = Vec::new();
        if let Some(server) = self.server.take()
            && let Err(error) = server.shutdown().await
        {
            failures.push(error.to_string());
        }
        if let Some(runtime) = self.runtime.as_mut() {
            match runtime.shutdown().await {
                Ok(_) => {
                    self.runtime = None;
                }
                Err(error) => failures.push(error.to_string()),
            }
        }

        if let Some(dokime_app::Error::StartupCleanup { rollback, .. }) = &self.failed_start {
            match rollback.retry().await {
                Ok(()) => self.failed_start = None,
                Err(error) => failures.push(error.to_string()),
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(to_value_error(failures.join("; ")))
        }
    }
}

impl Drop for PyRuntime {
    fn drop(&mut self) {
        if let Ok(state) = self.inner.try_lock()
            && state.runtime.is_none()
            && state.failed_start.is_none()
        {
            return;
        }
        let state = self.inner.clone();
        pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
            if let Err(error) = state.lock().await.close().await {
                Python::attach(|py| error.write_unraisable(py, None));
            }
        });
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
