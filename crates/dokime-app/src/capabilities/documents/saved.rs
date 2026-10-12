/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod snapshot;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{
        ObjectQuery, ObjectQueryResult, ObjectRevision, ObjectVersions, QueryOptions, QueryValue,
        ReadLimits, StorageFileReader,
    },
};

// Local Imports
use self::snapshot::PendingSnapshot;
use crate::execution_runtime::outputs::{
    ExecutionFileContent, ExecutionFileContentKind, ExecutionFileEntry, ExecutionFileStreamState,
    MAX_ARTIFACT_READ_BYTES,
};
use crate::{Error, Result};
use dokime::{
    core::{TestExecutionId, Timestamp},
    execution::{
        TestExecution,
        saved::{
            ExecutionCheckpoint, ExecutionOutput, ExecutionOutputContent, SavedStreamState,
            validate_relative_output_path,
        },
    },
    storage::{Manager, objects::execution::Execution},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct SavedOutputs {
    original: ObjectQueryResult<Execution>,
    revision: QueryValue<ObjectRevision<TestExecution>>,
}

pub(crate) struct SavedOutputFile {
    outputs: SavedOutputs,
    index: usize,
    reader: StorageFileReader,
    window: Option<PendingWindow>,
    snapshot: Option<PendingSnapshot>,
}

struct PendingWindow {
    offset: u64,
    maximum: NonZeroUsize,
    bytes: Option<QueryValue<Vec<u8>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl SavedOutputs {
    pub(crate) async fn open(
        storage: &Manager,
        id: &TestExecutionId,
        checkpoint: Option<&ExecutionCheckpoint>,
        mut options: QueryOptions,
        projection: NonZeroUsize,
    ) -> Result<Option<Self>> {
        options.page.get_or_insert(ReadLimits {
            rows: NonZeroUsize::new(1).unwrap(),
            bytes: projection,
        });
        if let Some(point) = checkpoint {
            point.validate().map_err(dokime::storage::Error::from)?;
            if &point.execution_id != id {
                return Err(invalid("saved output checkpoint execution differs"));
            }
            options.after = point
                .position
                .sequence
                .checked_sub(1)
                .ok_or_else(|| invalid("saved output checkpoint has no admission position"))?;
            options.through = Some(point.position.sequence);
        }
        let mut query = ObjectQuery::<Execution>::new();
        query.item = Some(id.as_str());
        query.options = options;
        query.versions = if checkpoint.is_some() {
            ObjectVersions::Revisions
        } else {
            ObjectVersions::Latest
        };
        let mut original = storage.query(query).await?;
        original.retain_view()?;
        let projection = projection.min(original.decoded_bytes());
        let Some(revision) = original
            .next_page_projected(projection, |rows, allowance| {
                let [row] = rows else {
                    return Err(refused(
                        "saved output query did not select one original revision",
                    ));
                };
                let bound = row
                    .memory_size()
                    .checked_add(row.value.projection_memory().map_err(codec_error)?)
                    .ok_or_else(|| refused("saved output revision projection overflows"))?;
                if bound > allowance.get() {
                    return Err(refused(
                        "saved output revision exceeds projection admission",
                    ));
                }
                let value = row
                    .value
                    .clone()
                    .into_execution(allowance)
                    .map_err(codec_error)?;
                if &value.test_execution_id != id {
                    return Err(refused(
                        "saved output revision belongs to another execution",
                    ));
                }
                row.version.check_position(&row.position)?;
                if let Some(point) = checkpoint
                    && (row.position != point.position || row.version != point.version)
                {
                    return Err(refused("saved output checkpoint revision differs"));
                }
                if let Some(seal) = &value.saved {
                    seal.validate(&value).map_err(|source| {
                        harmos::Error::backend(
                            "validate saved output seal",
                            harmos::BackendFailure::Corrupt,
                            source,
                        )
                    })?;
                    if row.position.history != seal.dependencies_saved.history
                        || row.position.sequence <= seal.dependencies_saved.sequence
                        || checkpoint.is_some_and(|point| {
                            point.evidence_through != seal.source.evidence_through
                        })
                    {
                        return Err(refused(
                            "saved output seal admission or evidence cut differs",
                        ));
                    }
                } else if checkpoint.is_some() {
                    return Err(refused("saved output checkpoint is unsealed"));
                }
                Ok(ObjectRevision {
                    value,
                    position: row.position.clone(),
                    version: row.version.clone(),
                })
            })
            .await?
        else {
            return Ok(None);
        };
        if revision.value.saved.is_none() {
            return Ok(None);
        }
        Ok(Some(Self { original, revision }))
    }

    pub(crate) fn entries(
        &mut self,
        projection: NonZeroUsize,
    ) -> Result<QueryValue<Vec<ExecutionFileEntry>>> {
        let projection = projection.min(self.original.decoded_bytes());
        let entries =
            self.original
                .project(&self.revision, projection, |revision, allowance| {
                    let outputs = &source(revision)?.outputs;
                    let id = &revision.value.test_execution_id;
                    let mut bytes = std::mem::size_of::<Vec<ExecutionFileEntry>>();
                    for output in outputs {
                        bytes = bytes
                            .checked_add(entry_memory(id, output))
                            .ok_or_else(|| exhausted("saved output inventory memory overflows"))?;
                    }
                    if bytes > allowance.get() {
                        return Err(refused(
                            "saved output inventory exceeds projection admission",
                        ));
                    }
                    let mut entries = slots(outputs.len())?;
                    for output in outputs {
                        entries.push(entry(id, output)?);
                    }
                    entries.sort_unstable_by(|left, right| {
                        left.relative_path.cmp(&right.relative_path)
                    });
                    Ok(entries)
                })?;
        Ok(entries)
    }

    pub(crate) fn project<T: MemorySize, U: MemorySize>(
        &mut self,
        input: &QueryValue<T>,
        map: impl FnOnce(&T, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        let maximum = self.original.decoded_bytes();
        Ok(self.original.project(input, maximum, map)?)
    }

    pub(crate) async fn file(self, path: &str) -> Result<Option<SavedOutputFile>> {
        validate_relative_output_path(path).map_err(dokime::storage::Error::from)?;
        let outputs = &source(&self.revision)?.outputs;
        let Some(index) = outputs
            .iter()
            .position(|output| output.relative_path == path)
        else {
            return Ok(None);
        };
        let reference = &outputs[index].file;
        let view = self
            .original
            .view()
            .ok_or_else(|| invalid("saved output query lost its original view"))?;
        if !view.saved()?.covers(&reference.position)? {
            return Err(invalid("saved output file exceeds its original view"));
        }
        let reader = view.read_file(reference).await?;
        Ok(Some(SavedOutputFile {
            outputs: self,
            index,
            reader,
            window: None,
            snapshot: None,
        }))
    }
}

impl SavedOutputFile {
    pub(crate) fn descriptor(&self) -> &ExecutionOutput {
        &self
            .outputs
            .revision
            .value
            .saved
            .as_ref()
            .expect("sealed output selection")
            .source
            .outputs[self.index]
    }

    pub(crate) fn metadata(&mut self) -> Result<QueryValue<ExecutionFileContent>> {
        let output = self.descriptor();
        let memory = std::mem::size_of::<ExecutionFileContent>()
            .checked_add(output.relative_path.len())
            .and_then(|bytes| bytes.checked_add(output.file.metadata.media_type.len()))
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| invalid("saved metadata memory overflows"))?;
        let index = self.index;
        Ok(self.outputs.original.project(
            &self.outputs.revision,
            memory,
            |revision, allowance| window(&source(revision)?.outputs[index], 0, &[], allowance),
        )?)
    }

    pub(crate) fn window_memory(&self, offset: u64, maximum: NonZeroUsize) -> Result<NonZeroUsize> {
        let output = self.descriptor();
        let offset = offset.min(output.file.length);
        let remaining = usize::try_from((output.file.length - offset).min(maximum.get() as u64))
            .map_err(|_| invalid("saved output window length overflows"))?;
        std::mem::size_of::<ExecutionFileContent>()
            .checked_add(output.relative_path.len())
            .and_then(|bytes| bytes.checked_add(output.file.metadata.media_type.len()))
            .and_then(|bytes| bytes.checked_add(remaining))
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| invalid("saved output window memory overflows"))
    }

    pub(crate) fn project<T: MemorySize, U: MemorySize>(
        &mut self,
        input: &QueryValue<T>,
        maximum: NonZeroUsize,
        map: impl FnOnce(&T, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        let maximum = maximum.min(self.outputs.original.decoded_bytes());
        Ok(self.outputs.original.project(input, maximum, map)?)
    }

    pub(crate) async fn window(
        &mut self,
        offset: u64,
        maximum: NonZeroUsize,
        projection: NonZeroUsize,
    ) -> Result<QueryValue<ExecutionFileContent>> {
        if self.snapshot.is_some() {
            return Err(invalid("saved output has an unsettled snapshot projection"));
        }
        let maximum = NonZeroUsize::new(maximum.get().min(MAX_ARTIFACT_READ_BYTES)).unwrap();
        let offset = offset.min(self.descriptor().file.length);
        if let Some(pending) = &self.window {
            if pending.offset != offset || pending.maximum != maximum {
                return Err(invalid("saved output retry changed its original window"));
            }
        } else {
            if offset < self.reader.offset() {
                return Err(invalid(
                    "saved output window precedes the native reader position",
                ));
            }
            self.window = Some(PendingWindow {
                offset,
                maximum,
                bytes: None,
            });
        }
        while self.reader.offset() < offset {
            let remaining = usize::try_from(
                (offset - self.reader.offset()).min(MAX_ARTIFACT_READ_BYTES as u64),
            )
            .map_err(|_| invalid("saved output prefix length overflows"))?;
            if self
                .reader
                .next_chunk(NonZeroUsize::new(remaining).unwrap())
                .await?
                .is_none()
            {
                return Err(invalid("saved output ended before selected prefix"));
            }
        }
        if self
            .window
            .as_ref()
            .expect("selected window")
            .bytes
            .is_none()
            && let Some(chunk) = self.reader.next_chunk(maximum).await?
        {
            self.window.as_mut().expect("selected window").bytes = Some(chunk.into_query_value());
        }
        let index = self.index;
        let content = if let Some(bytes) = &self.window.as_ref().expect("selected window").bytes {
            let output = &source(&self.outputs.revision)?.outputs[index];
            self.outputs
                .original
                .project(bytes, projection, |bytes, allowance| {
                    window(output, offset, bytes, allowance)
                })?
        } else {
            self.outputs.original.project(
                &self.outputs.revision,
                projection,
                |revision, allowance| {
                    window(&source(revision)?.outputs[index], offset, &[], allowance)
                },
            )?
        };
        self.window = None;
        Ok(content)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn source(
    revision: &ObjectRevision<TestExecution>,
) -> harmos::Result<&dokime::execution::saved::ExecutionSource> {
    revision
        .value
        .saved
        .as_ref()
        .map(|seal| &seal.source)
        .ok_or_else(|| refused("saved output selection is unsealed"))
}
fn state(output: &ExecutionOutput) -> ExecutionFileStreamState {
    match output.stream_state {
        Some(SavedStreamState::Interrupted) => ExecutionFileStreamState::Interrupted,
        Some(SavedStreamState::Finalized) | None => ExecutionFileStreamState::Finalized,
    }
}
fn entry_memory(id: &TestExecutionId, output: &ExecutionOutput) -> usize {
    let identity = match &output.content {
        ExecutionOutputContent::File => id
            .as_str()
            .len()
            .saturating_add(1)
            .saturating_add(output.relative_path.len()),
        ExecutionOutputContent::Artifact { record, .. } => record.artifact_id.as_str().len(),
    };
    std::mem::size_of::<ExecutionFileEntry>()
        .saturating_add(identity)
        .saturating_add(output.relative_path.len())
        .saturating_add(output.file.metadata.media_type.len())
        .saturating_add(output.created_at.as_str().len())
}
pub(crate) fn entry(
    id: &TestExecutionId,
    output: &ExecutionOutput,
) -> harmos::Result<ExecutionFileEntry> {
    let artifact_id = match &output.content {
        ExecutionOutputContent::Artifact { record, .. } => text(record.artifact_id.as_str())?,
        ExecutionOutputContent::File => {
            let length = id
                .as_str()
                .len()
                .checked_add(1)
                .and_then(|length| length.checked_add(output.relative_path.len()))
                .ok_or_else(|| refused("saved output identity length overflows"))?;
            let mut value = text_buffer(length)?;
            value.push_str(id.as_str());
            value.push(':');
            value.push_str(&output.relative_path);
            value
        }
    };
    Ok(ExecutionFileEntry {
        artifact_id,
        relative_path: text(&output.relative_path)?,
        content_type: Some(text(&output.file.metadata.media_type)?),
        content_kind: ExecutionFileContentKind::from_metadata(
            &output.relative_path,
            Some(&output.file.metadata.media_type),
        ),
        stream_state: state(output),
        size_bytes: output.file.length,
        updated_at: Timestamp::new(text(output.created_at.as_str())?),
    })
}
fn window(
    output: &ExecutionOutput,
    offset: u64,
    bytes: &[u8],
    allowance: NonZeroUsize,
) -> harmos::Result<ExecutionFileContent> {
    let bound = std::mem::size_of::<ExecutionFileContent>()
        .checked_add(output.relative_path.len())
        .and_then(|bound| bound.checked_add(output.file.metadata.media_type.len()))
        .and_then(|bound| bound.checked_add(bytes.len()))
        .ok_or_else(|| exhausted("saved output window memory overflows"))?;
    if bound > allowance.get() {
        return Err(exhausted(
            "saved output window exceeds projection admission",
        ));
    }
    let mut selected = slots(bytes.len())?;
    selected.extend_from_slice(bytes);
    Ok(ExecutionFileContent {
        relative_path: text(&output.relative_path)?,
        content_type: Some(text(&output.file.metadata.media_type)?),
        content_kind: ExecutionFileContentKind::from_metadata(
            &output.relative_path,
            Some(&output.file.metadata.media_type),
        ),
        stream_state: state(output),
        offset,
        size_bytes: output.file.length,
        bytes: selected,
    })
}
fn text(value: &str) -> harmos::Result<String> {
    let mut target = text_buffer(value.len())?;
    target.push_str(value);
    Ok(target)
}
fn text_buffer(length: usize) -> harmos::Result<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(length)
        .map_err(|_| exhausted("saved output text allocation failed"))?;
    if value.capacity() > length {
        return Err(exhausted("saved output text capacity exceeds admission"));
    }
    Ok(value)
}
fn slots<T>(length: usize) -> harmos::Result<Vec<T>> {
    let mut value = Vec::new();
    value
        .try_reserve_exact(length)
        .map_err(|_| exhausted("saved output slot allocation failed"))?;
    if value.capacity() > length {
        return Err(exhausted("saved output slot capacity exceeds admission"));
    }
    Ok(value)
}
fn invalid(message: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(message))
}
fn refused(reason: &'static str) -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "saved execution output",
        reason,
    }
}
fn exhausted(resource: &'static str) -> harmos::Error {
    harmos::Error::ResourceExhausted { resource }
}
fn codec_error(source: dokime::storage::Error) -> harmos::Error {
    harmos::Error::backend(
        "project saved execution output",
        harmos::BackendFailure::Corrupt,
        source,
    )
}

/* === Tests === */
