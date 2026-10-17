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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::storage::{ByteRange, ObjectRevision, ObjectVersion, ReadFile};

// Local Imports
use crate::execution_runtime::outputs::{
    ExecutionFileContent, ExecutionFileContentKind, ExecutionFileEntry, ExecutionFileStreamState,
    MAX_ARTIFACT_READ_BYTES,
};
use crate::{runtime::Error, runtime::Result};
use crate::{
    core::TestExecutionId,
    execution::{
        TestExecution,
        saved::{
            ExecutionCheckpoint, ExecutionOutput, ExecutionOutputContent, SavedStreamState,
            validate_relative_output_path,
        },
    },
    storage::Manager,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// The sealed outputs of one stored execution revision.
pub(crate) struct SavedOutputs {
    storage: Manager,
    revision: ObjectRevision<TestExecution>,
}

/// One sealed output file, read by byte range.
pub(crate) struct SavedOutputFile {
    outputs: SavedOutputs,
    index: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl SavedOutputs {
    /// The latest sealed revision, or the exact checkpointed one.
    pub(crate) async fn open(
        storage: &Manager,
        id: &TestExecutionId,
        checkpoint: Option<&ExecutionCheckpoint>,
    ) -> Result<Option<Self>> {
        let version = match checkpoint {
            Some(point) => {
                point.validate().map_err(crate::storage::Error::from)?;
                if &point.execution_id != id {
                    return Err(invalid("saved output checkpoint execution differs"));
                }
                ObjectVersion::Exact(point.version.clone())
            }
            None => ObjectVersion::Latest,
        };
        let Some(revision) =
            crate::storage::objects::execution::read(storage, id, version).await?
        else {
            return Ok(None);
        };
        if let Some(point) = checkpoint
            && (revision.position != point.position || revision.version != point.version)
        {
            return Err(invalid("saved output checkpoint revision differs"));
        }
        let Some(seal) = &revision.value.saved else {
            if checkpoint.is_some() {
                return Err(invalid("saved output checkpoint is unsealed"));
            }
            return Ok(None);
        };
        seal.validate(&revision.value)
            .map_err(crate::storage::Error::from)?;
        if revision.position.history != seal.dependencies_saved.history
            || revision.position.sequence <= seal.dependencies_saved.sequence
            || checkpoint
                .is_some_and(|point| point.evidence_through != seal.source.evidence_through)
        {
            return Err(invalid(
                "saved output seal admission or evidence cut differs",
            ));
        }
        Ok(Some(Self {
            storage: storage.clone(),
            revision,
        }))
    }

    fn outputs(&self) -> &[ExecutionOutput] {
        &self
            .revision
            .value
            .saved
            .as_ref()
            .expect("sealed output selection")
            .source
            .outputs
    }

    pub(crate) fn entries(&self) -> Result<Vec<ExecutionFileEntry>> {
        let id = &self.revision.value.test_execution_id;
        let mut entries = self
            .outputs()
            .iter()
            .map(|output| entry(id, output))
            .collect::<harmos::Result<Vec<_>>>()?;
        entries.sort_unstable_by(|left, right| left.relative_path.cmp(&right.relative_path));
        Ok(entries)
    }

    pub(crate) fn file(self, path: &str) -> Result<Option<SavedOutputFile>> {
        validate_relative_output_path(path).map_err(crate::storage::Error::from)?;
        let Some(index) = self
            .outputs()
            .iter()
            .position(|output| output.relative_path == path)
        else {
            return Ok(None);
        };
        Ok(Some(SavedOutputFile {
            outputs: self,
            index,
        }))
    }
}

impl SavedOutputFile {
    pub(crate) fn descriptor(&self) -> &ExecutionOutput {
        &self.outputs.outputs()[self.index]
    }

    pub(crate) fn metadata(&self) -> ExecutionFileContent {
        window(self.descriptor(), 0, Vec::new())
    }

    /// Up to `maximum` verified bytes from `offset`.
    pub(crate) async fn window(
        &self,
        offset: u64,
        maximum: NonZeroUsize,
    ) -> Result<ExecutionFileContent> {
        let maximum = maximum.get().min(MAX_ARTIFACT_READ_BYTES);
        let output = self.descriptor();
        let offset = offset.min(output.file.length);
        let bytes = self.read(offset, maximum as u64).await?;
        Ok(window(output, offset, bytes))
    }

    /// The whole file, or its first `maximum` bytes unless it must be complete.
    pub(crate) async fn snapshot(
        &self,
        maximum: NonZeroUsize,
        require_complete: bool,
    ) -> Result<ExecutionFileContent> {
        let output = self.descriptor();
        if require_complete && output.file.length > maximum.get() as u64 {
            return Err(crate::runtime::Error::ExecutionFileTooLarge);
        }
        let bytes = self.read(0, maximum.get() as u64).await?;
        Ok(window(output, 0, bytes))
    }

    async fn read(&self, offset: u64, limit: u64) -> Result<Vec<u8>> {
        let mut reader = self
            .outputs
            .storage
            .read_file(ReadFile {
                file: self.descriptor().file.clone(),
                range: ByteRange {
                    offset,
                    limit: Some(limit),
                },
            })
            .await?;
        let mut bytes = Vec::new();
        while let Some(batch) = reader.next().await? {
            bytes.extend_from_slice(&batch.bytes);
        }
        Ok(bytes)
    }
}

/* === Public Functions === */
pub(crate) fn entry(
    id: &TestExecutionId,
    output: &ExecutionOutput,
) -> harmos::Result<ExecutionFileEntry> {
    let artifact_id = match &output.content {
        ExecutionOutputContent::Artifact { record, .. } => record.artifact_id.as_str().to_owned(),
        ExecutionOutputContent::File => format!("{}:{}", id.as_str(), output.relative_path),
    };
    Ok(ExecutionFileEntry {
        artifact_id,
        relative_path: output.relative_path.clone(),
        content_type: Some(output.file.metadata.media_type.clone()),
        content_kind: ExecutionFileContentKind::from_metadata(
            &output.relative_path,
            Some(&output.file.metadata.media_type),
        ),
        stream_state: state(output),
        size_bytes: output.file.length,
        updated_at: output.created_at.clone(),
    })
}

/* === Private Functions === */
fn state(output: &ExecutionOutput) -> ExecutionFileStreamState {
    match output.stream_state {
        Some(SavedStreamState::Interrupted) => ExecutionFileStreamState::Interrupted,
        Some(SavedStreamState::Finalized) | None => ExecutionFileStreamState::Finalized,
    }
}

fn window(output: &ExecutionOutput, offset: u64, bytes: Vec<u8>) -> ExecutionFileContent {
    ExecutionFileContent {
        relative_path: output.relative_path.clone(),
        content_type: Some(output.file.metadata.media_type.clone()),
        content_kind: ExecutionFileContentKind::from_metadata(
            &output.relative_path,
            Some(&output.file.metadata.media_type),
        ),
        stream_state: state(output),
        offset,
        size_bytes: output.file.length,
        bytes,
    }
}

fn invalid(message: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(message))
}

/* === Tests === */
