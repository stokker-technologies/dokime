/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Owned Callback Context Handles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeSet,
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};

// Third Party Imports
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::PyDict,
};
use pyo3_async_runtimes::tokio::future_into_py;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

// Local Imports
use super::{provider::Compiled, schema::Shape};
use dokime::harmos::{
    self, MemorySize, Message,
    operations::{
        CallContext, ContextError, ContextValue, EncodedInvocation, EndpointSet, HostContext, Instance, OperationDeclaration,
        OperationHost, Response, RetryPolicy,
    },
};
use dokime::runtime::EmbeddedHost;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[pyclass(name = "CallbackContext", module = "dokime._native", frozen)]
pub(super) struct PyCallbackContext {
    pub inner: Arc<CallbackContext>,
}

pub(super) struct CallbackContext {
    pub active: Arc<AtomicBool>,
    pub gate: Arc<Mutex<()>>,
    pub host: EmbeddedHost,
    pub compiled: Arc<Compiled>,
    pub source: Source,
    pub streams: Option<dokime::telemetry::Streams>,
    pub allowed_streams: BTreeSet<String>,
    pub parameters: Value,
    pub execution: Option<String>,
    pub phase: String,
    pub actions: Option<tokio::sync::mpsc::Sender<Action>>,
    pub allowed_operations: Option<BTreeSet<String>>,
    pub cancellation: Option<harmos::operations::Cancellation>,
    pub evidence: Option<dokime::execution::ExecutionEvidence>,
    pub children: dokime::execution::Callbacks,
    pub settlement: Option<dokime::execution::Callbacks>,
}

pub(super) struct Action {
    pub kind: String,
    pub payload: Value,
    pub answer: tokio::sync::oneshot::Sender<Result<Value, String>>,
}

#[derive(Clone)]
struct Host {
    source: Source,
    route: String,
    context: CallContext,
    gate: Arc<Mutex<()>>,
    active: Arc<AtomicBool>,
    children: dokime::execution::Callbacks,
    settlement: Option<dokime::execution::Callbacks>,
}

struct Opaque(Vec<u8>);
struct Encoded;
struct Limited(Vec<u8>);
struct Owned {
    inner: Option<EncodedInvocation>,
    settled: CancellationToken,
}
struct Running<O: OperationDeclaration> {
    inner: Owned,
    cancellation: CancellationToken,
    marker: std::marker::PhantomData<fn() -> O>,
}

#[pyclass(name = "OperationInstance", module = "dokime._native", frozen)]
pub(super) struct PyOperationInstance {
    inner: Arc<tokio::sync::Mutex<Option<Joining>>>,
    cancellation: CancellationToken,
    id: u64,
    response: Shape,
}

struct Joining {
    task: tokio::task::JoinHandle<harmos::Result<Response<Opaque>>>,
    cancellation: CancellationToken,
}

struct Completion {
    cancellation: CancellationToken,
    settled: CancellationToken,
}

/* === Enums === */

#[derive(Clone)]
pub(super) enum Source {
    Plugin(harmos::operations::Caller),
    Operation(Arc<HostContext>),
}

/* === Traits === */

/* === Implementations === */

