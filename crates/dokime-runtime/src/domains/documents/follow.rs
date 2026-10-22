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
    services::Fault,
    services::Resident,
    services::Service,
    storage::{FileBatch, FileFollower, FilePhase, FileReference, StagedFile},
};

// Local Imports
use super::saved::{SavedOutputFile, SavedOutputs};
use crate::{
    RuntimeHandle,
    runtime::Error,
    runtime::Result,
    runtime::storage::native_outputs::follow::{LiveOutput, NativeAdmission, Settlement},
};
use dokime_domain::model::execution::saved::SavedStreamState;
use dokime_domain::model::execution::wire_checkpoint as checkpoint;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

const SOURCE_POLL: Duration = Duration::from_millis(250);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ExecutionFileFollower {
    reader: FileFollower,
    source: Source,
    metadata: dokime_domain::model::content::wire::ExecutionFileFollowMetadata,
    finished: bool,
}

struct NativeCapture {
    source: LiveOutput,
}

// The Served Position Of A Followed File: Its Phase, Known Length And Resume Offset.
struct Position {
    phase: dokime_domain::model::content::wire::ExecutionFilePhase,
    length: u64,
    offset: u64,
}

/* === Enums === */

enum Source {
    Live(LiveOutput),
    Saved(Box<SavedOutputFile>),
    Captured(Box<FileReference>, bool),
}

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn follow_execution_file(
        &self, execution: &TestExecutionId, path: &str, expected: Option<&dokime_domain::model::content::wire::ExecutionFileIdentity>,
        offset: u64,
    ) -> Result<Option<ExecutionFileFollower>> {
        dokime_domain::model::execution::saved::validate_relative_output_path(path).map_err(crate::storage::Error::from)?;
        if offset != 0 && expected.is_none() {
            return Err(invalid("resume offset requires original identity"));
        }
        let expected_staged = expected
            .map(|identity| {
                if identity.execution_id != execution.as_str() || identity.relative_path != path {
                    return Err(invalid("resume identity belongs to another registered output"));
                }
                let history = identity.history.as_ref().ok_or_else(|| invalid("resume identity has no original history"))?;
                Ok(StagedFile {
                    history: harmos::storage::StoreHistory {
                        store: Identifier::new(&history.store).map_err(|_| invalid("invalid original store identity"))?,
                        epoch: Identifier::new(&history.epoch).map_err(|_| invalid("invalid original store epoch"))?,
                    },
                    id: Identifier::new(&identity.file_id).map_err(|_| invalid("invalid original file identity"))?,
                })
            })
            .transpose()?;
        if let Some(saved) = SavedOutputs::open(self.storage(), execution, None).await? {
            let Some(file) = saved.file(path)? else {
                return Ok(None);
            };
            let reference = &file.descriptor().file;
            let staged = StagedFile { history: reference.position.history.clone(), id: reference.metadata.id.clone() };
            if expected_staged.as_ref().is_some_and(|expected| expected != &staged) {
                return Err(invalid("saved output replaced the selected identity"));
            }
            if offset > reference.length {
                return Err(invalid("resume offset exceeds original saved length"));
            }
            let reader = self.storage().follow_file(&staged, offset).await?;
            let output = file.descriptor();
            let phase = if output.stream_state == Some(SavedStreamState::Interrupted) {
                dokime_domain::model::content::wire::ExecutionFilePhase::Abandoned
            } else {
                dokime_domain::model::content::wire::ExecutionFilePhase::Finished
            };
            let metadata = metadata(
                &staged,
                execution,
                path,
                &output.file.metadata.media_type,
                Position { phase, length: output.file.length, offset },
            );
            return Ok(Some(ExecutionFileFollower { reader, source: Source::Saved(Box::new(file)), metadata, finished: false }));
        }
        let (reader, live, abandoned) =
            match self.execution_storage.native_follower(execution, path, offset, expected_staged.as_ref()).await? {
                None => return Ok(None),
                Some(NativeAdmission::Live { follower, output, abandoned }) => (follower, output, abandoned),
                Some(NativeAdmission::Captured { reference, interrupted }) => {
                    let staged = StagedFile { history: reference.position.history.clone(), id: reference.metadata.id.clone() };
                    if expected_staged.as_ref().is_some_and(|expected| expected != &staged) {
                        return Err(invalid("captured output replaced the selected identity"));
                    }
                    if offset > reference.length {
                        return Err(invalid("resume offset exceeds captured output length"));
                    }
                    let reader = self.storage().follow_file(&staged, offset).await?;
                    let phase = if interrupted {
                        dokime_domain::model::content::wire::ExecutionFilePhase::Abandoned
                    } else {
                        dokime_domain::model::content::wire::ExecutionFilePhase::Finished
                    };
                    let metadata = metadata(
                        &staged,
                        execution,
                        path,
                        &reference.metadata.media_type,
                        Position { phase, length: reference.length, offset },
                    );
                    return Ok(Some(ExecutionFileFollower {
                        reader,
                        source: Source::Captured(Box::new(reference), interrupted),
                        metadata,
                        finished: false,
                    }));
                }
            };
        self.start_execution_file_capture(execution, &live).await?;
        let progress = reader.progress();
        let media = crate::execution_runtime::outputs::media_type_for_path(path).unwrap_or("application/octet-stream");
        let metadata = metadata(
            &live.identity,
            execution,
            path,
            media,
            Position {
                // A Live Admission Precedes Capture; The Sealed Harmos Phase Cannot
                // Distinguish An Abandoned Production From A Finished One.
                phase: if abandoned {
                    dokime_domain::model::content::wire::ExecutionFilePhase::Abandoned
                } else {
                    dokime_domain::model::content::wire::ExecutionFilePhase::Producing
                },
                length: progress.length,
                offset,
            },
        );
        Ok(Some(ExecutionFileFollower { reader, source: Source::Live(live), metadata, finished: false }))
    }
}

