/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cancellable Python Callbacks

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

// Third Party Imports
use pyo3::prelude::*;
use pyo3_async_runtimes::{TaskLocals, into_future_with_locals};
use serde_json::Value;

// Local Imports
use super::context::PyCallbackContext;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(super) struct Callback {
    pub name: String,
    pub function: Arc<Py<PyAny>>,
    pub locals: Arc<TaskLocals>,
}

struct Pending {
    job: Py<PyAny>,
    active: Arc<AtomicBool>,
    grace: Duration,
    completed: bool,
    cancellation_sent: bool,
    gate: Arc<std::sync::Mutex<()>>,
    children: dokime::execution::Callbacks,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Callback {
    pub async fn call(&self, context: PyCallbackContext, input: Value, grace: Duration) -> Result<Value, String> {
        let active = context.inner.active.clone();
        let gate = context.inner.gate.clone();
        let cancellation = context.inner.cancellation.clone();
        let children = context.inner.children.clone();
        let (mut pending, future) = Python::attach(|py| -> PyResult<_> {
            let bridge = py.import("dokime._callbacks")?;
            let context = Py::new(py, context)?;
            let job = bridge.call_method1(
                "start",
                (self.locals.event_loop(py), self.function.as_ref().bind(py), context, input.to_string(), &self.name),
            )?;
            let awaitable = bridge.call_method1("wait", (&job,))?;
            let future = into_future_with_locals(&self.locals, awaitable)?;
            let pending = Pending {
                job: job.unbind(),
                active,
                grace,
                completed: false,
                cancellation_sent: false,
                gate,
                children: children.clone(),
            };
            Ok((pending, future))
        })
        .map_err(traceback)?;
        tokio::pin!(future);
        let result = if let Some(cancellation) = cancellation {
            tokio::select! {
                answer = &mut future => answer,
                _ = cancellation.cancelled() => { pending.cancel(); future.await }
            }
        } else {
            future.await
        };
        pending.completed = true;
        pending.close();
        let failures = children.settle().await;
        if let Some(failure) = failures.first() {
            return Err(failure.to_string());
        }
        let value = result.map_err(traceback)?;
        Python::attach(|py| {
            let json: String = value.extract(py).map_err(traceback)?;
            serde_json::from_str(&json).map_err(|error| error.to_string())
        })
    }
}

impl Drop for Pending {
    fn drop(&mut self) {
        self.close();
        if self.completed || self.cancellation_sent {
            return;
        }
        self.cancel();
    }
}

impl Pending {
    fn close(&self) {
        self.active.store(false, Ordering::Release);
        let _gate = self.gate.lock().expect("callback admission gate is healthy");
        self.children.cancel();
    }
    fn cancel(&mut self) {
        self.cancellation_sent = true;
        self.close();
        Python::attach(|py| {
            let result = py
                .import("dokime._callbacks")
                .and_then(|bridge| bridge.call_method1("cancel", (self.job.bind(py), self.grace.as_secs_f64())));
            if let Err(error) = result {
                error.write_unraisable(py, Some(self.job.bind(py)));
            }
        });
    }
}

/* === Public Functions === */

pub(super) fn traceback(error: PyErr) -> String {
    Python::attach(|py| {
        let formatted = || -> PyResult<String> {
            let traceback = py.import("traceback")?;
            let lines = traceback.call_method1("format_exception", (error.value(py),))?;
            lines.extract::<Vec<String>>().map(|lines| lines.join(""))
        };
        formatted().unwrap_or_else(|_| error.to_string())
    })
}

/* === Private Functions === */

/* === Tests === */
