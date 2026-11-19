/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Features Shared by Host Stories

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::{Read, Write};
use std::net::TcpStream;
use std::{collections::HashMap, env, fmt, result};

// Third Party Imports
use harmos::plugin::{Context, Contribution, Dependency, Operation, Plugin};
use harmos::{Error, Message, OperationRequest, Result, Visibility};
use harmos_protocol::{model::FileMetadata, schema};
use prost::{
    DecodeError,
    bytes::{Buf, BufMut},
    encoding::{DecodeContext, WireType},
};
use prost_reflect::ReflectMessage;
use prost_types::FileDescriptorSet;
use sha2::{Digest, Sha256};

// Local Imports
use dokime_protocol::host::content::{ArtifactsCreateRequest, ArtifactsCreateResponse};
use dokime_protocol::host::evidence::{TraceabilityGetRequest, TraceabilityGetResponse, TraceabilitySetRequest, TraceabilitySetResponse};
use dokime_protocol::host::executions::{ProgressGetRequest, ProgressGetResponse, ProgressSetRequest, ProgressSetResponse};
use dokime_protocol::host::files::{
    UploadsAppendRequest, UploadsAppendResponse, UploadsCreateRequest, UploadsCreateResponse, UploadsDeleteRequest, UploadsDeleteResponse,
    UploadsPublishRequest, UploadsPublishResponse, UploadsResumeRequest, UploadsResumeResponse,
};
use dokime_protocol::host::{PromptsCreateRequest, PromptsCreateResponse};
use dokime_protocol::model::{
    ExecutionProgressValue, Metadata, Prompt, PromptAction, Step, Suite, Suites, TestCase, TestCaseTemplate, TraceabilityEntry, Value,
    value::Kind,
};
use dokime_protocol::plugin::{StepsExecuteRequest, StepsExecuteResponse, TestCasesMaterializeRequest, TestCasesMaterializeResponse};

/* === Constants === */

/* === Macros === */

