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
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
// Third Party Imports
use pyo3::prelude::*;
use pyo3_async_runtimes::TaskLocals;
// Local Imports
use dokime::diagnostics::Diagnostic;
use dokime::execution::{CallbackTask, Callbacks};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Task {
    owner: Py<PyAny>,
    locals: TaskLocals,
    settled: AtomicBool,
}

struct CancelOnDrop<'a>(&'a Task);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Task {
    pub(super) fn start(
        py: Python<'_>,
        locals: &TaskLocals,
        callbacks: &Callbacks,
        coroutine: Bound<'_, PyAny>,
    ) -> PyResult<Arc<Self>> {
        let owner = py
            .import("dokime._callbacks")?
            .getattr("Callback")?
            .call1((locals.event_loop(py), locals.context(py), coroutine))?
            .unbind();
        let task = Arc::new(Self {
            owner,
            locals: locals.clone(),
            settled: AtomicBool::new(false),
        });
        callbacks.register(task.clone());
        if let Err(error) = task.owner.call_method0(py, "schedule") {
            task.settled.store(true, Ordering::Release);
            return Err(error);
        }

        Ok(task)
    }

    async fn wait(&self) -> PyResult<Py<PyAny>> {
        let future = Python::attach(|py| {
            pyo3_async_runtimes::into_future_with_locals(
                &self.locals,
                self.owner.call_method0(py, "wait")?.into_bound(py),
            )
        })?;
        let result = future.await;
        self.settled.store(true, Ordering::Release);

        result
    }

    pub(super) async fn result(&self) -> Result<Py<PyAny>, Diagnostic> {
        let _cancel = CancelOnDrop(self);
        self.wait().await.map_err(diagnostic)
    }
}

impl CallbackTask for Task {
    fn is_settled(&self) -> bool {
        self.settled.load(Ordering::Acquire)
    }

    fn cancel(&self) {
        if !self.is_settled() {
            Python::attach(|py| {
                if let Err(error) = self.owner.call_method0(py, "cancel") {
                    error.write_unraisable(py, Some(self.owner.bind(py)));
                }
            });
        }
    }

    fn join(&self) -> Pin<Box<dyn Future<Output = Result<(), Diagnostic>> + Send + '_>> {
        Box::pin(async move {
            if self.is_settled() {
                return Ok(());
            }
            match self.wait().await {
                Ok(_) => Ok(()),
                Err(error) if cancelled(&error) => Ok(()),
                Err(error) => Err(diagnostic(error)),
            }
        })
    }
}

impl Drop for CancelOnDrop<'_> {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

/* === Public Functions === */
pub(super) fn diagnostic(error: PyErr) -> Diagnostic {
    Python::attach(|py| {
        let mut diagnostic = Diagnostic::of(error.to_string());
        if let Ok(traceback) = py
            .import("traceback")
            .and_then(|module| module.call_method1("format_exception", (error.value(py),)))
            .and_then(|lines| lines.extract::<Vec<String>>())
        {
            diagnostic.backtrace = traceback.concat();
        }

        diagnostic
    })
}

/* === Private Functions === */
fn cancelled(error: &PyErr) -> bool {
    Python::attach(|py| {
        py.import("asyncio")
            .and_then(|module| module.getattr("CancelledError"))
            .is_ok_and(|kind| error.is_instance(py, &kind))
    })
}

/* === Tests === */
