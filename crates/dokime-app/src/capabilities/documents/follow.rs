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
use std::{num::NonZeroUsize, sync::Arc, time::Duration};
// Third Party Imports
use harmos::{
    MemorySize,
    services::Fault,
    services::Resident,
    services::Service,
    services::ServicePolicy,
    storage::{FileFollower, FilePhase, FileReference, QueryOptions, QueryValue, StagedFile},
};
// Local Imports
use super::saved::{SavedOutputFile, SavedOutputs};
use crate::{Error, Result, RuntimeHandle, runtime::storage::native_outputs::follow::LiveOutput};
use dokime::{
    core::{Identifier, TestExecutionId},
    execution::saved::SavedStreamState,
};
use dokime_contract::operations::{checkpoint, content as wire};

/* === Constants === */
const FRAME_MEMORY: usize = 64 * 1024 + 4096;
const SOURCE_POLL: Duration = Duration::from_millis(250);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct ExecutionFileFollower {
    reader: FileFollower,
    source: Source,
    metadata: QueryValue<wire::ExecutionFileFollowMetadata>,
    finished: bool,
}

struct NativeCapture {
    source: LiveOutput,
}

/* === Enums === */
enum Source {
    Live(LiveOutput),
    Saved(Box<SavedOutputFile>),
    Captured(QueryValue<FileReference>, bool),
}

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn follow_execution_file(
        &self,
        execution: &TestExecutionId,
        path: &str,
        expected: Option<&wire::ExecutionFileIdentity>,
        offset: u64,
    ) -> Result<Option<ExecutionFileFollower>> {
        dokime::execution::saved::validate_relative_output_path(path)
            .map_err(dokime::storage::Error::from)?;
        if offset != 0 && expected.is_none() {
            return Err(invalid("resume offset requires original identity"));
        }
        let expected_staged = expected
            .map(|identity| {
                if identity.execution_id != execution.as_str() || identity.relative_path != path {
                    return Err(invalid(
                        "resume identity belongs to another registered output",
                    ));
                }
                let history = identity
                    .history
                    .as_ref()
                    .ok_or_else(|| invalid("resume identity has no original history"))?;
                Ok(StagedFile {
                    history: harmos::storage::StoreHistory {
                        store: Identifier::new(&history.store)
                            .map_err(|_| invalid("invalid original store identity"))?,
                        epoch: Identifier::new(&history.epoch)
                            .map_err(|_| invalid("invalid original store epoch"))?,
                    },
                    id: Identifier::new(&identity.file_id)
                        .map_err(|_| invalid("invalid original file identity"))?,
                })
            })
            .transpose()?;
        let maximum = allowance()?;
        if let Some(saved) = SavedOutputs::open(
            self.storage(),
            execution,
            None,
            QueryOptions::default(),
            self.config().storage_reads.decoded_bytes,
        )
        .await?
        {
            let Some(mut file) = saved.file(path).await? else {
                return Ok(None);
            };
            let reference = &file.descriptor().file;
            let staged = StagedFile {
                history: reference.position.history.clone(),
                id: reference.metadata.id.clone(),
            };
            if expected_staged
                .as_ref()
                .is_some_and(|expected| expected != &staged)
            {
                return Err(invalid("saved output replaced the selected identity"));
            }
            if offset > reference.length {
                return Err(invalid("resume offset exceeds original saved length"));
            }
            let reader = self.storage().follow_file(&staged, offset).await?;
            let metadata = file.project_descriptor(maximum, |output, allowance| {
                let phase = if output.stream_state == Some(SavedStreamState::Interrupted) {
                    wire::ExecutionFilePhase::Abandoned
                } else {
                    wire::ExecutionFilePhase::Finished
                };
                metadata(
                    &staged,
                    execution,
                    path,
                    &output.file.metadata.media_type,
                    phase,
                    output.file.length,
                    offset,
                    allowance,
                )
            })?;
            return Ok(Some(ExecutionFileFollower {
                reader,
                source: Source::Saved(Box::new(file)),
                metadata,
                finished: false,
            }));
        }
        if let Some((reference, interrupted)) = self
            .execution_storage
            .captured_output(execution, path)
            .await?
        {
            let staged = StagedFile {
                history: reference.position.history.clone(),
                id: reference.metadata.id.clone(),
            };
            if expected_staged
                .as_ref()
                .is_some_and(|expected| expected != &staged)
            {
                return Err(invalid("captured output replaced the selected identity"));
            }
            if offset > reference.length {
                return Err(invalid("resume offset exceeds captured output length"));
            }
            let reader = self.storage().follow_file(&staged, offset).await?;
            let metadata = reference.try_project(maximum, |reference, allowance| {
                let phase = if interrupted {
                    wire::ExecutionFilePhase::Abandoned
                } else {
                    wire::ExecutionFilePhase::Finished
                };
                metadata(
                    &staged,
                    execution,
                    path,
                    &reference.metadata.media_type,
                    phase,
                    reference.length,
                    offset,
                    allowance,
                )
            })?;
            return Ok(Some(ExecutionFileFollower {
                reader,
                source: Source::Captured(reference, interrupted),
                metadata,
                finished: false,
            }));
        }
        let Some((reader, live)) = self
            .execution_storage
            .native_follower(execution, path, offset, expected_staged.as_ref())
            .await?
        else {
            return Ok(None);
        };
        self.start_execution_file_capture(execution, &live).await?;
        let progress = reader.progress();
        let media = crate::execution_runtime::outputs::media_type_for_path(path)
            .unwrap_or("application/octet-stream");
        let metadata = live.identity.try_project(maximum, |staged, allowance| {
            metadata(
                staged,
                execution,
                path,
                media,
                phase(progress.phase),
                progress.length,
                offset,
                allowance,
            )
        })?;
        Ok(Some(ExecutionFileFollower {
            reader,
            source: Source::Live(live),
            metadata,
            finished: false,
        }))
    }
}

