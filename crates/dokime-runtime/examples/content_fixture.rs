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
use dokime::plugin::{
    HostContentArtifactsCreateOptions, HostEvidenceTraceabilityGetOptions, HostEvidenceTraceabilitySetOptions,
    HostExecutionsProgressSetOptions, HostFilesUploadsAppendOptions, HostFilesUploadsCreateOptions, HostFilesUploadsDeleteOptions,
    HostFilesUploadsPublishOptions, HostFilesUploadsResumeOptions, HostOperation, HostPromptsCreateOptions,
};
use dokime::prelude::*;
use dokime::value::Kind;
use dokime::{ExecutionProgressValue, FileMetadata, Metadata, TraceabilityEntry, Value};
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
        .requires_host(HostOperation::EvidenceTraceabilityGet)
        .requires_host(HostOperation::EvidenceTraceabilitySet)
        .requires_host(HostOperation::FilesUploadsCreate)
        .requires_host(HostOperation::FilesUploadsAppend)
        .requires_host(HostOperation::FilesUploadsResume)
        .requires_host(HostOperation::FilesUploadsPublish)
        .requires_host(HostOperation::FilesUploadsDelete)
        .requires_host(HostOperation::ContentArtifactsCreate)
        .requires_host(HostOperation::ExecutionsProgressGet)
        .requires_host(HostOperation::ExecutionsProgressSet)
        .requires_host(HostOperation::PromptsCreate);
    let plugin = if std::env::args().any(|argument| argument == "--remote") { plugin.remote() } else { plugin };
    plugin.serve()
}

async fn revisioned_progress(ctx: &Context) -> Result<()> {
    // Establish the Whole Value and Replay Its Original Receipt
    let progress = ctx.host().executions.progress;
    let initial = progress.get().await?;
    let first = ExecutionProgressValue {
        completed: 1,
        total: 2,
        label: "First Value".into(),
    };
    let options = HostExecutionsProgressSetOptions {
        request_id: Some("first".into()),
        ..Default::default()
    };
    let saved = progress.set_with(first.clone(), initial.revision.clone(), options).await?;
    let options = HostExecutionsProgressSetOptions {
        request_id: Some("first".into()),
        ..Default::default()
    };
    if progress.set_with(first.clone(), initial.revision.clone(), options).await? != saved {
        return Err(Error::Invalid("Set Progress: Replay Lost Its Original Receipt".into()));
    }
    let mut changed = first.clone();
    changed.label = "Changed Value".into();
    let options = HostExecutionsProgressSetOptions {
        request_id: Some("first".into()),
        ..Default::default()
    };
    if progress.set_with(changed, initial.revision.clone(), options).await.is_ok() {
        return Err(Error::Invalid("Set Progress: Changed Request Payload Was Accepted".into()));
    }
    let options = HostExecutionsProgressSetOptions {
        request_id: Some("stale".into()),
        ..Default::default()
    };
    if progress.set_with(first.clone(), initial.revision.clone(), options).await.is_ok() {
        return Err(Error::Invalid("Set Progress: Stale Revision Was Accepted".into()));
    }

    // Await the Operator Before Replacing the Value
    let definition = dokime::plugin::Prompt::new("content.continue", "Continue", "Continue").action("continue", "Continue");
    let options = HostPromptsCreateOptions {
        definition: Some(definition.into_message()),
        ..Default::default()
    };
    ctx.host().prompts.create_with("continue".into(), "content.continue".into(), HashMap::new(), options).await?;
    let current = progress.get().await?;
    let second = ExecutionProgressValue {
        completed: 2,
        total: 2,
        label: String::new(),
    };
    let options = HostExecutionsProgressSetOptions {
        request_id: Some("second".into()),
        ..Default::default()
    };
    progress.set_with(second, current.revision, options).await?;
    let options = HostExecutionsProgressSetOptions {
        request_id: Some("first".into()),
        ..Default::default()
    };
    if progress.set_with(first, initial.revision, options).await? != saved {
        return Err(Error::Invalid("Set Progress: Later Writes Changed an Earlier Receipt".into()));
    }
    Ok(())
}

