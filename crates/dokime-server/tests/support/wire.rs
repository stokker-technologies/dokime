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
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
// Third Party Imports
use tokio::sync::Notify;
use tonic::{
    Request,
    transport::{Channel, Endpoint},
};
// Local Imports
use dokime_client::contracts;
use dokime_domain::model;
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_server::grpc::{Config, Server};
use {
    dokime_domain::model::execution::result::CycleView, dokime_domain::model::execution::result::ExecutionStatus,
    dokime_runtime::ExecutionFixture, dokime_runtime::ExecutionFixtureContext, dokime_runtime::ExecutionFixtureFuture,
    dokime_runtime::Identifier, dokime_runtime::Metadata, dokime_runtime::Runtime, dokime_runtime::transport::ProviderCycleOutcome,
    dokime_runtime::transport::ProviderExecutionOutcome, dokime_runtime::transport::ProviderStatusOutcome,
};

/* === Constants === */
pub const TOKEN: &str = "fixture-token-012345678901234567890123456789";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Keep Per-Request Metadata Available For Wire Authorization Scenarios.
#[derive(Clone, Debug)]
pub struct WireClient {
    pub channel: Channel,
    pub decoding: usize,
    pub encoding: usize,
}

pub struct SelectionAppend {
    pub cycle_id: String,
    pub plan: Option<model::planning::wire::SelectionPlan>,
    pub context: Option<model::execution::wire::MutationContext>,
}

pub struct CaseSelection {
    pub cycle_id: String,
    pub case_id: String,
    pub context: Option<model::execution::wire::MutationContext>,
}

pub struct WireBytes<C> {
    pub codec: C,
}

pub struct BytesEncoder;

pub struct WireStream<T> {
    pub frames: std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<T, tonic::Status>> + Send>>,
}

pub struct Gate {
    pub calls: AtomicUsize,
    pub started: Notify,
    pub release: Notify,
    pub returned: AtomicBool,
    pub outcome: fn(&CycleView) -> ProviderCycleOutcome,
}

/* === Enums === */

/* === Traits === */
pub trait WireMessage: Sized {
    fn wire_bytes(&self) -> Vec<u8>;

    fn from_wire(bytes: &[u8]) -> Result<Self, harmos::MessageError>;
}

impl<T: harmos::domain::Portable> WireMessage for T {
    fn wire_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        <harmos::grpc::Transcoded as harmos::grpc::Wire<T>>::encode(self, &mut bytes).expect("a story message transcodes");
        bytes
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, harmos::MessageError> {
        <harmos::grpc::Transcoded as harmos::grpc::Wire<T>>::decode(harmos_sdk::grpc::DecodeInput::new(bytes))
    }
}

/* === Implementations === */
impl WireClient {
    pub async fn start_first(
        &self, request: impl tonic::IntoRequest<contracts::execution::GetCycle>,
    ) -> Result<tonic::Response<model::execution::wire::RunReply>, tonic::Status> {
        let (metadata, _, request) = request.into_request().into_parts();
        let list = contracts::execution::ListExecutions { cycle_id: request.cycle_id, ..Default::default() };
        let response =
            self.unary::<contracts::execution::ListExecutions>(Request::from_parts(metadata.clone(), Default::default(), list)).await?;
        let executions = response.into_inner().executions;
        let execution = executions.first().ok_or_else(|| tonic::Status::not_found("execution not found"))?;
        let start = contracts::execution::StartExecution {
            execution_id: execution.execution_id.clone(),
            mode: model::execution::wire::ExecutionMode::Automatic,
        };
        self.unary::<contracts::execution::StartExecution>(Request::from_parts(metadata, Default::default(), start)).await
    }

    pub async fn start_case(
        &self, request: impl tonic::IntoRequest<CaseSelection>,
    ) -> Result<tonic::Response<model::execution::wire::RunReply>, tonic::Status> {
        let (metadata, _, request) = request.into_request().into_parts();
        let list = contracts::execution::ListExecutions { cycle_id: request.cycle_id, ..Default::default() };
        let response =
            self.unary::<contracts::execution::ListExecutions>(Request::from_parts(metadata.clone(), Default::default(), list)).await?;
        let executions = response.into_inner().executions;
        let execution = executions
            .iter()
            .rev()
            .find(|execution| execution.case_id == request.case_id)
            .ok_or_else(|| tonic::Status::not_found("execution not found"))?;
        let start = contracts::execution::StartExecution {
            execution_id: execution.execution_id.clone(),
            mode: model::execution::wire::ExecutionMode::Automatic,
        };
        self.unary::<contracts::execution::StartExecution>(Request::from_parts(metadata, Default::default(), start)).await
    }

