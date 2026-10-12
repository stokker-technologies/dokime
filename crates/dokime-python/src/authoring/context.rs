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
use std::collections::BTreeMap;
use std::sync::Arc;
// Third Party Imports
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods};
use serde::Deserialize;
use serde_json::Value as Json;
use tokio::sync::{Mutex, mpsc, oneshot, watch};
use tokio_stream::StreamExt;
// Local Imports
use super::{Binding, Own, Template, callbacks};
use crate::{Error as RequestError, pyserde, to_value_error};
use dokime::core::{Identifier, PromptId, RecordScope, Value};
use dokime::diagnostics::Diagnostic;
use dokime::execution::listen::{ListeningEvidence, ListeningValues};
use dokime::execution::*;
use dokime::prompts::{Prompt, PromptDefinition, PromptRequest, PromptResponse};
use dokime::publication::{Publication, Publishable};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[pyclass(name = "AuthoringContext", module = "dokime._native")]
pub(crate) struct PyContext {
    inner: ContextState,
}

struct ContextState {
    commands: mpsc::Sender<Request>,
    parameters: Json,
    evidence: Json,
    artifacts: Option<dokime::artifacts::Artifacts>,
}

struct Request {
    operation: String,
    payload: Json,
    reply: oneshot::Sender<Result<Json, RequestError>>,
}

#[derive(Default)]
struct Handles<'a> {
    next: u64,
    progress: BTreeMap<u64, Progress<std::ops::Range<usize>>>,
    leases: BTreeMap<u64, Lease<'a, ()>>,
    listeners: BTreeMap<u64, Arc<Listener>>,
}

struct Listener {
    stream: Mutex<Subscription>,
    closed: watch::Sender<bool>,
}

struct Published(Publication);

#[derive(Deserialize)]
struct Invocation {
    definition: PromptDefinition,
    values: BTreeMap<String, Value>,
}

/* === Enums === */
enum Subscription {
    Rows(ListeningValues),
    Records(ListeningEvidence),
}

pub(super) enum Bridge<'c, 'a> {
    Initialize(&'c mut InitializationContext<'a, Template, Own>),
    Traceability(&'c TraceabilityContext<'a, Template, Own>),
    Step(&'c mut StepContext<'a, Template, Own>),
    Cleanup(&'c mut CleanupContext<'a, Template, Own>),
    Criterion(&'c CriterionContext<'a, Template>),
}

/* === Traits === */

/* === Implementations === */
impl Subscription {
    async fn next(&mut self) -> Result<Json, RequestError> {
        match self {
            Self::Rows(rows) => serde_json::to_value(rows.next().await),
            Self::Records(records) => {
                serde_json::to_value(records.next().await.map(|record| record.details))
            }
        }
        .map_err(|error| RequestError::Invalid(error.to_string()))
    }
}

impl Handles<'_> {
    fn id(&mut self) -> u64 {
        self.next += 1;
        self.next
    }
}

impl Drop for Handles<'_> {
    fn drop(&mut self) {
        for listener in self.listeners.values() {
            let _ = listener.closed.send(true);
        }
    }
}

#[pymethods]
impl PyContext {
    #[getter]
    fn artifacts(&self) -> PyResult<super::artifacts::PyArtifacts> {
        let inner = self
            .inner
            .artifacts
            .clone()
            .ok_or_else(|| to_value_error("criterion contexts cannot produce artifacts"))?;
        Ok(super::artifacts::PyArtifacts { inner })
    }

    #[getter]
    fn parameters(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        pyserde::to_py(py, &self.inner.parameters)
    }

    #[getter]
    fn evidence(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        pyserde::to_py(py, &self.inner.evidence)
    }

