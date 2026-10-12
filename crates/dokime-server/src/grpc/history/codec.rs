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
pub use super::super::charged::{Bytes as Page, Codec};

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use prost::Message;
// Local Imports
use dokime_app::runtime::query::execution_history::{
    ExecutionBrowsePage, ExecutionCheckpoint, ExecutionCheckpointContext,
    ExecutionCheckpointDocument,
};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
/* === Public Functions === */
pub(super) fn encode_checkpoint(
    checkpoint: &ExecutionCheckpoint,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    let graph = [
        checkpoint.execution_id.as_str().len(),
        harmos::Message::encoded_len(&checkpoint.version),
        checkpoint.position.history.store.as_str().len(),
        checkpoint.position.history.epoch.as_str().len(),
        checkpoint.evidence_through.history.store.as_str().len(),
        checkpoint.evidence_through.history.epoch.as_str().len(),
    ]
    .into_iter()
    .try_fold(
        std::mem::size_of::<v1::ExecutionCheckpoint>(),
        |bytes, field| bytes.checked_add(field).ok_or_else(exhausted),
    )?;
    if graph > allowance.get() {
        return Err(exhausted());
    }
    let wire = super::wire::checkpoint_wire_ref(checkpoint).map_err(|source| {
        if source.code() == tonic::Code::ResourceExhausted {
            return exhausted();
        }
        harmos::Error::backend(
            "encode inspected execution checkpoint",
            harmos::BackendFailure::Corrupt,
            source,
        )
    })?;
    let length = wire.encoded_len();
    let peak = graph
        .checked_add(std::mem::size_of::<Vec<u8>>())
        .and_then(|bytes| bytes.checked_add(length))
        .ok_or_else(exhausted)?;
    if peak > allowance.get() || length > maximum {
        return Err(exhausted());
    }
    let mut output = super::super::value::projection::slots(length)?;
    wire.encode(&mut output).map_err(|_| invalid_wire())?;
    if output.len() != length || output.capacity() > length {
        return Err(invalid_wire());
    }
    Ok(output)
}

pub(super) fn encode_document(
    document: ExecutionCheckpointDocument,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    let native = harmos::MemorySize::memory_size(&document);
    if native == usize::MAX {
        return Err(exhausted());
    }
    let graph = super::construction::document(&document)?;
    let scratch = native.checked_add(graph).ok_or_else(exhausted)?;
    if scratch > allowance.get() {
        return Err(exhausted());
    }
    let wire = super::wire::checkpoint_document(document).map_err(|source| {
        if source.code() == tonic::Code::ResourceExhausted {
            return exhausted();
        }
        harmos::Error::backend(
            "project captured checkpoint document",
            harmos::BackendFailure::Corrupt,
            source,
        )
    })?;
    let length = wire.encoded_len();
    let peak = scratch
        .checked_add(std::mem::size_of::<Vec<u8>>())
        .and_then(|bytes| bytes.checked_add(length))
        .ok_or_else(exhausted)?;
    if peak > allowance.get() || length > maximum {
        return Err(exhausted());
    }
    let mut bytes = super::super::value::projection::slots(length)?;
    wire.encode(&mut bytes).map_err(|_| invalid_wire())?;
    if bytes.len() != length || bytes.capacity() > length {
        return Err(invalid_wire());
    }
    Ok(bytes)
}

pub(super) fn encode_context(
    context: ExecutionCheckpointContext,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    let native = harmos::MemorySize::memory_size(&context);
    if native == usize::MAX {
        return Err(exhausted());
    }
    let graph = super::construction::context(&context)?;
    let scratch = native.checked_add(graph).ok_or_else(exhausted)?;
    if scratch > allowance.get() {
        return Err(exhausted());
    }
    let wire = super::wire::checkpoint_context(context).map_err(|source| {
        if source.code() == tonic::Code::ResourceExhausted {
            return exhausted();
        }
        harmos::Error::backend(
            "project captured checkpoint context",
            harmos::BackendFailure::Corrupt,
            source,
        )
    })?;
    let length = wire.encoded_len();
    let peak = scratch
        .checked_add(std::mem::size_of::<Vec<u8>>())
        .and_then(|bytes| bytes.checked_add(length))
        .ok_or_else(exhausted)?;
    if peak > allowance.get() || length > maximum {
        return Err(exhausted());
    }
    let mut bytes = super::super::value::projection::slots(length)?;
    wire.encode(&mut bytes).map_err(|_| invalid_wire())?;
    if bytes.len() != length || bytes.capacity() > length {
        return Err(invalid_wire());
    }
    Ok(bytes)
}