    pub async fn append_selection(
        &self, request: impl tonic::IntoRequest<SelectionAppend>,
    ) -> Result<tonic::Response<model::execution::wire::AppendCasesReply>, tonic::Status> {
        let (metadata, _, request) = request.into_request().into_parts();
        let mut plan = request.plan.ok_or_else(|| tonic::Status::invalid_argument("selection plan is required"))?;
        if plan.case_ids.is_empty() {
            if plan.cycle_id.is_empty() {
                plan.cycle_id.clone_from(&request.cycle_id);
            } else if plan.cycle_id != request.cycle_id {
                return Err(tonic::Status::invalid_argument("selection plan numbers a different cycle"));
            }
        }
        let preview = if plan.case_ids.is_empty() {
            let register = contracts::planning::RegisterSelection { plan: Some(plan), idempotency_key: uuid::Uuid::now_v7().to_string() };
            let response = self
                .unary::<contracts::planning::RegisterSelection>(Request::from_parts(metadata.clone(), Default::default(), register))
                .await?
                .into_inner();
            response
        } else {
            let preview = contracts::planning::PreviewSelection { plan: Some(plan) };
            self.unary::<contracts::planning::PreviewSelection>(Request::from_parts(metadata.clone(), Default::default(), preview))
                .await?
                .into_inner()
        };
        let executions = preview
            .cases
            .into_iter()
            .map(|case| contracts::execution::ExecutionBinding { execution_id: uuid::Uuid::now_v7().to_string(), case_id: case.case_id })
            .collect();
        let append = contracts::execution::AppendExecutions { cycle_id: request.cycle_id, executions };
        self.unary::<contracts::execution::AppendExecutions>(Request::from_parts(metadata, Default::default(), append)).await
    }

    pub fn new(channel: Channel) -> Self {
        Self { channel, decoding: 4 * 1024 * 1024, encoding: u32::MAX as usize }
    }

