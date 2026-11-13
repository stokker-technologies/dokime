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
        let mut connection =
            std::net::TcpStream::connect(address).map_err(|error| Error::Invalid(format!("Announce Pending Install: {error}")))?;
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
        .requires_host::<dokime_protocol::context::content::PublishArtifact>()
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
                attributes: std::collections::HashMap::from([
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
    // Start an Ordered Transfer
    let request = dokime_protocol::context::content::PublishArtifact {
        transfer_id: "capture".into(),
        name: "producer.txt".into(),
        mime: "text/plain".into(),
        data: b"old prefix".to_vec(),
        ..Default::default()
    };
    ctx.host(request.clone()).await?;
    ctx.host(request).await?;

    // Refuse a Sequence Gap
    let request = dokime_protocol::context::content::PublishArtifact {
        transfer_id: "capture".into(),
        name: "producer.txt".into(),
        mime: "text/plain".into(),
        sequence: 3,
        data: b"gap".to_vec(),
        ..Default::default()
    };
    if ctx.host(request).await.is_ok() {
        return Err(Error::Invalid("Capture: Sequence Gap Was Accepted".into()));
    }

    // Replace the Prefix Within the Same Transfer
    let request = dokime_protocol::context::content::PublishArtifact {
        transfer_id: "capture".into(),
        name: "producer.txt".into(),
        mime: "text/plain".into(),
        sequence: 1,
        data: b"replacement".to_vec(),
        replace: true,
        ..Default::default()
    };
    ctx.host(request).await?;

    // Commit an Explicit Terminal Chunk
    let request = dokime_protocol::context::content::PublishArtifact {
        transfer_id: "capture".into(),
        name: "producer.txt".into(),
        mime: "text/plain".into(),
        sequence: 2,
        data: b"\n".to_vec(),
        last: true,
        ..Default::default()
    };
    let receipt = ctx.host(request).await?;
    if !receipt.sealed || receipt.size_bytes != 12 {
        return Err(Error::Invalid("Capture: Wrong Terminal Receipt".into()));
    }

    // Abort Another Transfer Without Publishing Its Prefix
    let request = dokime_protocol::context::content::PublishArtifact {
        transfer_id: "aborted".into(),
        name: "aborted.txt".into(),
        mime: "text/plain".into(),
        data: b"unpublished".to_vec(),
        ..Default::default()
    };
    ctx.host(request).await?;
    let request = dokime_protocol::context::content::PublishArtifact {
        transfer_id: "aborted".into(),
        name: "aborted.txt".into(),
        mime: "text/plain".into(),
        sequence: 1,
        last: true,
        aborted: true,
        ..Default::default()
    };
    let receipt = ctx.host(request).await?;
    if receipt.sealed {
        return Err(Error::Invalid("Capture: Aborted Transfer Was Published".into()));
    }
    Ok(())
}

/* === Tests === */