macro_rules! request {
    ($request:ty, $response:ty, $id:literal) => {
        impl OperationRequest for Wire<$request> {
            type Response = Wire<$response>;
            const ID: &'static str = $id;
        }
    };
}
request!(ProgressGetRequest, ProgressGetResponse, "dokime.host.v1.executions.Progress/Get");
request!(ProgressSetRequest, ProgressSetResponse, "dokime.host.v1.executions.Progress/Set");
request!(TraceabilityGetRequest, TraceabilityGetResponse, "dokime.host.v1.evidence.Traceability/Get");
request!(TraceabilitySetRequest, TraceabilitySetResponse, "dokime.host.v1.evidence.Traceability/Set");
request!(UploadsCreateRequest, UploadsCreateResponse, "dokime.host.v1.files.Uploads/Create");
request!(UploadsAppendRequest, UploadsAppendResponse, "dokime.host.v1.files.Uploads/Append");
request!(UploadsResumeRequest, UploadsResumeResponse, "dokime.host.v1.files.Uploads/Resume");
request!(UploadsPublishRequest, UploadsPublishResponse, "dokime.host.v1.files.Uploads/Publish");
request!(UploadsDeleteRequest, UploadsDeleteResponse, "dokime.host.v1.files.Uploads/Delete");
request!(ArtifactsCreateRequest, ArtifactsCreateResponse, "dokime.host.v1.content.Artifacts/Create");
request!(PromptsCreateRequest, PromptsCreateResponse, "dokime.host.v1.Prompts/Create");

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default, PartialEq)]
struct Wire<T>(T);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<T: fmt::Debug> fmt::Debug for Wire<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
impl<T: prost::Message> prost::Message for Wire<T> {
    fn encode_raw(&self, buffer: &mut impl BufMut) {
        self.0.encode_raw(buffer);
    }
    fn merge_field(&mut self, tag: u32, wire: WireType, buffer: &mut impl Buf, context: DecodeContext) -> result::Result<(), DecodeError> {
        self.0.merge_field(tag, wire, buffer, context)
    }
    fn encoded_len(&self) -> usize {
        self.0.encoded_len()
    }
    fn clear(&mut self) {
        self.0.clear();
    }
}
impl<T: ReflectMessage + Default + 'static> Message for Wire<T> {
    fn name() -> String {
        T::default().descriptor().full_name().into()
    }
    fn types() -> FileDescriptorSet {
        let descriptor = T::default().descriptor();
        let types = FileDescriptorSet {
            file: descriptor.parent_pool().files().map(|file| file.file_descriptor_proto().clone()).collect(),
        };
        schema::operation_types(&types, &[descriptor.full_name()]).expect("Protocol Fixture Message Descriptor")
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    // Hold Managed Startup Until Its Caller Cancels the Installation.
    if let Ok(address) = env::var("DOKIME_PENDING_INSTALL") {
        let mut connection = TcpStream::connect(address).map_err(|error| Error::Invalid(format!("Announce Pending Install: {error}")))?;
        connection.write_all(&[1]).map_err(|error| Error::Invalid(format!("Announce Pending Install: {error}")))?;
        let mut release = Vec::new();
        connection.read_to_end(&mut release).map_err(|error| Error::Invalid(format!("Wait for Installation Release: {error}")))?;
        return Ok(());
    }

    // Declare Three Protocol Fixtures Without an Authoring SDK.
    let mut plugin = Plugin::new("content_fixture", "1.0.0").register(|registrar| {
        registrar.register(Dependency::<Wire<TraceabilityGetRequest>>::new())?;
        registrar.register(Dependency::<Wire<TraceabilitySetRequest>>::new())?;
        registrar.register(Dependency::<Wire<UploadsCreateRequest>>::new())?;
        registrar.register(Dependency::<Wire<UploadsAppendRequest>>::new())?;
        registrar.register(Dependency::<Wire<UploadsResumeRequest>>::new())?;
        registrar.register(Dependency::<Wire<UploadsPublishRequest>>::new())?;
        registrar.register(Dependency::<Wire<UploadsDeleteRequest>>::new())?;
        registrar.register(Dependency::<Wire<ArtifactsCreateRequest>>::new())?;
        registrar.register(Dependency::<Wire<ProgressGetRequest>>::new())?;
        registrar.register(Dependency::<Wire<ProgressSetRequest>>::new())?;
        registrar.register(Dependency::<Wire<PromptsCreateRequest>>::new())?;
        Ok(())
    })?;
    let mut templates = Vec::new();
    for (name, label, step_label) in [
        ("capture", "Capture Content", "Capture"),
        ("progress", "Observe Progress", "Progress"),
        ("traceability", "Observe Traceability", "Traceability"),
    ] {
        let identifier = format!("content.{name}");
        let operation = format!("{identifier}.{name}");
        let step = Step {
            metadata: Some(Metadata {
                identifier: operation.clone(),
                label: step_label.into(),
                ..Default::default()
            }),
            operation: operation.clone(),
            ..Default::default()
        };
        let materialize = format!("{identifier}.materialize");
        let metadata = Metadata {
            identifier: identifier.clone(),
            label: label.into(),
            ..Default::default()
        };
        let case = TestCase {
            steps: vec![step.clone()],
            ..Default::default()
        };
        templates.push(TestCaseTemplate {
            metadata: Some(metadata),
            materialize: materialize.clone(),
            steps: vec![step],
            ..Default::default()
        });
        plugin = plugin.register(|registrar| {
            registrar.register(
                Operation::new(materialize, async move |ctx: &Context, Wire(request): Wire<TestCasesMaterializeRequest>| {
                    if ctx.scope().is_ok() || request.template != identifier {
                        return Err(Error::Invalid("Materialize Fixture: Supply Its Template Without an Execution Scope".into()));
                    }
                    let mut case = case.clone();
                    case.parameters = request.parameters;
                    Ok(Wire(TestCasesMaterializeResponse { test_case: Some(case) }))
                })
                .visibility(Visibility::Protected),
            )
        })?;
        plugin = plugin.register(|registrar| {
            registrar.register(
                Operation::new(operation.clone(), async move |ctx: &Context, Wire(request): Wire<StepsExecuteRequest>| {
                    if ctx.scope()? != request.execution || request.step != operation {
                        return Err(Error::Invalid("Execute Fixture: Supply Its Step and Bound Execution Scope".into()));
                    }
                    match name {
                        "capture" => capture(ctx).await?,
                        "progress" => revisioned_progress(ctx).await?,
                        "traceability" => revisioned_traceability(ctx).await?,
                        _ => unreachable!(),
                    }
                    Ok(Wire(StepsExecuteResponse {}))
                })
                .visibility(Visibility::Protected),
            )
        })?;
    }
    plugin = plugin.register(|registrar| {
        registrar.register(Contribution(Wire(Suites {
            suites: vec![Suite {
                metadata: Some(Metadata {
                    identifier: "content".into(),
                    label: "Content".into(),
                    ..Default::default()
                }),
                templates,
                ..Default::default()
            }],
        })))
    })?;
    let plugin = if env::args().any(|argument| argument == "--remote") { plugin.remote() } else { plugin };
    plugin.serve()
}

