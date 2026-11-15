/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Artifacts and Traceability Entries

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use crate::catalog::metadata;
use dokime::plugin::{Context, HostEvidenceTraceabilityGetOptions};
use dokime::{Error, FileMetadata, Result, TraceabilityEntry, Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Upload Bytes Into This Execution's Scope and Attach Them as a Named Artifact.
pub async fn attach(ctx: &Context, filename: &str, content_type: &str, bytes: &[u8]) -> Result<()> {
    let uploads = ctx.host().files.uploads;
    let sha256 = Sha256::digest(bytes).to_vec();
    let metadata = FileMetadata {
        scope: Some(ctx.execution_id().into()),
        media_type: content_type.into(),
        expected_length: Some(bytes.len() as u64),
        expected_sha256: Some(sha256.clone()),
        ..Default::default()
    };
    let created = uploads.create(metadata).await?;
    let upload = created.upload.ok_or_else(|| Error::Invalid(format!("Attach {filename}: Missing Upload Writer")))?;
    let chunk = usize::try_from(created.chunk_bytes.max(1)).unwrap_or(usize::MAX);
    for (index, part) in bytes.chunks(chunk).enumerate() {
        uploads.append(upload.clone(), (index * chunk) as u64, part.to_vec()).await?;
    }
    let published = uploads.publish(upload, bytes.len() as u64, sha256).await?;
    let file = published.upload.ok_or_else(|| Error::Invalid(format!("Attach {filename}: Missing Published File")))?;
    ctx.host().content.artifacts.create(file, filename.into(), content_type.into()).await?;
    Ok(())
}

/// Record One Traceability Entry for This Execution, Replacing Its Current Revision.
pub async fn trace(ctx: &Context, id: &str, label: &str, value: Value, unit: &str, source: &str) -> Result<()> {
    let traceability = ctx.host().evidence.traceability;
    let options = HostEvidenceTraceabilityGetOptions {
        entry: Some(id.into()),
        ..Default::default()
    };
    let current = traceability.get_with(options).await?;
    let entry = TraceabilityEntry {
        metadata: Some(metadata(id, label)),
        value: Some(value),
        source: source.into(),
        unit: unit.into(),
    };
    traceability.set(entry, current.revision, id.into()).await?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