    pub async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        let channel = Endpoint::from_shared(endpoint)?.connect().await?;
        Ok(Self::new(channel))
    }

    pub fn max_decoding_message_size(mut self, bytes: usize) -> Self {
        self.decoding = bytes;
        self
    }

    pub fn max_encoding_message_size(mut self, bytes: usize) -> Self {
        self.encoding = bytes;
        self
    }

    pub fn codec<D, E>(&self) -> Result<harmos::grpc::Codec<harmos::grpc::Transcoded, D, harmos::operations::Response<E>>, tonic::Status>
    where
        harmos::grpc::Transcoded: harmos::grpc::Wire<D> + harmos::grpc::Wire<E>,
        D: harmos::MemorySize + Send + 'static,
        E: Send + Sync + 'static,
    {
        use std::num::NonZeroUsize;
        harmos::grpc::Codec::new(
            harmos::grpc::DecodeSettings {
                message_bytes: NonZeroUsize::new(self.decoding).unwrap(),
                retained_bytes: NonZeroUsize::new(self.decoding.saturating_mul(4)).unwrap(),
            },
            harmos::grpc::EncodeSettings {
                message_bytes: NonZeroUsize::new(self.encoding).unwrap(),
                buffer_bytes: NonZeroUsize::new(self.encoding.min(8192)).unwrap(),
            },
        )
    }

    pub async fn grpc(&self) -> Result<tonic::client::Grpc<Channel>, tonic::Status> {
        let mut grpc = tonic::client::Grpc::new(self.channel.clone())
            .max_decoding_message_size(self.decoding)
            .max_encoding_message_size(self.encoding);
        grpc.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
        Ok(grpc)
    }

    pub async fn unary<O>(&self, request: impl tonic::IntoRequest<O>) -> Result<tonic::Response<O::Response>, tonic::Status>
    where
        O: contracts::Endpoint,
        O: harmos::domain::Portable + Sync,
        O::Response: harmos::domain::Portable + harmos::MemorySize,
    {
        let request = request.into_request().map(harmos::operations::Response::new);
        let codec = self.codec::<O::Response, O>()?;
        let response = self.grpc().await?.unary(request, O::PATH.parse().unwrap(), codec).await?;
        let (metadata, value, extensions) = response.into_parts();
        let (value, _claims) = value.into_request().map_err(|error| tonic::Status::data_loss(error.to_string()))?;
        Ok(tonic::Response::from_parts(metadata, value, extensions))
    }

    // Exercise Both Malformed Wire Types And Well-Formed Unknown Variants.
    pub async fn invalid_enum<O, E>(
        &self, request: Request<O>, field: &str, unknown_variant: bool,
    ) -> Result<tonic::Response<O::Response>, tonic::Status>
    where
        O: contracts::Endpoint,
        O: harmos::domain::Portable + harmos::schema::Described + Sync,
        O::Response: harmos::domain::Portable + harmos::MemorySize,
        E: harmos::schema::Described,
    {
        use prost::encoding::{DecodeContext, WireType, decode_key, encode_key, encode_varint, skip_field};
        let tag = <O as harmos::schema::Described>::FIELDS
            .iter()
            .find(|entry| entry.name == field)
            .expect("canonical request declares the enum field")
            .tag;
        let request = request.map(|message| {
            let original = message.wire_bytes();
            let mut remaining = original.as_slice();
            let mut bytes = Vec::new();
            // Replace The Entire Enum Field So No Valid Variant Masks The Refusal.
            while !remaining.is_empty() {
                let before = remaining;
                let (current, wire) = decode_key(&mut remaining).unwrap();
                skip_field(wire, current, &mut remaining, DecodeContext::default()).unwrap();
                if current != tag {
                    bytes.extend_from_slice(&before[..before.len() - remaining.len()]);
                }
            }
            if unknown_variant {
                let unknown = (1..=536_870_911)
                    .find(|candidate| !(19_000..=19_999).contains(candidate) && !E::FIELDS.iter().any(|entry| entry.tag == *candidate))
                    .expect("an undeclared enum tag exists");
                let mut variant = Vec::new();
                encode_key(unknown, WireType::LengthDelimited, &mut variant);
                encode_varint(0, &mut variant);
                encode_key(tag, WireType::LengthDelimited, &mut bytes);
                encode_varint(variant.len() as u64, &mut bytes);
                bytes.extend(variant);
            } else {
                encode_key(tag, WireType::Varint, &mut bytes);
                encode_varint(99, &mut bytes);
            }
            bytes
        });
        let codec = WireBytes { codec: self.codec::<O::Response, ()>()? };
        let response = self.grpc().await?.unary(request, O::PATH.parse().unwrap(), codec).await?;
        let (metadata, value, extensions) = response.into_parts();
        let (value, _claims) = value.into_request().map_err(|error| tonic::Status::data_loss(error.to_string()))?;
        Ok(tonic::Response::from_parts(metadata, value, extensions))
    }

    pub async fn download<O>(
        &self, request: impl tonic::IntoRequest<O>,
    ) -> Result<tonic::Response<WireStream<<O::Output as harmos::operations::Direction>::Item>>, tonic::Status>
    where
        O: contracts::Endpoint,
        O: harmos::domain::Portable + Sync,
        O::Response: harmos::domain::Portable + harmos::MemorySize,
        <O::Output as harmos::operations::Direction>::Item: harmos::domain::Portable + harmos::MemorySize,
    {
        use futures_util::StreamExt;
        use harmos::grpc::OutputEnvelope;
        let request = request.into_request().map(harmos::operations::Response::new);
        let codec = self.codec::<OutputEnvelope<<O::Output as harmos::operations::Direction>::Item, O::Response>, O>()?;
        let response = self.grpc().await?.server_streaming(request, O::PATH.parse().unwrap(), codec).await?;
        let (metadata, incoming, extensions) = response.into_parts();
        let frames = futures_util::stream::unfold(Some(incoming), |state| async move {
            let mut incoming = state?;
            match incoming.next().await {
                Some(Err(error)) => Some((Err(error), None)),
                None => Some((Err(tonic::Status::data_loss("operation stream has no completion")), None)),
                Some(Ok(frame)) => match OutputEnvelope::project(frame) {
                    Err(error) => Some((Err(tonic::Status::data_loss(error.to_string())), None)),
                    Ok(OutputEnvelope::Item(item)) => {
                        let value =
                            item.into_request().map(|(value, _claims)| value).map_err(|error| tonic::Status::data_loss(error.to_string()));
                        Some((value, Some(incoming)))
                    }
                    Ok(OutputEnvelope::Response(_)) => match incoming.next().await {
                        Some(Err(error)) => Some((Err(error), None)),
                        Some(Ok(_)) => Some((Err(tonic::Status::data_loss("operation continued after completion")), None)),
                        None => None,
                    },
                },
            }
        });
        Ok(tonic::Response::from_parts(metadata, WireStream { frames: Box::pin(frames) }, extensions))
    }

    pub async fn upload<O>(
        &self, request: impl tonic::IntoStreamingRequest<Message = <O::Input as harmos::operations::Direction>::Item>,
    ) -> Result<tonic::Response<O::Response>, tonic::Status>
    where
        O: contracts::Endpoint,
        O: harmos::domain::Portable + Sync + Default,
        O::Response: harmos::domain::Portable + harmos::MemorySize,
        <O::Input as harmos::operations::Direction>::Item: harmos::domain::Portable + Sync,
    {
        use futures_util::StreamExt;
        use harmos::grpc::InputEnvelope;
        use harmos::operations::Response;
        let request = request.into_streaming_request().map(|items| {
            let opening = futures_util::stream::once(async { Response::new(InputEnvelope::Request(O::default())) });
            opening.chain(items.map(|item| Response::new(InputEnvelope::Item(item))))
        });
        let codec = self.codec::<O::Response, InputEnvelope<O, <O::Input as harmos::operations::Direction>::Item>>()?;
        let response = self.grpc().await?.client_streaming(request, O::PATH.parse().unwrap(), codec).await?;
        let (metadata, value, extensions) = response.into_parts();
        let (value, _claims) = value.into_request().map_err(|error| tonic::Status::data_loss(error.to_string()))?;
        Ok(tonic::Response::from_parts(metadata, value, extensions))
    }
}