pub(super) fn encode_evidence(
    evidence: dokime_app::runtime::query::execution_history::ExecutionCheckpointEvidence,
    sections: &[v1::EvidenceSection],
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    let native = harmos::MemorySize::memory_size(&evidence);
    if native == usize::MAX {
        return Err(exhausted());
    }
    let graph = super::construction::evidence(&evidence, sections)?;
    let scratch = native.checked_add(graph).ok_or_else(exhausted)?;
    if scratch > allowance.get() {
        return Err(exhausted());
    }
    let wire = super::wire::checkpoint_evidence(evidence, sections).map_err(|source| {
        if source.code() == tonic::Code::ResourceExhausted {
            return exhausted();
        }
        harmos::Error::backend(
            "project captured checkpoint evidence",
            harmos::BackendFailure::Corrupt,
            source,
        )
    })?;
    let length = wire.encoded_len();
    let peak = scratch
        .checked_add(std::mem::size_of::<Vec<u8>>())
        .and_then(|bytes| bytes.checked_add(length))
        .ok_or_else(exhausted)?;
    if peak > allowance.get() || length > maximum {
        return Err(exhausted());
    }
    let mut bytes = super::super::value::projection::slots(length)?;
    wire.encode(&mut bytes).map_err(|_| invalid_wire())?;
    if bytes.len() != length || bytes.capacity() > length {
        return Err(invalid_wire());
    }
    Ok(bytes)
}

pub(super) fn encode_window(
    checkpoint: ExecutionCheckpoint,
    content: dokime_app::ExecutionFileContent,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    use harmos::MemorySize;
    let graph = std::mem::size_of::<v1::CheckpointFileWindow>()
        .saturating_add(std::mem::size_of::<v1::ExecutionFileWindow>())
        .saturating_add(checkpoint.memory_size())
        .saturating_add(content.memory_size());
    let scratch = graph
        .saturating_add(checkpoint.memory_size())
        .saturating_add(content.memory_size());
    if scratch > allowance.get() {
        return Err(exhausted());
    }
    let failure = |source: tonic::Status| {
        if source.code() == tonic::Code::ResourceExhausted {
            return exhausted();
        }
        harmos::Error::backend(
            "encode checkpoint output window",
            harmos::BackendFailure::Corrupt,
            source,
        )
    };
    let wire = v1::CheckpointFileWindow {
        checkpoint: Some(super::wire::checkpoint_wire(checkpoint).map_err(failure)?),
        content: Some(super::wire::file_window(content).map_err(failure)?),
    };
    let length = wire.encoded_len();
    let peak = scratch
        .saturating_add(std::mem::size_of::<Vec<u8>>())
        .saturating_add(length);
    if peak > allowance.get() || length > maximum {
        return Err(exhausted());
    }
    let mut bytes = super::super::value::projection::slots(length)?;
    wire.encode(&mut bytes).map_err(|_| invalid_wire())?;
    if bytes.len() != length || bytes.capacity() > length {
        return Err(invalid_wire());
    }
    Ok(bytes)
}

