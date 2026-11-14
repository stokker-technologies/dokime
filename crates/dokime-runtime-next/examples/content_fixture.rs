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

// Third Party Imports

// Local Imports
use dokime::{prelude::*, v1};
use dokime_protocol::context::{
    content::ArtifactsCreateRequest,
    files::{UploadsAppendRequest, UploadsCreateRequest, UploadsDeleteRequest, UploadsPublishRequest, UploadsResumeRequest},
};
use harmos_protocol::storage::FileMetadata;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::TcpStream;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    // Hold Managed Startup Until Its Caller Cancels the Installation.
    if let Ok(address) = std::env::var("DOKIME_PENDING_INSTALL") {
        use std::io::{Read, Write};
        let mut connection = TcpStream::connect(address).map_err(|error| Error::Invalid(format!("Announce Pending Install: {error}")))?;
        connection.write_all(&[1]).map_err(|error| Error::Invalid(format!("Announce Pending Install: {error}")))?;
        let mut release = Vec::new();
        connection.read_to_end(&mut release).map_err(|error| Error::Invalid(format!("Wait for Installation Release: {error}")))?;
        return Ok(());
    }

    let template = Template::new("content.capture", "Capture Content").step(Step::new("Capture", capture));
    let progress = Template::new("content.progress", "Observe Progress").step(Step::new("Progress", revisioned_progress));
    let traceability =
        Template::new("content.traceability", "Observe Traceability").step(Step::new("Traceability", revisioned_traceability));
    let plugin = Plugin::new("content_fixture", "1.0.0")
        .suite(Suite::new("content", "Content").template(template).template(progress).template(traceability))
        .requires_host::<dokime::protocol::plugin::host::v1::evidence::TraceabilityGetRequest>()
        .requires_host::<dokime::protocol::plugin::host::v1::evidence::TraceabilitySetRequest>()
        .requires_host::<UploadsCreateRequest>()
        .requires_host::<UploadsAppendRequest>()
        .requires_host::<UploadsResumeRequest>()
        .requires_host::<UploadsPublishRequest>()
        .requires_host::<UploadsDeleteRequest>()
        .requires_host::<ArtifactsCreateRequest>()
        .requires_host::<dokime::protocol::plugin::host::v1::executions::ProgressGetRequest>()
        .requires_host::<dokime::protocol::plugin::host::v1::executions::ProgressSetRequest>()
        .requires_host::<dokime::protocol::plugin::host::v1::PromptsCreateRequest>();
    let plugin = if std::env::args().any(|argument| argument == "--remote") { plugin.remote() } else { plugin };
    plugin.serve()
}