impl<C: tonic::codec::Codec> tonic::codec::Codec for WireBytes<C> {
    type Encode = Vec<u8>;
    type Decode = C::Decode;
    type Encoder = BytesEncoder;
    type Decoder = C::Decoder;

    fn encoder(&mut self) -> Self::Encoder {
        BytesEncoder
    }
    fn decoder(&mut self) -> Self::Decoder {
        self.codec.decoder()
    }
}

impl tonic::codec::Encoder for BytesEncoder {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn encode(&mut self, item: Self::Item, buffer: &mut tonic::codec::EncodeBuf<'_>) -> Result<(), Self::Error> {
        use prost::bytes::BufMut;
        buffer.put_slice(&item);
        Ok(())
    }
}

impl<T> std::fmt::Debug for WireStream<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("WireStream").finish_non_exhaustive()
    }
}

impl<T> WireStream<T> {
    pub async fn message(&mut self) -> Result<Option<T>, tonic::Status> {
        use futures_util::StreamExt;
        self.frames.next().await.transpose()
    }
}

impl Default for Gate {
    fn default() -> Self {
        Self::reporting(passed_outcome)
    }
}

impl Gate {
    pub fn reporting(outcome: fn(&CycleView) -> ProviderCycleOutcome) -> Self {
        Self { calls: AtomicUsize::new(0), started: Notify::new(), release: Notify::new(), returned: AtomicBool::new(false), outcome }
    }
}

impl ExecutionFixture for Gate {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a CycleView, _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.started.notify_one();
            let released = self.release.notified();
            tokio::pin!(released);
            tokio::select! {
                _ = &mut released => {},
                () = context.cancelled() => {
                    return Err(dokime_runtime::transport::ProviderExecutionFailure::new(
                        "cancel", "fixture acknowledged cancellation",
                    ));
                }
            }
            let outcome = (self.outcome)(cycle);
            self.returned.store(true, Ordering::Release);
            Ok(outcome)
        })
    }
}