    fn request<'py>(
        &self,
        py: Python<'py>,
        operation: String,
        payload: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let value_field = match operation.as_str() {
            "publish_record" => Some("details"),
            "publish_row" | "ask" => Some("values"),
            _ => None,
        };
        let payload = if let Some(field) = value_field {
            let copied = payload.cast::<PyDict>()?.copy()?;
            let values = copied
                .get_item(field)?
                .ok_or_else(|| to_value_error("missing authored values"))?;
            let values = pyserde::value_map_from_py(&values)?;
            copied.set_item(field, pyserde::to_py(py, &values)?)?;
            pyserde::py_to_json(copied.as_any())?
        } else {
            pyserde::py_to_json(&payload)?
        };
        let commands = self.inner.commands.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (reply, response) = oneshot::channel();
            let closing = matches!(
                operation.as_str(),
                "lease_close" | "listen_close" | "progress_close"
            );
            let sent = commands
                .send(Request {
                    operation,
                    payload,
                    reply,
                })
                .await;
            let result = if sent.is_err() && closing {
                Json::Null
            } else {
                sent.map_err(|_| to_value_error("execution context is closed"))?;
                match response.await {
                    Ok(result) => result.map_err(RequestError::into_pyerr)?,
                    Err(_) if closing => Json::Null,
                    Err(_) => return Err(to_value_error("execution context is closed")),
                }
            };

            Python::attach(|py| pyserde::to_py(py, &result))
        })
    }
}

impl<'c, 'a> Bridge<'c, 'a> {
    fn parameters(&self) -> &BTreeMap<String, Value> {
        match self {
            Self::Initialize(ctx) => ctx.raw_parameters(),
            Self::Traceability(ctx) => ctx.raw_parameters(),
            Self::Step(ctx) => ctx.raw_parameters(),
            Self::Cleanup(ctx) => ctx.raw_parameters(),
            Self::Criterion(ctx) => ctx.raw_parameters(),
        }
    }

    fn callbacks(&self) -> &Callbacks {
        match self {
            Self::Initialize(ctx) => ctx.environment().callbacks(),
            Self::Traceability(ctx) => ctx.environment().callbacks(),
            Self::Step(ctx) => ctx.environment().callbacks(),
            Self::Cleanup(ctx) => ctx.environment().callbacks(),
            Self::Criterion(ctx) => ctx.callbacks(),
        }
    }

    fn log(&self, level: &str, message: &str) -> Result<(), String> {
        macro_rules! log {
            ($ctx:expr) => {
                match level {
                    "trace" => $ctx.trace(message),
                    "debug" => $ctx.debug(message),
                    "info" => $ctx.info(message),
                    "warning" => $ctx.warn(message),
                    "error" => $ctx.error(message),
                    _ => return Err(format!("unknown diagnostic level {level:?}")),
                }
            };
        }
        match self {
            Self::Initialize(ctx) => log!(ctx),
            Self::Traceability(ctx) => log!(ctx),
            Self::Step(ctx) => log!(ctx),
            Self::Cleanup(ctx) => log!(ctx),
            Self::Criterion(ctx) => log!(ctx),
        }

        Ok(())
    }