impl std::io::Write for Limited {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) > 64 * 1024 * 1024 {
            return Err(std::io::Error::other("criterion evidence snapshot exceeds bounded workspace"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl PyCallbackContext {
    pub fn plugin(host: EmbeddedHost, compiled: Arc<Compiled>, streams: Option<dokime::telemetry::Streams>) -> Self {
        let source = Source::Plugin(host.caller.clone());
        let allowed_streams = compiled.streams.keys().cloned().collect();
        Self {
            inner: Arc::new(CallbackContext {
                active: Arc::new(AtomicBool::new(true)),
                gate: Arc::new(Mutex::new(())),
                host,
                compiled,
                source,
                streams,
                allowed_streams,
                parameters: Value::Object(Default::default()),
                execution: None,
                phase: "plugin".into(),
                actions: None,
                allowed_operations: None,
                cancellation: None,
                evidence: None,
                children: Default::default(),
                settlement: None,
            }),
        }
    }

    pub fn operation(source: Arc<HostContext>, host: EmbeddedHost, compiled: Arc<Compiled>) -> Self {
        let mut context = Self::plugin(host, compiled, None);
        let inner = Arc::get_mut(&mut context.inner).expect("new callback context");
        inner.source = Source::Operation(source);
        if let Source::Operation(context) = &inner.source {
            inner.cancellation = Some(context.borrow().scope().cancellation().clone());
            inner.execution = dokime::execution::ExecutionScope::current(context.borrow().context()).map(|scope| scope.execution);
        }
        inner.phase = "operation".into();
        context
    }

    pub fn template(
        host: EmbeddedHost, compiled: Arc<Compiled>, streams: Option<dokime::telemetry::Streams>, scope: dokime::execution::ExecutionScope,
        parameters: Value, phase: &str, allowed_streams: BTreeSet<String>,
    ) -> Result<(Self, tokio::sync::mpsc::Receiver<Action>), String> {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        let mut context = Self::plugin(host, compiled, streams);
        let inner = Arc::get_mut(&mut context.inner).expect("new callback context");
        inner.source = inner.source.with_value(scope.clone()).map_err(|error| error.to_string())?;
        inner.execution = Some(scope.execution);
        inner.parameters = parameters;
        inner.phase = phase.into();
        inner.allowed_streams = allowed_streams;
        inner.actions = Some(sender);
        Ok((context, receiver))
    }
}

impl CallbackContext {
    fn ensure_active(&self) -> PyResult<()> {
        if self.active.load(Ordering::Acquire) { Ok(()) } else { Err(PyRuntimeError::new_err("callback context is closed")) }
    }

    fn declaration(&self, stream: &str) -> PyResult<&super::provider::StreamSpec> {
        self.ensure_active()?;
        if !matches!(self.phase.as_str(), "step" | "observer" | "plugin" | "operation") {
            return Err(PyRuntimeError::new_err("this callback cannot publish streams"));
        }
        if !self.allowed_streams.contains(stream) {
            return Err(PyValueError::new_err(format!("undeclared stream {stream}")));
        }
        self.compiled.streams.get(stream).ok_or_else(|| PyValueError::new_err(format!("undeclared stream {stream}")))
    }

    async fn publish(self: Arc<Self>, stream: String, rows: super::columns::Rows) -> PyResult<usize> {
        let completion = {
            let _gate = self.gate.lock().expect("callback admission gate is healthy");
            self.ensure_active()?;
            let completion = Arc::new(Completion {
                cancellation: CancellationToken::new(),
                settled: CancellationToken::new(),
            });
            self.children.register(completion.clone());
            if let Some(settlement) = &self.settlement {
                settlement.register(completion.clone());
            }
            completion
        };
        let _settled = completion.settled.clone().drop_guard();
        tokio::select! {
            biased;
            _ = completion.cancellation.cancelled() => Err(PyRuntimeError::new_err("callback context is closed")),
            result = self.publish_rows(stream, rows) => result,
        }
    }

    async fn publish_rows(&self, stream: String, rows: super::columns::Rows) -> PyResult<usize> {
        self.ensure_active()?;
        let stream = dokime::Identifier::new(stream).map_err(value_error)?;
        let count = rows.len();
        if let Some(streams) = &self.streams {
            streams.publish_rows(&stream, rows).await.map_err(runtime_error)?;
        } else {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(runtime_error)?.as_micros();
            let rows = rows
                .into_iter()
                .map(|(values, storage_payload)| dokime::execution::StampedRow {
                    values,
                    storage_payload: Some(storage_payload),
                    published_unix_micros: now as i64,
                })
                .collect();
            self.host.publish(None, &stream, rows).await.map_err(runtime_error)?;
        }
        Ok(count)
    }
}

#[pymethods]
impl PyCallbackContext {
    fn evidence(&self, py: Python<'_>) -> PyResult<String> {
        self.inner.ensure_active()?;
        let evidence = self.inner.evidence.as_ref().ok_or_else(|| PyValueError::new_err("captured evidence belongs to criteria"))?;
        py.detach(|| {
            let mut output = Limited(Vec::new());
            serde_json::to_writer(&mut output, evidence).map_err(value_error)?;
            String::from_utf8(output.0).map_err(value_error)
        })
    }
    #[getter]
    fn cancelled(&self) -> bool {
        !self.inner.active.load(Ordering::Acquire)
    }
    #[getter]
    fn parameters(&self) -> String {
        self.inner.parameters.to_string()
    }
    #[getter]
    fn settings(&self) -> PyResult<String> {
        let values = self
            .inner
            .host
            .configuration
            .settings
            .iter()
            .map(|(key, value)| value.to_plain_json().map(|value| (key.clone(), value)))
            .collect::<Result<serde_json::Map<_, _>, _>>()
            .map_err(value_error)?;
        Ok(Value::Object(values).to_string())
    }
    #[getter]
    fn execution(&self) -> Option<String> {
        self.inner.execution.clone()
    }

    #[pyo3(signature = (operation, request, request_shape, response_shape, retry=None))]
    fn start<'py>(
        &self, py: Python<'py>, operation: String, request: String, request_shape: String, response_shape: String, retry: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.inner.ensure_active()?;
        if self.inner.allowed_operations.as_ref().is_some_and(|allowed| !allowed.contains(&operation)) {
            return Err(PyValueError::new_err(format!("operation {operation} is not declared in this template's requires catalog")));
        }
        let inner = self.inner.clone();
        let request: Value = serde_json::from_str(&request).map_err(value_error)?;
        let request_shape: Shape = serde_json::from_str(&request_shape).map_err(value_error)?;
        let response: Shape = serde_json::from_str(&response_shape).map_err(value_error)?;
        request_shape.validate().map_err(value_error)?;
        response.validate().map_err(value_error)?;
        let retry: Option<RetryPolicy> = retry.map(|retry| serde_json::from_str(&retry).map_err(value_error)).transpose()?;
        let payload = py.detach(|| request_shape.encode(&request)).map_err(value_error)?;
        future_into_py(py, async move {
            inner.ensure_active()?;
            let local = inner.compiled.operations.iter().any(|(spec, _)| spec.id == operation);
            let (route, definition) = inner.host.operation_definition(&operation, local).map_err(operation_error)?;
            let expected = harmos::operations::Contract::from_schema(
                definition.clone(),
                harmos::operations::Codec::Postcard,
                super::schema::contract_schema(&request_shape, &response),
                &Default::default(),
            )
            .map_err(value_error)?;
            if expected.fingerprint() != definition.fingerprint {
                return Err(PyValueError::new_err("request or response differs from the admitted contract"));
            }
            let mut source = inner.source.clone();
            if let Some(scope) = dokime::execution::ExecutionScope::current(&source.context()) {
                source = source.with_value(scope.routed(&operation)).map_err(value_error)?;
            }
            inner.ensure_active()?;
            let host = Host {
                context: source.context(),
                source,
                route,
                gate: inner.gate.clone(),
                active: inner.active.clone(),
                children: inner.children.clone(),
                settlement: inner.settlement.clone(),
            };
            let invocation = harmos::operations::Invocation::<_, Encoded>::new(host, Opaque(payload));
            let running = match retry {
                Some(retry) => invocation.retry(retry).start().await,
                None => invocation.start().await,
            }
            .map_err(operation_error)?;
            let id = running.id().value();
            let cancellation = running.cancellation.clone();
            let joining = Joining {
                task: tokio::spawn(running.join()),
                cancellation: cancellation.clone(),
            };
            Ok(PyOperationInstance {
                inner: Arc::new(tokio::sync::Mutex::new(Some(joining))),
                cancellation,
                id,
                response,
            })
        })
    }

    fn push_columns<'py>(&self, py: Python<'py>, stream: String, columns: &Bound<'py, PyDict>) -> PyResult<Bound<'py, PyAny>> {
        let declaration = self.inner.declaration(&stream)?;
        let snapshot = super::columns::snapshot(py, &declaration.row, columns)?;
        let rows = py.detach(|| super::columns::encode(&declaration.row, snapshot)).map_err(value_error)?;
        future_into_py(py, self.inner.clone().publish(stream, rows))
    }

    fn push<'py>(&self, py: Python<'py>, stream: String, row: String) -> PyResult<Bound<'py, PyAny>> {
        let declaration = self.inner.declaration(&stream)?;
        let rows = py.detach(|| {
            let row = serde_json::from_str(&row).map_err(value_error)?;
            let payload = declaration.row.encode(&row).map_err(value_error)?;
            super::columns::rows(&declaration.row, vec![payload]).map_err(value_error)
        })?;
        future_into_py(py, self.inner.clone().publish(stream, rows))
    }