impl RuntimeHandle {
    async fn start_execution_file_capture(&self, execution: &TestExecutionId, source: &LiveOutput) -> Result<()> {
        let runtime = self.clone();
        let source = source.clone();
        let name = format!("execution-log:{execution}:native-{}", source.file_id());
        tokio::spawn(async move {
            let mut stops = runtime.execution_storage.tail_stops.lock().await;
            if stops.contains_key(&name) {
                return Ok(());
            }
            let preparation = runtime.execution_storage.prepare_tail_stop(&name)?;
            runtime
                .operations
                .register_service(
                    harmos::services::ServiceDefinition { id: name.clone(), policy: harmos::services::POLICY },
                    move |_: Arc<()>| NativeCapture { source: source.clone() },
                )
                .map_err(Error::from)?;
            let lease = match runtime.operations.submit_resident(&name).await {
                Ok(lease) => lease,
                Err(error) => {
                    runtime.operations.unregister_resident(&name).await.map_err(Error::from)?;
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
    async fn run(&mut self, mut context: Service<crate::app::Dokime>) -> std::result::Result<(), Fault<Error>> {
        context.ready();
        loop {
            let stable = self.source.synchronize().await.map_err(Fault::Application)?;
            if context.is_stopping() {
                return Ok(());
            }
            if !stable || self.source.caught_up().await.map_err(Fault::Application)? {
                context.rest(SOURCE_POLL).await;
            }
        }
    }
    async fn cleanup(&mut self) -> Result<()> {
        while !self.source.caught_up().await? {
            if !self.source.synchronize().await? {
                tokio::time::sleep(SOURCE_POLL).await;
            }
        }
        Ok(())
    }
}

impl ExecutionFileFollower {
    pub fn metadata(&self) -> &dokime_domain::model::content::wire::ExecutionFileFollowMetadata {
        &self.metadata
    }

    pub fn metadata_frame(&self) -> dokime_domain::model::content::wire::ExecutionFileFollowFrame {
        dokime_domain::model::content::wire::ExecutionFileFollowFrame {
            payload: Some(dokime_domain::model::content::wire::ExecutionFileFollowFramePayload::Metadata(self.metadata.clone())),
        }
    }

    pub async fn next_frame(
        &mut self, maximum: NonZeroUsize,
    ) -> Result<Option<dokime_domain::model::content::wire::ExecutionFileFollowFrame>> {
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
                        result = self.reader.next_batch(maximum) => {
                            let chunk = result?;
                            if let Some(chunk) = chunk { return self.chunk(offset, chunk).map(Some); }
                        }
                        _ = tokio::time::sleep(SOURCE_POLL) => continue,
                    }
                } else if let Some(chunk) = self.reader.next_batch(maximum).await? {
                    return self.chunk(offset, chunk).map(Some);
                }
                return match live.settlement().await? {
                    Settlement::Abandoned => self.terminal(None, dokime_domain::model::content::wire::ExecutionFilePhase::Abandoned),
                    Settlement::Finished(reference) => {
                        self.terminal(Some(&reference), dokime_domain::model::content::wire::ExecutionFilePhase::Finished)
                    }
                    // A Dropped Original Producer Abandons The File Before Any Capture.
                    Settlement::Producing if self.reader.progress().phase == FilePhase::Abandoned => {
                        self.terminal(None, dokime_domain::model::content::wire::ExecutionFilePhase::Abandoned)
                    }
                    Settlement::Producing => Err(invalid("finished original production has no sealed reference")),
                }
                .map(Some);
            }
            if let Some(chunk) = self.reader.next_batch(maximum).await? {
                return self.chunk(offset, chunk).map(Some);
            }
            if let Source::Captured(reference, interrupted) = &self.source {
                let interrupted = *interrupted;
                let reference = (**reference).clone();
                let phase = if interrupted {
                    dokime_domain::model::content::wire::ExecutionFilePhase::Abandoned
                } else {
                    dokime_domain::model::content::wire::ExecutionFilePhase::Finished
                };
                return self.terminal((!interrupted).then_some(&reference), phase).map(Some);
            }
            let Source::Saved(file) = &mut self.source else { unreachable!() };
            let interrupted = file.descriptor().stream_state == Some(SavedStreamState::Interrupted);
            let reference = file.descriptor().file.clone();
            let phase = if interrupted {
                dokime_domain::model::content::wire::ExecutionFilePhase::Abandoned
            } else {
                dokime_domain::model::content::wire::ExecutionFilePhase::Finished
            };
            return self.terminal((!interrupted).then_some(&reference), phase).map(Some);
        }
    }

    fn chunk(&self, offset: u64, chunk: FileBatch) -> Result<dokime_domain::model::content::wire::ExecutionFileFollowFrame> {
        Ok(dokime_domain::model::content::wire::ExecutionFileFollowFrame {
            payload: Some(dokime_domain::model::content::wire::ExecutionFileFollowFramePayload::Chunk(
                dokime_domain::model::content::wire::ArtifactChunk { offset, data: chunk.bytes },
            )),
        })
    }

    fn terminal(
        &mut self, reference: Option<&FileReference>, phase: dokime_domain::model::content::wire::ExecutionFilePhase,
    ) -> Result<dokime_domain::model::content::wire::ExecutionFileFollowFrame> {
        let length = self.reader.offset();
        let metadata = &self.metadata;
        let frame = {
            let sealed = reference
                .map(|reference| {
                    let identity = metadata.identity.as_ref().ok_or_else(|| refused("follow metadata has no original identity"))?;
                    if phase != dokime_domain::model::content::wire::ExecutionFilePhase::Finished
                        || reference.length != length
                        || reference.metadata.id.as_str() != identity.file_id
                        || history(&reference.position.history)
                            != identity.history.clone().ok_or_else(|| refused("follow identity has no history"))?
                    {
                        return Err(refused("original sealed reference differs from followed identity or length"));
                    }
                    Ok(dokime_domain::model::content::wire::ExecutionFileSeal {
                        identity: Some(identity.clone()),
                        position: Some(checkpoint::StorageCursor {
                            history: Some(history(&reference.position.history)),
                            sequence: reference.position.sequence,
                        }),
                        length: reference.length,
                        sha256: hex(&reference.sha256),
                    })
                })
                .transpose()?;
            dokime_domain::model::content::wire::ExecutionFileFollowFrame {
                payload: Some(dokime_domain::model::content::wire::ExecutionFileFollowFramePayload::Terminal(
                    dokime_domain::model::content::wire::ExecutionFileFollowTerminal { phase, length, sealed },
                )),
            }
        };
        self.finished = true;
        Ok(frame)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn metadata(
    staged: &StagedFile, execution: &TestExecutionId, path: &str, media: &str, position: Position,
) -> dokime_domain::model::content::wire::ExecutionFileFollowMetadata {
    let Position { phase, length, offset } = position;
    dokime_domain::model::content::wire::ExecutionFileFollowMetadata {
        identity: Some(dokime_domain::model::content::wire::ExecutionFileIdentity {
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
    }
}

fn kind(path: &str, media: &str) -> dokime_domain::model::content::wire::ExecutionFileKind {
    use crate::ExecutionFileContentKind as Kind;
    match Kind::from_metadata(path, Some(media)) {
        Kind::Text => dokime_domain::model::content::wire::ExecutionFileKind::Text,
        Kind::Binary => dokime_domain::model::content::wire::ExecutionFileKind::Binary,
        Kind::Media => dokime_domain::model::content::wire::ExecutionFileKind::Media,
        Kind::Unknown => dokime_domain::model::content::wire::ExecutionFileKind::Unknown,
    }
}

fn history(value: &harmos::storage::StoreHistory) -> checkpoint::StoreHistory {
    checkpoint::StoreHistory { store: value.store.as_str().to_owned(), epoch: value.epoch.as_str().to_owned() }
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

fn refused(reason: &'static str) -> harmos::Error {
    harmos::Error::InvalidInput { field: "execution file follow", reason }
}

fn invalid(reason: &'static str) -> Error {
    refused(reason).into()
}

/* === Tests === */