async fn revisioned_progress(ctx: &Context) -> Result<()> {
    use dokime::protocol::plugin::host::v1::executions::{ProgressGetRequest, ProgressSetRequest};

    // Establish a Whole Value and Recover Its Original Receipt
    let initial = ctx.host(ProgressGetRequest { name: ctx.execution_id().into() }).await?;
    let first = ProgressSetRequest {
        name: ctx.execution_id().into(),
        value: Some(v1::ExecutionProgressValue {
            completed: 1,
            total: 2,
            label: "First Value".into(),
        }),
        expected_revision: initial.revision,
        request_id: "first".into(),
    };
    let saved = ctx.host(first.clone()).await?;
    if ctx.host(first.clone()).await? != saved {
        return Err(Error::Invalid("Set Progress: Replay Lost Its Original Receipt".into()));
    }
    let mut changed = first.clone();
    changed.value.as_mut().unwrap().label = "Changed Value".into();
    if ctx.host(changed).await.is_ok() {
        return Err(Error::Invalid("Set Progress: Changed Request ID Payload Was Accepted".into()));
    }
    let stale = ProgressSetRequest {
        request_id: "stale".into(),
        ..first.clone()
    };
    if ctx.host(stale).await.is_ok() {
        return Err(Error::Invalid("Set Progress: Stale Revision Was Accepted".into()));
    }

    // Let the Client Retain This Value Before Replacing It
    ctx.prompt(dokime::protocol::plugin::host::v1::PromptsCreateRequest {
        prompt: "continue".into(),
        prompt_type: "content.continue".into(),
        definition: Some(v1::Prompt {
            metadata: Some(v1::Metadata {
                identifier: "content.continue".into(),
                label: "Continue".into(),
                ..Default::default()
            }),
            actions: vec![v1::PromptAction {
                identifier: "continue".into(),
                label: "Continue".into(),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    })
    .await?;

    // Replace Every Writable Field, Including Clearing the Earlier Label
    let current = ctx.host(ProgressGetRequest { name: ctx.execution_id().into() }).await?;
    ctx.host(ProgressSetRequest {
        name: ctx.execution_id().into(),
        value: Some(v1::ExecutionProgressValue {
            completed: 2,
            total: 2,
            label: String::new(),
        }),
        expected_revision: current.revision,
        request_id: "second".into(),
    })
    .await?;
    if ctx.host(first).await? != saved {
        return Err(Error::Invalid("Set Progress: Later Writes Changed an Earlier Receipt".into()));
    }
    Ok(())
}
async fn revisioned_traceability(ctx: &Context) -> Result<()> {
    use dokime::protocol::plugin::host::v1::evidence::{TraceabilityGetRequest, TraceabilitySetRequest};

    // Establish Two Independently Revisioned Whole Entries
    let get = TraceabilityGetRequest {
        name: ctx.execution_id().into(),
        entry: "serial".into(),
    };
    let initial = ctx.host(get.clone()).await?;
    let first = TraceabilitySetRequest {
        name: ctx.execution_id().into(),
        entry: "serial".into(),
        value: Some(v1::TraceabilityEntry {
            metadata: Some(v1::Metadata {
                identifier: "serial".into(),
                label: "Serial Number".into(),
                attributes: HashMap::from([
                    (
                        "verified".into(),
                        v1::Value {
                            kind: Some(v1::value::Kind::Boolean(true)),
                        },
                    ),
                    (
                        "channel".into(),
                        v1::Value {
                            kind: Some(v1::value::Kind::UnsignedInteger(2)),
                        },
                    ),
                ]),
                ..Default::default()
            }),
            value: Some(v1::Value {
                kind: Some(v1::value::Kind::Text("first".into())),
            }),
            source: "reader".into(),
            unit: "unit".into(),
        }),
        expected_revision: initial.revision,
        request_id: "first".into(),
    };
    let saved = ctx.host(first.clone()).await?;
    if ctx.host(first.clone()).await? != saved {
        return Err(Error::Invalid("Set Traceability: Replay Lost Its Original Receipt".into()));
    }
    let mut changed = first.clone();
    changed.value.as_mut().unwrap().source.clear();
    if ctx.host(changed).await.is_ok() {
        return Err(Error::Invalid("Set Traceability: Changed Request Payload Was Accepted".into()));
    }
    if ctx
        .host(TraceabilitySetRequest {
            request_id: "stale".into(),
            ..first.clone()
        })
        .await
        .is_ok()
    {
        return Err(Error::Invalid("Set Traceability: Stale Revision Was Accepted".into()));
    }
    let other = ctx.host(TraceabilityGetRequest { entry: "batch".into(), ..get.clone() }).await?;
    ctx.host(TraceabilitySetRequest {
        entry: "batch".into(),
        value: Some(v1::TraceabilityEntry {
            metadata: Some(v1::Metadata {
                identifier: "batch".into(),
                ..Default::default()
            }),
            value: Some(v1::Value {
                kind: Some(v1::value::Kind::Text("batch-one".into())),
            }),
            ..Default::default()
        }),
        expected_revision: other.revision,
        request_id: "batch".into(),
        ..first.clone()
    })
    .await?;
    if ctx.host(get.clone()).await?.revision != saved.revision {
        return Err(Error::Invalid("Get Traceability: Another Entry Changed This Revision".into()));
    }

    // Let the Client Retain the First Values
    ctx.prompt(dokime::protocol::plugin::host::v1::PromptsCreateRequest {
        prompt: "continue".into(),
        prompt_type: "content.continue".into(),
        definition: Some(v1::Prompt {
            metadata: Some(v1::Metadata {
                identifier: "content.continue".into(),
                label: "Continue".into(),
                ..Default::default()
            }),
            actions: vec![v1::PromptAction {
                identifier: "continue".into(),
                label: "Continue".into(),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    })
    .await?;

    // Replace the Whole Serial Entry and Preserve Both Earlier Receipts
    ctx.host(TraceabilitySetRequest {
        value: Some(v1::TraceabilityEntry {
            metadata: Some(v1::Metadata {
                identifier: "serial".into(),
                ..Default::default()
            }),
            value: Some(v1::Value {
                kind: Some(v1::value::Kind::Text("second".into())),
            }),
            ..Default::default()
        }),
        expected_revision: saved.revision.clone(),
        request_id: "second".into(),
        ..first.clone()
    })
    .await?;
    if ctx.host(first).await? != saved {
        return Err(Error::Invalid("Set Traceability: Later Writes Changed an Earlier Receipt".into()));
    }
    Ok(())
}
async fn capture(ctx: &Context) -> Result<()> {
    // Create and Replay an Upload Opening.
    let opening = UploadsCreateRequest {
        metadata: Some(FileMetadata {
            scope: Some(ctx.execution_id().into()),
            media_type: "text/plain".into(),
            ..Default::default()
        }),
        request_id: "capture/create".into(),
    };
    let created = ctx.host(opening.clone()).await?;
    if ctx.host(opening).await? != created {
        return Err(Error::Invalid("Create Upload: Replay Changed Its Session".into()));
    }
    let session = created
        .production
        .and_then(|production| production.session)
        .ok_or_else(|| Error::Invalid("Capture: Missing Upload Session".into()))?;
    let bytes = b"replacement\n";
    let chunk = UploadsAppendRequest {
        session: Some(session.clone()),
        offset: 0,
        data: bytes[..11].to_vec(),
        request_id: "capture/prefix".into(),
    };
    let saved = ctx.host(chunk.clone()).await?;
    if ctx.host(chunk).await? != saved {
        return Err(Error::Invalid("Append Upload: Replay Changed Its Receipt".into()));
    }
    // Refuse a Gap and a Writer That Lost Ownership.
    let gap = UploadsAppendRequest {
        session: Some(session.clone()),
        offset: 30,
        data: b"gap".to_vec(),
        request_id: "capture/gap".into(),
    };
    if ctx.host(gap).await.is_ok() {
        return Err(Error::Invalid("Capture: Offset Gap Was Accepted".into()));
    }
    let resumed = ctx
        .host(UploadsResumeRequest {
            session: Some(session.clone()),
            request_id: "capture/resume".into(),
        })
        .await?;
    let stale = UploadsAppendRequest {
        session: Some(session),
        offset: 11,
        data: b"\n".to_vec(),
        request_id: "capture/stale".into(),
    };
    if ctx.host(stale).await.is_ok() {
        return Err(Error::Invalid("Capture: Stale Writer Was Accepted".into()));
    }
    let session = resumed
        .production
        .and_then(|production| production.session)
        .ok_or_else(|| Error::Invalid("Capture: Missing Resumed Writer".into()))?;
    ctx.host(UploadsAppendRequest {
        session: Some(session.clone()),
        offset: 11,
        data: b"\n".to_vec(),
        request_id: "capture/suffix".into(),
    })
    .await?;
    // Publish Verified Content, Then Attach Its Metadata.
    let published = ctx
        .host(UploadsPublishRequest {
            session: Some(session),
            length: bytes.len() as u64,
            sha256: Sha256::digest(bytes).to_vec(),
            request_id: "capture/publish".into(),
        })
        .await?;
    ctx.host(ArtifactsCreateRequest {
        execution: ctx.execution_id().into(),
        file: published.file,
        metadata: Some(v1::Metadata {
            identifier: "producer.txt".into(),
            label: "producer.txt".into(),
            ..Default::default()
        }),
        media_type: "text/plain".into(),
        request_id: "capture/attach".into(),
    })
    .await?;
    // Abandon Another Upload Without Publishing Its Prefix.
    let abandoned = ctx
        .host(UploadsCreateRequest {
            metadata: Some(FileMetadata {
                scope: Some(ctx.execution_id().into()),
                ..Default::default()
            }),
            request_id: "abandoned/create".into(),
        })
        .await?;
    ctx.host(UploadsDeleteRequest {
        session: abandoned.production.and_then(|production| production.session),
        request_id: "abandoned/delete".into(),
    })
    .await?;
    Ok(())
}

/* === Tests === */