impl RuntimeHandle {
    async fn start_execution_file_capture(
        &self,
        execution: &TestExecutionId,
        source: &LiveOutput,
    ) -> Result<()> {
        let runtime = self.clone();
        let source = source.clone();
        let bytes = std::mem::size_of::<String>()
            + std::mem::size_of::<LiveOutput>()
            + execution.as_str().len()
            + source.file_id().len()
            + "execution-log::native-".len();
        let admission = runtime.storage().prepare_production(
            NonZeroUsize::new(bytes)
                .ok_or_else(|| invalid("capture registration memory overflows"))?,
        )?;
        let name = format!("execution-log:{execution}:native-{}", source.file_id());
        tokio::spawn(async move {
            let _admission = admission;
            let mut stops = runtime.execution_storage.tail_stops.lock().await;
            if stops.contains_key(&name) {
                return Ok(());
            }
            let preparation = runtime.execution_storage.prepare_tail_stop(&name)?;
            runtime
                .services
                .register(
                    harmos::services::ServiceDefinition {
                        id: name.clone(),
                        policy: ServicePolicy::default(),
                    },
                    move |_: Arc<()>| NativeCapture {
                        source: source.clone(),
                    },
                )
                .map_err(Error::from)?;
            let lease = match runtime
                .services
                .activate(&name, &runtime.services.root_scope())
                .await
            {
                Ok(lease) => lease,
                Err(error) => {
                    runtime
                        .services
                        .unregister(&name)
                        .await
                        .map_err(Error::from)?;
                    return Err(error.into());
                }
            };
            let (key, barrier) = preparation.into_entry(lease);
            stops.insert(key, barrier);
            Ok(())
        })
        .await
        .map_err(|source| Error::Join { source })?
    }
}

impl Resident<crate::app::Dokime> for NativeCapture {
    type Error = Error;
    async fn attempt(
        &mut self,
        mut context: Service<crate::app::Dokime>,
    ) -> std::result::Result<(), Fault<Error>> {
        context.ready();
        loop {
            let stable = self
                .source
                .synchronize()
                .await
                .map_err(Fault::Application)?;
            if context.is_stopping() {
                return Ok(());
            }
            if !stable || self.source.caught_up().await.map_err(Fault::Application)? {
                context.rest(SOURCE_POLL).await;
            }
        }
    }
    async fn finalize(&mut self) -> Result<()> {
        while !self.source.caught_up().await? {
            if !self.source.synchronize().await? {
                tokio::time::sleep(SOURCE_POLL).await;
            }
        }
        Ok(())
    }
}

impl ExecutionFileFollower {
    pub fn metadata(&self) -> &wire::ExecutionFileFollowMetadata {
        &self.metadata
    }