async fn revisioned_traceability(ctx: &Context) -> Result<()> {
    // Establish Two Independently Revisioned Entries
    let traceability = ctx.host().evidence.traceability;
    let options = HostEvidenceTraceabilityGetOptions {
        entry: Some("serial".into()),
        ..Default::default()
    };
    let initial = traceability.get_with(options).await?;
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
    let options = HostEvidenceTraceabilitySetOptions {
        request_id: Some("first".into()),
        ..Default::default()
    };
    let saved = traceability.set_with(first.clone(), initial.revision.clone(), "serial".into(), options).await?;
    let options = HostEvidenceTraceabilitySetOptions {
        request_id: Some("first".into()),
        ..Default::default()
    };
    if traceability.set_with(first.clone(), initial.revision.clone(), "serial".into(), options).await? != saved {
        return Err(Error::Invalid("Set Traceability: Replay Lost Its Original Receipt".into()));
    }
    let mut changed = first.clone();
    changed.source.clear();
    let options = HostEvidenceTraceabilitySetOptions {
        request_id: Some("first".into()),
        ..Default::default()
    };
    if traceability.set_with(changed, initial.revision.clone(), "serial".into(), options).await.is_ok() {
        return Err(Error::Invalid("Set Traceability: Changed Request Payload Was Accepted".into()));
    }
    let options = HostEvidenceTraceabilitySetOptions {
        request_id: Some("stale".into()),
        ..Default::default()
    };
    if traceability.set_with(first.clone(), initial.revision.clone(), "serial".into(), options).await.is_ok() {
        return Err(Error::Invalid("Set Traceability: Stale Revision Was Accepted".into()));
    }
    let options = HostEvidenceTraceabilityGetOptions {
        entry: Some("batch".into()),
        ..Default::default()
    };
    let other = traceability.get_with(options).await?;
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
    let options = HostEvidenceTraceabilitySetOptions {
        request_id: Some("batch".into()),
        ..Default::default()
    };
    traceability.set_with(batch, other.revision, "batch".into(), options).await?;
    let options = HostEvidenceTraceabilityGetOptions {
        entry: Some("serial".into()),
        ..Default::default()
    };
    if traceability.get_with(options).await?.revision != saved.revision {
        return Err(Error::Invalid("Get Traceability: Another Entry Changed This Revision".into()));
    }

    // Await the Operator Before Replacing the Whole Entry
    let definition = dokime::plugin::Prompt::new("content.continue", "Continue", "Continue").action("continue", "Continue");
    let options = HostPromptsCreateOptions {
        definition: Some(definition.into_message()),
        ..Default::default()
    };
    ctx.host().prompts.create_with("continue".into(), "content.continue".into(), HashMap::new(), options).await?;
    let second = TraceabilityEntry {
        metadata: Some(Metadata {
            identifier: "serial".into(),
            ..Default::default()
        }),
        value: Some(Value { kind: Some(Kind::Text("second".into())) }),
        ..Default::default()
    };
    let options = HostEvidenceTraceabilitySetOptions {
        request_id: Some("second".into()),
        ..Default::default()
    };
    traceability.set_with(second, saved.revision.clone(), "serial".into(), options).await?;
    let options = HostEvidenceTraceabilitySetOptions {
        request_id: Some("first".into()),
        ..Default::default()
    };
    if traceability.set_with(first, initial.revision, "serial".into(), options).await? != saved {
        return Err(Error::Invalid("Set Traceability: Later Writes Changed an Earlier Receipt".into()));
    }
    Ok(())
}