    fn command<'py>(&self, py: Python<'py>, kind: String, payload: String) -> PyResult<Bound<'py, PyAny>> {
        self.inner.ensure_active()?;
        let sender =
            self.inner.actions.clone().ok_or_else(|| PyRuntimeError::new_err("this callback has no execution action capability"))?;
        let payload = serde_json::from_str(&payload).map_err(value_error)?;
        future_into_py(py, async move {
            let (answer, response) = tokio::sync::oneshot::channel();
            sender.send(Action { kind, payload, answer }).await.map_err(runtime_error)?;
            let value = response.await.map_err(runtime_error)?.map_err(runtime_error)?;
            Ok(value.to_string())
        })
    }
}

#[pymethods]
impl PyOperationInstance {
    #[getter]
    fn id(&self) -> u64 {
        self.id
    }
    fn request_cancel(&self) {
        self.cancellation.cancel();
    }
    fn join<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let response = self.response.clone();
        future_into_py(py, async move {
            let mut joining = inner.lock().await.take().ok_or_else(|| PyRuntimeError::new_err("operation instance was already joined"))?;
            let answer = (&mut joining.task).await.map_err(runtime_error)?.map_err(operation_error)?;
            let value = response.decode(&answer.0).map_err(value_error)?;
            Ok(value.to_string())
        })
    }
}