    async fn handle(
        &mut self,
        operation: &str,
        payload: Json,
        handles: &mut Handles<'a>,
    ) -> Result<Json, RequestError> {
        match operation {
            "progress" => {
                let Self::Step(ctx) = self else {
                    return Err("progress requires a step context".into());
                };
                #[derive(Deserialize)]
                struct Work {
                    phase: String,
                    total: usize,
                }
                let work: Work =
                    serde_json::from_value(payload).map_err(|error| error.to_string())?;
                let id = handles.id();
                handles
                    .progress
                    .insert(id, ctx.progress(work.phase, 0..work.total));
                return Ok(Json::from(id));
            }
            "progress_next" => {
                let id: u64 = serde_json::from_value(payload).map_err(|error| error.to_string())?;
                let progress = handles
                    .progress
                    .get_mut(&id)
                    .ok_or("progress iterator is closed")?;
                return Ok(progress.next().map(Json::from).unwrap_or(Json::Null));
            }
            "progress_close" => {
                let id: u64 = serde_json::from_value(payload).map_err(|error| error.to_string())?;
                handles.progress.remove(&id);
            }
            "lease" => {
                let Self::Step(ctx) = self else {
                    return Err("leasing requires a step context".into());
                };
                let stream: Identifier =
                    serde_json::from_value(payload).map_err(|error| error.to_string())?;
                let lease = ctx
                    .lease_stream::<()>(stream)
                    .await
                    .map_err(|error| error.to_string())?;
                let id = handles.id();
                handles.leases.insert(id, lease);
                return Ok(Json::from(id));
            }
            "lease_close" => {
                let id =
                    serde_json::from_value::<u64>(payload).map_err(|error| error.to_string())?;
                handles.leases.remove(&id);
            }
            "listen" => {
                let Self::Step(ctx) = self else {
                    return Err("listening requires a step context".into());
                };
                #[derive(Deserialize)]
                struct Selection {
                    kind: String,
                    id: Identifier,
                }
                let selection: Selection =
                    serde_json::from_value(payload).map_err(|error| error.to_string())?;
                let stream = match selection.kind.as_str() {
                    "row" => Subscription::Rows(ListeningValues::new(
                        ctx.environment().streams(),
                        selection.id,
                    )),
                    "record" => Subscription::Records(ListeningEvidence::new(
                        ctx.environment().deliveries(),
                        selection.id.to_string(),
                    )),
                    _ => return Err("listener requires a row or record declaration".into()),
                };
                let id = handles.id();
                handles.listeners.insert(
                    id,
                    Arc::new(Listener {
                        stream: Mutex::new(stream),
                        closed: watch::channel(false).0,
                    }),
                );
                return Ok(Json::from(id));
            }
            "listen_close" => {
                let id =
                    serde_json::from_value::<u64>(payload).map_err(|error| error.to_string())?;
                if let Some(listener) = handles.listeners.remove(&id) {
                    let _ = listener.closed.send(true);
                }
            }
            "log" => {
                #[derive(Deserialize)]
                struct Log {
                    level: String,
                    message: String,
                }
                let log: Log = serde_json::from_value(payload).map_err(|e| e.to_string())?;
                self.log(&log.level, &log.message)?;
            }
            "criterion_evidence" => {
                let Self::Criterion(ctx) = self else {
                    return Err("evidence selection requires a criterion context".into());
                };
                let phase = serde_json::from_value::<dokime::core::CapturePhase>(payload)
                    .map_err(|error| error.to_string())?;
                return serde_json::to_value(ctx.evidence_from(phase))
                    .map_err(|error| RequestError::Invalid(error.to_string()));
            }
            "publish_record" => {
                let Self::Step(ctx) = self else {
                    return Err("publishing requires a step context".into());
                };
                let record =
                    serde_json::from_value::<dokime::records::RecordPublishRequest>(payload)
                        .map_err(|e| e.to_string())?;
                ctx.publish(record).await;
            }
            "publish_row" => {
                let Self::Step(ctx) = self else {
                    return Err("publishing requires a step context".into());
                };
                #[derive(Deserialize)]
                struct Row {
                    stream: Identifier,
                    values: BTreeMap<String, Value>,
                }
                let row: Row = serde_json::from_value(payload).map_err(|e| e.to_string())?;
                ctx.publish(Published(Publication::Row {
                    stream: row.stream,
                    values: row.values,
                }))
                .await;
            }
            "ask" => {
                let Self::Step(ctx) = self else {
                    return Err("asking requires a step context".into());
                };
                let invocation =
                    serde_json::from_value::<Invocation>(payload).map_err(|e| e.to_string())?;
                let response = ctx.ask(invocation).await.map_err(RequestError::from)?;
                return serde_json::to_value(response)
                    .map_err(|error| RequestError::Invalid(error.to_string()));
            }
            _ => return Err(format!("unknown authoring operation {operation:?}").into()),
        }

        Ok(Json::Null)
    }
}