async fn capture(ctx: &Context) -> Result<()> {
    // Open an Upload and Replay Its Receipt
    let uploads = ctx.host().files.uploads;
    let metadata = FileMetadata {
        scope: Some(ctx.execution_id().into()),
        media_type: "text/plain".into(),
        ..Default::default()
    };
    let options = HostFilesUploadsCreateOptions {
        request_id: Some("capture/create".into()),
        ..Default::default()
    };
    let created = uploads.create_with(metadata.clone(), options).await?;
    let options = HostFilesUploadsCreateOptions {
        request_id: Some("capture/create".into()),
        ..Default::default()
    };
    if uploads.create_with(metadata, options).await? != created {
        return Err(Error::Invalid("Create Upload: Replay Changed Its Session".into()));
    }
    let upload = created.upload.ok_or_else(|| Error::Invalid("Capture: Missing Upload Writer".into()))?;
    let bytes = b"replacement\n";
    let options = HostFilesUploadsAppendOptions {
        request_id: Some("capture/prefix".into()),
        ..Default::default()
    };
    let saved = uploads.append_with(upload.clone(), 0, bytes[..11].to_vec(), options).await?;
    let options = HostFilesUploadsAppendOptions {
        request_id: Some("capture/prefix".into()),
        ..Default::default()
    };
    if uploads.append_with(upload.clone(), 0, bytes[..11].to_vec(), options).await? != saved {
        return Err(Error::Invalid("Append Upload: Replay Changed Its Receipt".into()));
    }

    // Refuse Gaps and Fence the Previous Writer
    let options = HostFilesUploadsAppendOptions {
        request_id: Some("capture/gap".into()),
        ..Default::default()
    };
    if uploads.append_with(upload.clone(), 30, b"gap".to_vec(), options).await.is_ok() {
        return Err(Error::Invalid("Capture: Offset Gap Was Accepted".into()));
    }
    let options = HostFilesUploadsResumeOptions {
        request_id: Some("capture/resume".into()),
        ..Default::default()
    };
    let resumed = uploads.resume_with(upload.clone(), options).await?;
    let options = HostFilesUploadsAppendOptions {
        request_id: Some("capture/stale".into()),
        ..Default::default()
    };
    if uploads.append_with(upload, 11, b"\n".to_vec(), options).await.is_ok() {
        return Err(Error::Invalid("Capture: Stale Writer Was Accepted".into()));
    }
    let upload = resumed.upload.ok_or_else(|| Error::Invalid("Capture: Missing Resumed Writer".into()))?;
    let options = HostFilesUploadsAppendOptions {
        request_id: Some("capture/suffix".into()),
        ..Default::default()
    };
    uploads.append_with(upload.clone(), 11, b"\n".to_vec(), options).await?;

    // Publish Verified Content and Attach the Artifact
    let options = HostFilesUploadsPublishOptions {
        request_id: Some("capture/publish".into()),
        ..Default::default()
    };
    let published = uploads.publish_with(upload, bytes.len() as u64, Sha256::digest(bytes).to_vec(), options).await?;
    let file = published.upload.ok_or_else(|| Error::Invalid("Capture: Missing Published Content".into()))?;
    let options = HostContentArtifactsCreateOptions {
        request_id: Some("capture/attach".into()),
        metadata: Some(Metadata {
            identifier: "producer.txt".into(),
            label: "producer.txt".into(),
            ..Default::default()
        }),
        ..Default::default()
    };
    ctx.host().content.artifacts.create_with(file, "producer.txt".into(), "text/plain".into(), options).await?;

    // Abandon a Separate Upload Without Publishing It
    let metadata = FileMetadata {
        scope: Some(ctx.execution_id().into()),
        ..Default::default()
    };
    let options = HostFilesUploadsCreateOptions {
        request_id: Some("abandoned/create".into()),
        ..Default::default()
    };
    let abandoned = uploads.create_with(metadata, options).await?;
    let upload = abandoned.upload.ok_or_else(|| Error::Invalid("Capture: Missing Abandoned Writer".into()))?;
    let options = HostFilesUploadsDeleteOptions {
        request_id: Some("abandoned/delete".into()),
        ..Default::default()
    };
    uploads.delete_with(upload, options).await?;
    Ok(())
}

/* === Tests === */