    pub fn metadata_frame(&self) -> Result<QueryValue<wire::ExecutionFileFollowFrame>> {
        Ok(self
            .metadata
            .try_project(allowance()?, |metadata, allowance| {
                let frame = wire::ExecutionFileFollowFrame {
                    payload: Some(wire::ExecutionFileFollowFramePayload::Metadata(
                        metadata.clone(),
                    )),
                };
                admit(&frame, allowance)?;
                Ok(frame)
            })?)
    }

    pub async fn next_frame(
        &mut self,
        maximum: NonZeroUsize,
    ) -> Result<Option<QueryValue<wire::ExecutionFileFollowFrame>>> {
        if maximum.get() > 64 * 1024 {
            return Err(invalid("follow chunk exceeds 64 KiB"));
        }
        if self.finished {
            return Ok(None);
        }
        loop {
            if let Source::Live(live) = &self.source {
                live.synchronize().await?;
            }
            let offset = self.reader.offset();
            // Poll Only The External Native Source. The Original Harmos Reader Owns
            // Bounded Retained Bytes, History Checks And Terminal Production State.
            if let Source::Live(live) = &self.source {
                let progress = self.reader.progress();
                if progress.phase == FilePhase::Producing && offset == progress.length {
                    tokio::select! {
                        result = self.reader.next_chunk(maximum) => {
                            let chunk = result?;
                            if let Some(chunk) = chunk { return self.chunk(offset, chunk).map(Some); }
                        }
                        _ = tokio::time::sleep(SOURCE_POLL) => continue,
                    }
                } else if let Some(chunk) = self.reader.next_chunk(maximum).await? {
                    return self.chunk(offset, chunk).map(Some);
                }
                if live.interrupted().await? {
                    return self
                        .terminal(None, wire::ExecutionFilePhase::Abandoned)
                        .map(Some);
                }
                let reference = live.reference().await?;
                if self.reader.progress().phase == FilePhase::Abandoned {
                    return self
                        .terminal(None, wire::ExecutionFilePhase::Abandoned)
                        .map(Some);
                }
                let reference = reference.ok_or_else(|| {
                    invalid("finished original production has no sealed reference")
                })?;
                return self
                    .terminal(Some(&reference), wire::ExecutionFilePhase::Finished)
                    .map(Some);
            }
            if let Some(chunk) = self.reader.next_chunk(maximum).await? {
                return self.chunk(offset, chunk).map(Some);
            }
            if let Source::Captured(reference, interrupted) = &self.source {
                let interrupted = *interrupted;
                let reference = reference.try_project(allowance()?, |reference, allowance| {
                    admit(reference, allowance)?;
                    Ok(reference.clone())
                })?;
                let phase = if interrupted {
                    wire::ExecutionFilePhase::Abandoned
                } else {
                    wire::ExecutionFilePhase::Finished
                };
                return self
                    .terminal((!interrupted).then_some(&*reference), phase)
                    .map(Some);
            }
            let Source::Saved(file) = &mut self.source else {
                unreachable!()
            };
            let interrupted = file.descriptor().stream_state == Some(SavedStreamState::Interrupted);
            let reference = file.project_descriptor(allowance()?, |output, allowance| {
                admit(&output.file, allowance)?;
                Ok(output.file.clone())
            })?;
            let phase = if interrupted {
                wire::ExecutionFilePhase::Abandoned
            } else {
                wire::ExecutionFilePhase::Finished
            };
            return self
                .terminal((!interrupted).then_some(&*reference), phase)
                .map(Some);
        }
    }

    fn chunk(
        &self,
        offset: u64,
        chunk: harmos::storage::FileChunk,
    ) -> Result<QueryValue<wire::ExecutionFileFollowFrame>> {
        Ok(chunk
            .into_query_value()
            .try_map(allowance()?, |bytes, allowance| {
                let frame = wire::ExecutionFileFollowFrame {
                    payload: Some(wire::ExecutionFileFollowFramePayload::Chunk(
                        wire::ArtifactChunk {
                            offset,
                            data: bytes,
                        },
                    )),
                };
                admit(&frame, allowance)?;
                Ok(frame)
            })?)
    }