impl Publishable for Published {
    type Payload = Publication;
    fn publication(self) -> Self::Payload {
        self.0
    }
}

impl Prompt for Invocation {
    type Response = PromptResponse;
    fn definition(&self) -> PromptDefinition {
        self.definition.clone()
    }
    fn request(&self) -> PromptRequest {
        PromptRequest {
            prompt_id: PromptId::new(),
            prompt_type: self.definition.prompt_type.clone(),
            values: self.values.clone(),
            scope: RecordScope::default(),
        }
    }
    fn decode(&self, response: PromptResponse) -> dokime::Result<PromptResponse> {
        dokime::prompts::form_answer(&self.definition, &response)?;
        Ok(response)
    }
}

/* === Public Functions === */
pub(super) async fn call(
    binding: &Binding,
    target: &Py<PyAny>,
    method: &str,
    selector: Option<&str>,
    mut bridge: Bridge<'_, '_>,
) -> Result<Py<PyAny>, Diagnostic> {
    let (commands, mut requests) = mpsc::channel::<Request>(32);
    let parameters =
        serde_json::to_value(bridge.parameters()).map_err(|e| Diagnostic::of(e.to_string()))?;
    let evidence = match &bridge {
        Bridge::Criterion(ctx) => {
            serde_json::to_value(ctx.evidence()).map_err(|e| Diagnostic::of(e.to_string()))?
        }
        _ => Json::Null,
    };
    let artifacts = match &bridge {
        Bridge::Initialize(ctx) => Some(ctx.artifacts().clone()),
        Bridge::Traceability(ctx) => Some(ctx.artifacts().clone()),
        Bridge::Step(ctx) => Some(ctx.artifacts().clone()),
        Bridge::Cleanup(ctx) => Some(ctx.artifacts().clone()),
        Bridge::Criterion(_) => None,
    };
    let task = Python::attach(|py| {
        let context = Py::new(
            py,
            PyContext {
                inner: ContextState {
                    commands,
                    parameters,
                    evidence,
                    artifacts,
                },
            },
        )?;
        let coroutine = match selector {
            Some(selector) => target.call_method1(py, method, (selector, context))?,
            None => target.call_method1(py, method, (context,))?,
        };
        callbacks::Task::start(
            py,
            &binding.locals,
            bridge.callbacks(),
            coroutine.into_bound(py),
        )
    })
    .map_err(callbacks::diagnostic)?;
    let mut handles = Handles::default();
    let mut waits = tokio::task::JoinSet::new();
    let result = task.result();
    tokio::pin!(result);

    loop {
        tokio::select! {
            result = &mut result => return result,
            Some(_) = waits.join_next(), if !waits.is_empty() => {},
            Some(request) = requests.recv() => {
                if request.reply.is_closed() { continue; }
                if request.operation == "listen_next" {
                    let id = request.payload.as_u64();
                    let listener = id.and_then(|id| handles.listeners.get(&id).cloned());
                    if let Some(listener) = listener {
                        waits.spawn(async move {
                            let mut reply = request.reply;
                            let mut closed = listener.closed.subscribe();
                            let result = if *closed.borrow() {
                                Ok(Json::Null)
                            } else {
                                tokio::select! {
                                    biased;
                                    _ = reply.closed() => return,
                                    _ = closed.changed() => Ok(Json::Null),
                                    result = async { listener.stream.lock().await.next().await } => result,
                                }
                            };
                            let _ = reply.send(result);
                        });
                    } else {
                        let _ = request.reply.send(Ok(Json::Null));
                    }
                } else {
                    let response = bridge.handle(&request.operation, request.payload, &mut handles).await;
                    let _ = request.reply.send(response);
                }
            }
        }
    }
}

/* === Private Functions === */

/* === Tests === */