pub(super) fn encode_outputs(
    checkpoint: &ExecutionCheckpoint,
    outputs: &[dokime_app::execution::saved::ExecutionOutput],
    sections: &[v1::EvidenceSection],
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    let native = harmos::MemorySize::memory_size(checkpoint);
    if native == usize::MAX {
        return Err(exhausted());
    }
    let graph = super::construction::outputs(checkpoint, outputs, sections)?;
    let scratch = native.checked_add(graph).ok_or_else(exhausted)?;
    if scratch > allowance.get() {
        return Err(exhausted());
    }
    let wire =
        super::wire::checkpoint_outputs(checkpoint, outputs, sections).map_err(|source| {
            if source.code() == tonic::Code::ResourceExhausted {
                return exhausted();
            }
            harmos::Error::backend(
                "project captured output descriptors",
                harmos::BackendFailure::Corrupt,
                source,
            )
        })?;
    let length = wire.encoded_len();
    let peak = scratch
        .checked_add(std::mem::size_of::<Vec<u8>>())
        .and_then(|bytes| bytes.checked_add(length))
        .ok_or_else(exhausted)?;
    if peak > allowance.get() || length > maximum {
        return Err(exhausted());
    }
    let mut bytes = super::super::value::projection::slots(length)?;
    wire.encode(&mut bytes).map_err(|_| invalid_wire())?;
    if bytes.len() != length || bytes.capacity() > length {
        return Err(invalid_wire());
    }
    Ok(bytes)
}

pub(super) fn encode(
    page: &ExecutionBrowsePage,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    // Canonical Strings Copy Borrowed Metadata; Message Slots And Token Bytes
    // Are Admitted Separately Before The Wire Graph Is Constructed.
    let scratch = construction_memory(page);
    if scratch > allowance.get() {
        return Err(exhausted());
    }
    let wire = super::wire::browse_page(page).map_err(|_| invalid_wire())?;
    let length = wire.encoded_len();
    let required = scratch
        .saturating_add(std::mem::size_of::<Vec<u8>>())
        .saturating_add(length);
    if length > maximum || required > allowance.get() {
        return Err(exhausted());
    }
    let mut output = Vec::new();
    output.try_reserve_exact(length).map_err(|_| exhausted())?;
    if output.capacity() > length {
        return Err(exhausted());
    }
    wire.encode(&mut output).map_err(|_| invalid_wire())?;
    if output.len() != length {
        return Err(invalid_wire());
    }
    Ok(output)
}

/* === Private Functions === */
fn construction_memory(page: &ExecutionBrowsePage) -> usize {
    let slots = std::mem::size_of::<v1::ExecutionBrowsePage>()
        .saturating_add(
            page.entries
                .len()
                .saturating_mul(std::mem::size_of::<v1::ExecutionBrowseEntry>()),
        )
        .saturating_add(
            page.conflicts
                .len()
                .saturating_mul(std::mem::size_of::<String>()),
        )
        .saturating_add(
            page.issues
                .len()
                .saturating_mul(std::mem::size_of::<v1::HistoryIssue>()),
        );
    let mut text = page.next.as_ref().map_or(0, |id| id.as_str().len());
    for id in &page.conflicts {
        text = text.saturating_add(id.as_str().len());
    }
    for issue in &page.issues {
        text = text
            .saturating_add(issue.message.len())
            .saturating_add(issue.source.as_ref().map_or(0, String::len))
            .saturating_add(issue.dataset.as_ref().map_or(0, String::len));
    }
    for entry in &page.entries {
        let metadata = &entry.metadata;
        for length in [
            metadata.execution_id.as_str().len(),
            metadata.cycle_id.as_str().len(),
            metadata.case_id.as_str().len(),
            metadata.title.len(),
            metadata.template_id.as_ref().map_or(0, String::len),
            metadata.suite_label.as_ref().map_or(0, String::len),
        ] {
            text = text.saturating_add(length);
        }
        if let Some(checkpoint) = &entry.checkpoint {
            text = text
                .saturating_add(checkpoint.execution_id.as_str().len())
                .saturating_add(harmos::Message::encoded_len(&checkpoint.version));
            for cursor in [&checkpoint.position, &checkpoint.evidence_through] {
                text = text
                    .saturating_add(cursor.history.store.as_str().len())
                    .saturating_add(cursor.history.epoch.as_str().len());
            }
        }
    }
    // Optional Canonical Messages Are Inline In Each Entry Slot. Timestamp And
    // Result Conversion Own No Heap Fields; All Copied Text Uses Exact Capacity.
    slots.saturating_add(text)
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "historical response",
    }
}

fn invalid_wire() -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "historical response",
        reason: "canonical wire projection failed",
    }
}

/* === Tests === */
