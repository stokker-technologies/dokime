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
use harmos::storage::FileRead;
use harmos::storage::{ByteRange, ObjectRevision, ObjectVersion};

// Local Imports
use crate::execution_runtime::outputs::{ExecutionFileContent, ExecutionFileContentKind, ExecutionFileEntry, ExecutionFileStreamState};
use crate::storage::Manager;
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::execution::saved::ExecutionCheckpoint;
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionOutputContent;
use dokime_domain::model::execution::saved::SavedStreamState;
use dokime_domain::model::execution::saved::validate_relative_output_path;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct SavedOutputs {
    session: crate::storage::Manager,
    revision: harmos::storage::ReadBatch<ObjectRevision<TestExecution>>,
}

pub(crate) struct SavedOutputFile {
    outputs: SavedOutputs,
    index: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl SavedOutputs {
    pub(crate) async fn open(storage: &Manager, id: &TestExecutionId, checkpoint: Option<&ExecutionCheckpoint>) -> Result<Option<Self>> {
        Self::open_in(&storage.clone(), id, checkpoint).await
    }

    pub(crate) async fn open_in(
        session: &crate::storage::Manager, id: &TestExecutionId, checkpoint: Option<&ExecutionCheckpoint>,
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
        let Some(revision) = crate::storage::objects::execution::read(session, id, version).await? else {
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
        seal.validate(&revision.value).map_err(crate::storage::Error::from)?;
        if revision.position.history != seal.dependencies_saved.history
            || revision.position.sequence <= seal.dependencies_saved.sequence
            || checkpoint.is_some_and(|point| point.evidence_through != seal.source.evidence_through)
        {
            return Err(invalid("saved output seal admission or evidence cut differs"));
        }
        Ok(Some(Self { session: session.clone(), revision }))
    }

    fn outputs(&self) -> &[ExecutionOutput] {
        &self.revision.value.saved.as_ref().expect("sealed output selection").source.outputs
    }

    pub(crate) fn entries(&self) -> Result<Vec<ExecutionFileEntry>> {
        let id = &self.revision.value.test_execution_id;
        let mut entries = self.outputs().iter().map(|output| entry(id, output)).collect::<harmos::Result<Vec<_>>>()?;
        entries.sort_unstable_by(|left, right| left.relative_path.cmp(&right.relative_path));
        Ok(entries)
    }

    pub(crate) fn file(self, path: &str) -> Result<Option<SavedOutputFile>> {
        validate_relative_output_path(path).map_err(crate::storage::Error::from)?;
        let Some(index) = self.outputs().iter().position(|output| output.relative_path == path) else {
            return Ok(None);
        };
        Ok(Some(SavedOutputFile { outputs: self, index }))
    }
}

impl SavedOutputFile {
    pub(crate) fn descriptor(&self) -> &ExecutionOutput {
        &self.outputs.outputs()[self.index]
    }

    pub(crate) fn session(&self) -> &crate::storage::Manager {
        &self.outputs.session
    }

    pub(crate) fn metadata(&self) -> Result<harmos::storage::ReadBatch<ExecutionFileContent>> {
        let output = self.descriptor();
        harmos::storage::ReadBatch::new(()).try_map(|_| Ok(window(output, 0, Vec::new())))
    }

    pub(crate) async fn window(&self, offset: u64, maximum: NonZeroUsize) -> Result<harmos::storage::ReadBatch<ExecutionFileContent>> {
        let maximum = maximum.get();
        let output = self.descriptor();
        let offset = offset.min(output.file.length);
        self.read(offset, maximum as u64).await
    }

    pub(crate) async fn snapshot(&self) -> Result<harmos::storage::ReadBatch<ExecutionFileContent>> {
        self.read(0, self.descriptor().file.length).await
    }

    async fn read(&self, offset: u64, limit: u64) -> Result<harmos::storage::ReadBatch<ExecutionFileContent>> {
        let metadata = self.metadata()?;
        let length = usize::try_from((metadata.size_bytes - offset).min(limit)).map_err(|_| invalid("saved file extent overflows"))?;
        let reader = self
            .outputs
            .session
            .read_file(FileRead {
                source: harmos::storage::FileSource::Stored(self.descriptor().file.clone()),
                range: ByteRange { offset, limit: Some(limit) },
            })
            .await?;
        crate::runtime::storage::native_outputs::reader::collect(metadata, reader, offset, length).await
    }
}

/* === Public Functions === */

pub(crate) fn entry(id: &TestExecutionId, output: &ExecutionOutput) -> harmos::Result<ExecutionFileEntry> {
    let artifact_id = match &output.content {
        ExecutionOutputContent::Artifact { record, .. } => record.artifact_id.as_str().to_owned(),
        ExecutionOutputContent::File => {
            let bytes = id
                .as_str()
                .len()
                .checked_add(1)
                .and_then(|bytes| bytes.checked_add(output.relative_path.len()))
                .ok_or(harmos::Error::ResourceExhausted { resource: "execution file identity" })?;
            let mut value = String::new();
            value.try_reserve_exact(bytes).map_err(|_| harmos::Error::ResourceExhausted { resource: "execution file identity" })?;
            value.push_str(id.as_str());
            value.push(':');
            value.push_str(&output.relative_path);
            value
        }
    };
    Ok(ExecutionFileEntry {
        artifact_id,
        relative_path: output.relative_path.clone(),
        content_type: Some(output.file.metadata.media_type.clone()),
        content_kind: ExecutionFileContentKind::from_metadata(&output.relative_path, Some(&output.file.metadata.media_type)),
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
        content_kind: ExecutionFileContentKind::from_metadata(&output.relative_path, Some(&output.file.metadata.media_type)),
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