impl Source {
    fn context(&self) -> CallContext {
        match self {
            Self::Plugin(caller) => OperationHost::context(caller).clone(),
            Self::Operation(context) => context.borrow().context().clone(),
        }
    }
    fn with_value<T: ContextValue>(&self, value: T) -> Result<Self, ContextError> {
        match self {
            Self::Plugin(caller) => OperationHost::with_value(caller, value).map(Self::Plugin),
            Self::Operation(context) => {
                OperationHost::with_value(context.as_ref(), value).map(|context| Self::Operation(Arc::new(context)))
            }
        }
    }
    fn submit(&self, route: &str, payload: Vec<u8>, retry: Option<RetryPolicy>) -> harmos::Result<EncodedInvocation> {
        match self {
            Self::Plugin(caller) => caller.submit_encoded_with(route, payload, retry),
            Self::Operation(context) => context.borrow().submit_encoded_with(route, payload, retry),
        }
    }
}

impl Message for Opaque {
    const CODEC: &'static str = "harmos.native";
    fn encode(&self) -> Vec<u8> {
        self.0.clone()
    }
    fn encoded_size(&self) -> usize {
        self.0.len()
    }
    fn decoding_memory(bytes: &[u8]) -> Result<usize, harmos::MessageError> {
        Ok(bytes.len())
    }
    fn decode(bytes: &[u8]) -> Result<Self, harmos::MessageError> {
        Ok(Self(bytes.to_vec()))
    }
}
impl MemorySize for Opaque {
    fn memory_size(&self) -> usize {
        self.0.capacity()
    }
}
impl harmos::schema::Described for Opaque {
    const NAME: &'static str = "dokime.python.opaque";
    const FIELDS: &'static [harmos::schema::Field] = &[];
    const MARKER: &'static [u8] = &[];
}
impl OperationDeclaration for Encoded {
    type Request = Opaque;
    type Response = Opaque;
    type Input = harmos::operations::NoStream;
    type Output = harmos::operations::NoStream;
    const ID: &'static str = "dokime.python.invocation";
    const POLICY: harmos::OperationPolicy = harmos::builtins::host::Catalog::POLICY;
}