async fn revisioned_progress(ctx: &Context) -> Result<()> {
    // Establish the Whole Value and Replay Its Original Receipt
    let Wire(initial) = ctx.call(Wire(ProgressGetRequest::default())).await?;
    let first = ExecutionProgressValue {
        completed: 1,
        total: 2,
        label: "First Value".into(),
    };
    let request = ProgressSetRequest {
        request_id: "first".into(),
        value: Some(first.clone()),
        expected_revision: initial.revision.clone(),
        ..Default::default()
    };
    let Wire(saved) = ctx.call(Wire(request)).await?;
    let request = ProgressSetRequest {
        request_id: "first".into(),
        value: Some(first.clone()),
        expected_revision: initial.revision.clone(),
        ..Default::default()
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0)? != saved {
        return Err(Error::Invalid("Set Progress: Replay Lost Its Original Receipt".into()));
    }
    let mut changed = first.clone();
    changed.label = "Changed Value".into();
    let request = ProgressSetRequest {
        request_id: "first".into(),
        value: Some(changed),
        expected_revision: initial.revision.clone(),
        ..Default::default()
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0).is_ok() {
        return Err(Error::Invalid("Set Progress: Changed Request Payload Was Accepted".into()));
    }
    let request = ProgressSetRequest {
        request_id: "stale".into(),
        value: Some(first.clone()),
        expected_revision: initial.revision.clone(),
        ..Default::default()
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0).is_ok() {
        return Err(Error::Invalid("Set Progress: Stale Revision Was Accepted".into()));
    }

    // Await the Operator Before Replacing the Value
    let definition = Prompt {
        metadata: Some(Metadata {
            identifier: "content.continue".into(),
            label: "Continue".into(),
            ..Default::default()
        }),
        message: "Continue".into(),
        actions: vec![PromptAction {
            identifier: "continue".into(),
            label: "Continue".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let request = PromptsCreateRequest {
        definition: Some(definition),
        prompt: "continue".into(),
        prompt_type: "content.continue".into(),
        ..Default::default()
    };
    ctx.call(Wire(request)).await?;
    let Wire(current) = ctx.call(Wire(ProgressGetRequest::default())).await?;
    let second = ExecutionProgressValue {
        completed: 2,
        total: 2,
        label: String::new(),
    };
    let request = ProgressSetRequest {
        request_id: "second".into(),
        value: Some(second),
        expected_revision: current.revision,
        ..Default::default()
    };
    ctx.call(Wire(request)).await?;
    let request = ProgressSetRequest {
        request_id: "first".into(),
        value: Some(first),
        expected_revision: initial.revision,
        ..Default::default()
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0)? != saved {
        return Err(Error::Invalid("Set Progress: Later Writes Changed an Earlier Receipt".into()));
    }
    Ok(())
}

async fn revisioned_traceability(ctx: &Context) -> Result<()> {
    // Establish Two Independently Revisioned Entries
    let request = TraceabilityGetRequest {
        entry: Some("serial".into()),

        ..Default::default()
    };
    let Wire(initial) = ctx.call(Wire(request)).await?;
    let first = TraceabilityEntry {
        metadata: Some(Metadata {
            identifier: "serial".into(),
            label: "Serial Number".into(),
            attributes: HashMap::from([
                ("verified".into(), Value { kind: Some(Kind::Boolean(true)) }),
                ("channel".into(), Value { kind: Some(Kind::UnsignedInteger(2)) }),
            ]),
            ..Default::default()
        }),
        value: Some(Value { kind: Some(Kind::Text("first".into())) }),
        source: "reader".into(),
        unit: "unit".into(),
    };
    let request = TraceabilitySetRequest {
        request_id: "first".into(),
        value: Some(first.clone()),
        expected_revision: initial.revision.clone(),
        entry: "serial".into(),
        ..Default::default()
    };
    let Wire(saved) = ctx.call(Wire(request)).await?;
    let request = TraceabilitySetRequest {
        request_id: "first".into(),
        value: Some(first.clone()),
        expected_revision: initial.revision.clone(),
        entry: "serial".into(),
        ..Default::default()
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0)? != saved {
        return Err(Error::Invalid("Set Traceability: Replay Lost Its Original Receipt".into()));
    }
    let mut changed = first.clone();
    changed.source.clear();
    let request = TraceabilitySetRequest {
        request_id: "first".into(),
        value: Some(changed),
        expected_revision: initial.revision.clone(),
        entry: "serial".into(),
        ..Default::default()
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0).is_ok() {
        return Err(Error::Invalid("Set Traceability: Changed Request Payload Was Accepted".into()));
    }
    let request = TraceabilitySetRequest {
        request_id: "stale".into(),
        value: Some(first.clone()),
        expected_revision: initial.revision.clone(),
        entry: "serial".into(),
        ..Default::default()
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0).is_ok() {
        return Err(Error::Invalid("Set Traceability: Stale Revision Was Accepted".into()));
    }
    let request = TraceabilityGetRequest {
        entry: Some("batch".into()),

        ..Default::default()
    };
    let Wire(other) = ctx.call(Wire(request)).await?;
    let batch = TraceabilityEntry {
        metadata: Some(Metadata {
            identifier: "batch".into(),
            ..Default::default()
        }),
        value: Some(Value {
            kind: Some(Kind::Text("batch-one".into())),
        }),
        ..Default::default()
    };
    let request = TraceabilitySetRequest {
        request_id: "batch".into(),
        value: Some(batch),
        expected_revision: other.revision,
        entry: "batch".into(),
        ..Default::default()
    };
    ctx.call(Wire(request)).await?;
    let request = TraceabilityGetRequest {
        entry: Some("serial".into()),

        ..Default::default()
    };
    if ctx.call(Wire(request)).await?.0.revision != saved.revision {
        return Err(Error::Invalid("Get Traceability: Another Entry Changed This Revision".into()));
    }

    // Await the Operator Before Replacing the Whole Entry
    let definition = Prompt {
        metadata: Some(Metadata {
            identifier: "content.continue".into(),
            label: "Continue".into(),
            ..Default::default()
        }),
        message: "Continue".into(),
        actions: vec![PromptAction {
            identifier: "continue".into(),
            label: "Continue".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let request = PromptsCreateRequest {
        definition: Some(definition),
        prompt: "continue".into(),
        prompt_type: "content.continue".into(),
        ..Default::default()
    };
    ctx.call(Wire(request)).await?;
    let second = TraceabilityEntry {
        metadata: Some(Metadata {
            identifier: "serial".into(),
            ..Default::default()
        }),
        value: Some(Value { kind: Some(Kind::Text("second".into())) }),
        ..Default::default()
    };
    let request = TraceabilitySetRequest {
        request_id: "second".into(),
        value: Some(second),
        expected_revision: saved.revision.clone(),
        entry: "serial".into(),
        ..Default::default()
    };
    ctx.call(Wire(request)).await?;
    let request = TraceabilitySetRequest {
        request_id: "first".into(),
        value: Some(first),
        expected_revision: initial.revision,
        entry: "serial".into(),
        ..Default::default()
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0)? != saved {
        return Err(Error::Invalid("Set Traceability: Later Writes Changed an Earlier Receipt".into()));
    }
    Ok(())
}

async fn capture(ctx: &Context) -> Result<()> {
    // Open an Upload and Replay Its Receipt
    let metadata = FileMetadata {
        scope: Some(format!("execution/{}", ctx.scope()?)),
        media_type: "text/plain".into(),
        ..Default::default()
    };
    let request = UploadsCreateRequest {
        request_id: "capture/create".into(),
        metadata: Some(metadata.clone()),
    };
    let Wire(created) = ctx.call(Wire(request)).await?;
    let request = UploadsCreateRequest {
        request_id: "capture/create".into(),
        metadata: Some(metadata),
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0)? != created {
        return Err(Error::Invalid("Create Upload: Replay Changed Its Session".into()));
    }
    let upload = created.upload.ok_or_else(|| Error::Invalid("Capture: Missing Upload Writer".into()))?;
    let bytes = b"replacement\n";
    let request = UploadsAppendRequest {
        request_id: "capture/prefix".into(),
        upload: Some(upload.clone()),
        offset: 0,
        data: bytes[..11].to_vec(),
    };
    let Wire(saved) = ctx.call(Wire(request)).await?;
    let request = UploadsAppendRequest {
        request_id: "capture/prefix".into(),
        upload: Some(upload.clone()),
        offset: 0,
        data: bytes[..11].to_vec(),
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0)? != saved {
        return Err(Error::Invalid("Append Upload: Replay Changed Its Receipt".into()));
    }

    // Refuse Gaps and Fence the Previous Writer
    let request = UploadsAppendRequest {
        request_id: "capture/gap".into(),
        upload: Some(upload.clone()),
        offset: 30,
        data: b"gap".to_vec(),
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0).is_ok() {
        return Err(Error::Invalid("Capture: Offset Gap Was Accepted".into()));
    }
    let request = UploadsResumeRequest {
        request_id: "capture/resume".into(),
        upload: Some(upload.clone()),
    };
    let Wire(resumed) = ctx.call(Wire(request)).await?;
    let request = UploadsAppendRequest {
        request_id: "capture/stale".into(),
        upload: Some(upload),
        offset: 11,
        data: b"\n".to_vec(),
    };
    if ctx.call(Wire(request)).await.map(|reply| reply.0).is_ok() {
        return Err(Error::Invalid("Capture: Stale Writer Was Accepted".into()));
    }
    let upload = resumed.upload.ok_or_else(|| Error::Invalid("Capture: Missing Resumed Writer".into()))?;
    let request = UploadsAppendRequest {
        request_id: "capture/suffix".into(),
        upload: Some(upload.clone()),
        offset: 11,
        data: b"\n".to_vec(),
    };
    ctx.call(Wire(request)).await?;

    // Publish Verified Content and Attach the Artifact
    let request = UploadsPublishRequest {
        request_id: "capture/publish".into(),
        upload: Some(upload),
        length: bytes.len() as u64,
        sha256: Sha256::digest(bytes).to_vec(),
    };
    let Wire(published) = ctx.call(Wire(request)).await?;
    let file = published.upload.ok_or_else(|| Error::Invalid("Capture: Missing Published Content".into()))?;
    let request = ArtifactsCreateRequest {
        request_id: "capture/attach".into(),
        metadata: Some(Metadata {
            identifier: "producer.txt".into(),
            label: "producer.txt".into(),
            ..Default::default()
        }),
        file: Some(file),
        filename: "producer.txt".into(),
        content_type: "text/plain".into(),
        ..Default::default()
    };
    ctx.call(Wire(request)).await?;

    // Abandon a Separate Upload Without Publishing It
    let metadata = FileMetadata {
        scope: Some(format!("execution/{}", ctx.scope()?)),
        ..Default::default()
    };
    let request = UploadsCreateRequest {
        request_id: "abandoned/create".into(),
        metadata: Some(metadata),
    };
    let Wire(abandoned) = ctx.call(Wire(request)).await?;
    let upload = abandoned.upload.ok_or_else(|| Error::Invalid("Capture: Missing Abandoned Writer".into()))?;
    let request = UploadsDeleteRequest {
        request_id: "abandoned/delete".into(),
        upload: Some(upload),
    };
    ctx.call(Wire(request)).await?;
    Ok(())
}

/* === Tests === */