/* === Public Functions === */
pub async fn runtime(root: &std::path::Path) -> Runtime {
    let config = super::budgets::runtime_config(root, Metadata::new(Identifier::new("grpc.fixture").unwrap(), "gRPC fixture", None));
    Runtime::start(config).await.unwrap()
}

pub fn keyed() -> Config {
    Config::remote("127.0.0.1:0".parse().unwrap(), None, Some(TOKEN)).unwrap()
}

pub fn auth<T>(message: T, token: &str) -> Request<T> {
    let mut request = Request::new(message);
    request.metadata_mut().insert("authorization", format!("Bearer {token}").parse().unwrap());
    request
}

pub fn authenticated<T>(message: T) -> Request<T> {
    auth(message, TOKEN)
}

pub fn passed_outcome(cycle: &CycleView) -> ProviderCycleOutcome {
    ProviderCycleOutcome {
        status: "passed".into(),
        dependency_imports: vec![],
        provider: ProviderStatusOutcome {
            status: "active".into(),
            domain_registration_count: 1,
            test_suite_count: 1,
            test_suite_identifiers: vec!["fixture.suite".into()],
            error: None,
        },
        failures: vec![],
        executions: cycle
            .test_executions
            .iter()
            .filter(|execution| matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running))
            .map(|execution| {
                let case = cycle.cases().find(|case| case.test_case_id == execution.test_case_id).unwrap();
                ProviderExecutionOutcome {
                    case_id: case.test_case_id.to_string(),
                    test_execution_id: execution.test_execution_id.to_string(),
                    template_identifier: case.metadata.identifier.to_string(),
                    label: case.metadata.label.clone(),
                    module: "grpc.fixture".into(),
                    status: "passed".into(),
                    phases: vec![],
                    records: vec![],
                    telemetry: vec![],
                    criteria: vec![],
                    failures: vec![],
                }
            })
            .collect(),
    }
}

pub async fn connect(server: &Server) -> WireClient {
    WireClient::connect(server.local_endpoint()).await.unwrap()
}

pub fn selection_plan(owner: &Runtime) -> model::planning::wire::SelectionPlan {
    let snapshot = owner.handle().watch_state().latest();
    let suite = snapshot.state().suites.get("fixture.suite").unwrap();
    model::planning::wire::SelectionPlan {
        selections: suite
            .suite
            .test_case_templates
            .iter()
            .map(|template| model::planning::wire::RegisteredSelection {
                suite_id: suite.suite_id.clone(),
                template_id: template.test_case_template_id.to_string(),
                ..Default::default()
            })
            .collect(),
        max_cases: 0,
        case_ids: vec![],
        cycle_id: String::new(),
    }
}

pub async fn fixture(root: &std::path::Path, gate: Arc<Gate>) -> Runtime {
    fixture_with_provider(root, gate).await
}

pub async fn fixture_with_provider(root: &std::path::Path, provider: Arc<dyn ExecutionFixture>) -> Runtime {
    let owner = Runtime::with_execution_fixture(
        super::budgets::runtime_config(root, super::metadata("grpc.fixture", "Fixture")),
        "fixture",
        provider,
    )
    .await
    .unwrap();
    super::register_fixture_execution(&owner.handle(), super::fixture_suite(&["passed"])).await;
    owner
}

pub fn create() -> contracts::execution::CreateCycle {
    contracts::execution::CreateCycle {
        metadata: Some(model::identity::wire::Metadata {
            identifier: "grpc.cycle".into(),
            label: "gRPC cycle".into(),
            description: None,
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub fn accepted(reply: model::execution::wire::RunReply) -> model::execution::wire::Admission {
    match reply.outcome.unwrap() {
        model::execution::wire::RunReplyOutcome::Accepted(admission) => admission,
        model::execution::wire::RunReplyOutcome::Rejected(rejection) => {
            panic!("run rejected: {rejection:?}")
        }
    }
}

/* === Private Functions === */

/* === Tests === */