impl OperationHost for Host {
    type Error = harmos::Error;
    type Instance<O: OperationDeclaration> = Running<O>;
    fn context(&self) -> &CallContext {
        &self.context
    }
    fn with_value<T: ContextValue>(&self, value: T) -> Result<Self, ContextError> {
        let mut host = self.clone();
        host.source = host.source.with_value(value)?;
        host.context = host.source.context();
        Ok(host)
    }
    async fn invoke<O: OperationDeclaration, E: EndpointSet>(
        &self, request: O::Request, retry: Option<RetryPolicy>, _: E,
    ) -> harmos::Result<Running<O>> {
        let _gate = self.gate.lock().expect("callback admission gate is healthy");
        if !self.active.load(Ordering::Acquire) {
            return Err(harmos::Error::Closing { domain: "Python callback" });
        }
        let invocation = self.source.submit(&self.route, request.encode(), retry)?;
        let cancellation = CancellationToken::new();
        let completion = Arc::new(Completion {
            cancellation: cancellation.clone(),
            settled: CancellationToken::new(),
        });
        self.children.register(completion.clone());
        if let Some(settlement) = &self.settlement {
            settlement.register(completion.clone());
        }
        Ok(Running {
            inner: Owned {
                inner: Some(invocation),
                settled: completion.settled.clone(),
            },
            cancellation,
            marker: std::marker::PhantomData,
        })
    }
    async fn submit<O: OperationDeclaration>(&self, request: O::Request) -> harmos::Result<Running<O>> {
        self.invoke::<O, ()>(request, None, ()).await
    }
    async fn submit_owned<O: OperationDeclaration>(
        &self, _: O::Request, _: harmos::operations::RequestClaims,
    ) -> harmos::Result<Running<O>> {
        Err(harmos::Error::InvalidInput {
            field: "operation resources",
            reason: "Python calls do not lend native resources",
        })
    }
    async fn publish_on<S: harmos::StreamSpec>(&self, _: &S::Row) -> harmos::Result<()>
    where
        S::Row: Message,
    {
        Err(harmos::Error::InvalidInput {
            field: "publication",
            reason: "Python publication uses its declared stream handle",
        })
    }
}

impl<O: OperationDeclaration> Instance<O> for Running<O> {
    type Error = harmos::Error;
    fn id(&self) -> harmos::operations::instance::Id {
        self.inner.inner.as_ref().expect("owned invocation").id()
    }
    fn status(&self) -> impl Future<Output = harmos::Result<harmos::operations::instance::Status>> + Send {
        std::future::ready(Ok(self.inner.inner.as_ref().expect("owned invocation").status().portable()))
    }
    fn request_cancel(&self) -> impl Future<Output = harmos::Result<()>> + Send {
        self.cancellation.cancel();
        self.inner.inner.as_ref().expect("owned invocation").request_cancel();
        std::future::ready(Ok(()))
    }
    async fn join(mut self) -> harmos::Result<Response<O::Response>> {
        let bytes = tokio::select! {
            result = &mut self.inner => result?,
            _ = self.cancellation.cancelled() => {
                self.inner.inner.as_ref().expect("owned invocation").request_cancel();
                self.inner.await?
            }
        };
        O::Response::decode(&bytes).map(Response::new).map_err(|source| harmos::Error::OperationPayload { operation: O::ID.into(), source })
    }
    fn detach(self) -> harmos::operations::instance::Id {
        let id = self.id();
        pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
            let _ = self.join().await;
        });
        id
    }
}
impl Future for Owned {
    type Output = harmos::Result<Vec<u8>>;
    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let result = Pin::new(self.inner.as_mut().expect("owned invocation")).poll(context);
        if result.is_ready() {
            self.inner.take();
            self.settled.cancel();
        }
        result
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        if let Some(invocation) = self.inner.take() {
            invocation.request_cancel();
            let settled = self.settled.clone().drop_guard();
            pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
                let _settled = settled;
                let _ = invocation.await;
            });
        }
    }
}

impl Drop for Joining {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

impl dokime::execution::CallbackTask for Completion {
    fn is_settled(&self) -> bool {
        self.settled.is_cancelled()
    }
    fn cancel(&self) {
        self.cancellation.cancel();
    }
    fn join(&self) -> Pin<Box<dyn Future<Output = Result<(), dokime::diagnostics::Diagnostic>> + Send + '_>> {
        Box::pin(async move {
            self.settled.cancelled().await;
            Ok(())
        })
    }
}

/* === Public Functions === */

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyCallbackContext>()?;
    module.add_class::<PyOperationInstance>()
}

/* === Private Functions === */

fn value_error(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}
fn runtime_error(error: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

fn operation_error(error: harmos::Error) -> PyErr {
    let mut message = error.to_string();
    let mut source = std::error::Error::source(&error);
    while let Some(error) = source {
        message.push_str(": ");
        message.push_str(&error.to_string());
        source = error.source();
    }
    PyRuntimeError::new_err(message)
}

/* === Tests === */