    fn terminal(
        &mut self,
        reference: Option<&FileReference>,
        phase: wire::ExecutionFilePhase,
    ) -> Result<QueryValue<wire::ExecutionFileFollowFrame>> {
        let length = self.reader.offset();
        let frame = self.metadata.try_project(allowance()?, |metadata, allowance| {
            let sealed = reference.map(|reference| {
                let identity = metadata.identity.as_ref().ok_or_else(|| refused("follow metadata has no original identity"))?;
                if phase != wire::ExecutionFilePhase::Finished || reference.length != length
                    || reference.metadata.id.as_str() != identity.file_id
                    || history(&reference.position.history) != identity.history.clone().ok_or_else(|| refused("follow identity has no history"))? {
                    return Err(refused("original sealed reference differs from followed identity or length"));
                }
                Ok(wire::ExecutionFileSeal {
                    identity: Some(identity.clone()), position: Some(checkpoint::StorageCursor { history: Some(history(&reference.position.history)), sequence: reference.position.sequence }),
                    length: reference.length, sha256: hex(&reference.sha256),
                })
            }).transpose()?;
            let frame = wire::ExecutionFileFollowFrame { payload: Some(wire::ExecutionFileFollowFramePayload::Terminal(wire::ExecutionFileFollowTerminal { phase, length, sealed })) };
            admit(&frame, allowance)?;
            Ok(frame)
        })?;
        self.finished = true;
        Ok(frame)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn metadata(
    staged: &StagedFile,
    execution: &TestExecutionId,
    path: &str,
    media: &str,
    phase: wire::ExecutionFilePhase,
    length: u64,
    offset: u64,
    allowance: NonZeroUsize,
) -> harmos::Result<wire::ExecutionFileFollowMetadata> {
    let required = std::mem::size_of::<wire::ExecutionFileFollowMetadata>()
        + staged.history.store.as_str().len()
        + staged.history.epoch.as_str().len()
        + staged.id.as_str().len()
        + execution.as_str().len()
        + path.len()
        + media.len();
    if required > allowance.get() {
        return Err(refused("follow metadata exceeds original admission"));
    }
    let value = wire::ExecutionFileFollowMetadata {
        identity: Some(wire::ExecutionFileIdentity {
            history: Some(history(&staged.history)),
            file_id: staged.id.as_str().to_owned(),
            execution_id: execution.as_str().to_owned(),
            relative_path: path.to_owned(),
        }),
        content_type: media.to_owned(),
        kind: kind(path, media),
        phase,
        available_length: length,
        offset,
    };
    admit(&value, allowance)?;
    Ok(value)
}
fn kind(path: &str, media: &str) -> wire::ExecutionFileKind {
    use crate::ExecutionFileContentKind as Kind;
    match Kind::from_metadata(path, Some(media)) {
        Kind::Text => wire::ExecutionFileKind::Text,
        Kind::Binary => wire::ExecutionFileKind::Binary,
        Kind::Media => wire::ExecutionFileKind::Media,
        Kind::Unknown => wire::ExecutionFileKind::Unknown,
    }
}
fn phase(value: FilePhase) -> wire::ExecutionFilePhase {
    match value {
        FilePhase::Producing => wire::ExecutionFilePhase::Producing,
        FilePhase::Finished => wire::ExecutionFilePhase::Finished,
        FilePhase::Abandoned => wire::ExecutionFilePhase::Abandoned,
    }
}
fn history(value: &harmos::storage::StoreHistory) -> checkpoint::StoreHistory {
    checkpoint::StoreHistory {
        store: value.store.as_str().to_owned(),
        epoch: value.epoch.as_str().to_owned(),
    }
}
fn hex(value: &[u8; 32]) -> String {
    const ALPHABET: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(64);
    for byte in value {
        text.push(ALPHABET[(byte >> 4) as usize] as char);
        text.push(ALPHABET[(byte & 15) as usize] as char);
    }
    text
}
fn allowance() -> Result<NonZeroUsize> {
    NonZeroUsize::new(FRAME_MEMORY).ok_or_else(|| invalid("empty follow frame allowance"))
}
fn admit(value: &impl MemorySize, allowance: NonZeroUsize) -> harmos::Result<()> {
    if value.memory_size() > allowance.get() {
        return Err(refused("follow frame exceeds original storage admission"));
    }
    Ok(())
}
fn refused(reason: &'static str) -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "execution file follow",
        reason,
    }
}
fn invalid(reason: &'static str) -> Error {
    refused(reason).into()
}

/* === Tests === */
