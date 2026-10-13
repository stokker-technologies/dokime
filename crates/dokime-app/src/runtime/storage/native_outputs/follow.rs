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
use std::{
    num::NonZeroUsize,
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};
// Third Party Imports
use harmos::{
    MemorySize,
    storage::{
        FileFollower, FileMetadata, FilePhase, FileReference, QueryValue, SharedQueryValue,
        StagedFile, StorageFile, StorageProductionPreparation,
    },
};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
// Local Imports
use super::{
    COPY_BYTES, NativeOutput, copy_buffer, descriptor_memory, error, file_metadata, open_file,
};
use crate::{
    Result,
    runtime::storage::executions::{Acquisition, ExecutionStorage},
};
use dokime::{
    core::TestExecutionId,
    execution::saved::{ExecutionOutput, ExecutionOutputContent, SavedStreamState},
    storage::Manager,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Production {
    _metadata: QueryValue<FileMetadata>,
    pub(crate) staged: StagedFile,
    producer: StorageFile,
    length: u64,
    hash: Sha256,
    observed: Option<(u64, SystemTime)>,
    pub(crate) reference: Option<QueryValue<FileReference>>,
    pub(crate) interrupted: bool,
    failure: Option<&'static str>,
    _admission: StorageProductionPreparation,
}

#[derive(Clone)]
pub(crate) struct LiveOutput {
    owner: Arc<tokio::sync::Mutex<Acquisition>>,
    storage: Manager,
    path: SharedQueryValue<String>,
    pub(crate) identity: SharedQueryValue<StagedFile>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ExecutionStorage {
    pub(crate) async fn captured_output(
        &self,
        execution: &TestExecutionId,
        path: &str,
    ) -> Result<Option<(QueryValue<FileReference>, bool)>> {
        let Some(owner) = self.acquisition(execution).await else {
            return Ok(None);
        };
        let acquisition = owner.lock().await;
        let Some(output) = acquisition.native_outputs.get(path) else {
            return Ok(None);
        };
        if !output.captured {
            return Ok(None);
        }
        let descriptor = output
            .descriptor
            .as_ref()
            .ok_or_else(|| error("captured output has no original descriptor"))?;
        let maximum = NonZeroUsize::new(descriptor.file.memory_size())
            .ok_or_else(|| error("captured reference memory is empty"))?;
        let reference =
            descriptor.try_project(maximum, |descriptor, _| Ok(descriptor.file.clone()))?;
        Ok(Some((
            reference,
            descriptor.stream_state == Some(SavedStreamState::Interrupted),
        )))
    }

    pub(crate) async fn native_follower(
        self: &Arc<Self>,
        execution: &TestExecutionId,
        path: &str,
        offset: u64,
        expected: Option<&StagedFile>,
    ) -> Result<Option<(FileFollower, LiveOutput)>> {
        // Retain Accepted Admission And Writes If The Selecting Viewer Cancels.
        let bytes = std::mem::size_of::<TestExecutionId>()
            .checked_add(execution.as_str().len())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<String>() + path.len()))
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Option<StagedFile>>()))
            .and_then(|bytes| bytes.checked_add(expected.map_or(0, MemorySize::memory_size)))
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| error("native follow admission memory overflows"))?;
        let admission = self.storage.prepare_production(bytes)?;
        let storage = self.clone();
        let execution = execution.clone();
        let path = path.to_owned();
        let expected = expected.cloned();
        tokio::spawn(async move {
            let _admission = admission;
            storage
                .admit_native_follower(&execution, &path, offset, expected.as_ref())
                .await
        })
        .await
        .map_err(error)?
    }

    async fn admit_native_follower(
        self: &Arc<Self>,
        execution: &TestExecutionId,
        path: &str,
        offset: u64,
        expected: Option<&StagedFile>,
    ) -> Result<Option<(FileFollower, LiveOutput)>> {
        let Some(owner) = self.acquisition(execution).await else {
            return Ok(None);
        };
        let mut acquisition = owner.lock().await;
        let Some(output) = acquisition.native_outputs.get_mut(path) else {
            return Ok(None);
        };
        if output.captured && output.production.is_none() {
            return Err(error(
                "output capture completed during follow admission; refresh the output",
            ));
        }
        if output.production.is_none() && expected.is_some() {
            return Err(error(
                "resume identity has no registered original production",
            ));
        }
        if output.production.is_none() {
            let metadata = file_metadata(&self.storage, execution, path, None, None)?;
            let bytes = NonZeroUsize::new(metadata.memory_size())
                .ok_or_else(|| error("file production memory overflows"))?;
            let metadata_admission = self.storage.prepare_production(bytes)?;
            let producer = self.storage.produce_file((*metadata).clone()).await?;
            let session = producer
                .session()
                .ok_or_else(|| error("original production has no resumable session"))?;
            let bytes = session
                .memory_size()
                .checked_add(std::mem::size_of::<Production>())
                .and_then(NonZeroUsize::new)
                .ok_or_else(|| error("staged identity memory overflows"))?;
            let admission = self.storage.prepare_production(bytes)?;
            let staged = StagedFile {
                history: session.history.clone(),
                id: session.id.clone(),
            };
            drop(metadata_admission);
            output.production = Some(Production {
                _metadata: metadata,
                staged,
                producer,
                length: 0,
                hash: Sha256::new(),
                observed: None,
                reference: None,
                interrupted: false,
                failure: None,
                _admission: admission,
            });
        }
        if expected.is_some_and(|identity| {
            output
                .production
                .as_ref()
                .is_none_or(|production| &production.staged != identity)
        }) {
            return Err(error(
                "resume identity differs from registered output production",
            ));
        }
        // Admission Must Catch Up To The Requested Committed Offset Before Opening.
        loop {
            let stable = synchronize(&self.storage, output, false).await?;
            let production = output
                .production
                .as_ref()
                .expect("registered original production");
            if production.length >= offset {
                break;
            }
            let visible = output.visible_length(
                open_file(output)
                    .await?
                    .metadata()
                    .await
                    .map_err(error)?
                    .len(),
            )?;
            if production.length >= visible {
                return Err(error("resume offset exceeds retained output"));
            }
            if !stable {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
        let production = output
            .production
            .as_ref()
            .expect("registered original production");
        let bytes = NonZeroUsize::new(production.staged.memory_size())
            .ok_or_else(|| error("file identity memory is empty"))?;
        let identity = self
            .storage
            .produce(bytes, |_| Ok(production.staged.clone()))?
            .try_into_value()?
            .try_into_shared()?;
        let follower = self.storage.follow_file(&production.staged, offset).await?;
        let allowance = NonZeroUsize::new(std::mem::size_of::<String>() + path.len())
            .ok_or_else(|| error("output path memory overflows"))?;
        let path = self
            .storage
            .produce(allowance, |_| Ok(path.to_owned()))?
            .try_into_value()?
            .try_into_shared()?;
        drop(acquisition);
        Ok(Some((
            follower,
            LiveOutput {
                owner,
                storage: self.storage.clone(),
                path,
                identity,
            },
        )))
    }
}

impl LiveOutput {
    pub(crate) fn file_id(&self) -> &str {
        self.identity.id.as_str()
    }

    pub(crate) async fn caught_up(&self) -> Result<bool> {
        let acquisition = self.owner.lock().await;
        let output = acquisition
            .native_outputs
            .get(self.path.as_str())
            .ok_or_else(|| error("followed output registration disappeared"))?;
        let production = output
            .production
            .as_ref()
            .ok_or_else(|| error("followed output was remapped"))?;
        if production.staged != *self.identity {
            return Err(error("followed output was replaced"));
        }
        if production.reference.is_some() {
            return Ok(true);
        }
        let physical = open_file(output)
            .await?
            .metadata()
            .await
            .map_err(error)?
            .len();
        Ok(production.length == output.visible_length(physical)?)
    }

    pub(crate) async fn synchronize(&self) -> Result<bool> {
        // An Accepted Native Write Settles In Its Original Owner If The Viewer Cancels.
        let admission = self.storage.prepare_production(
            NonZeroUsize::new(
                std::mem::size_of::<Self>() + std::mem::size_of::<StorageProductionPreparation>(),
            )
            .unwrap(),
        )?;
        let owner = self.owner.clone();
        let path = self.path.clone();
        let identity = self.identity.clone();
        let storage = self.storage.clone();
        tokio::spawn(async move {
            let _admission = admission;
            let mut acquisition = owner.lock().await;
            let output = acquisition
                .native_outputs
                .get_mut(path.as_str())
                .ok_or_else(|| error("followed output registration disappeared"))?;
            let production = output
                .production
                .as_ref()
                .ok_or_else(|| error("followed output was remapped"))?;
            if production.staged != *identity {
                return Err(error("followed output was replaced"));
            }
            synchronize(&storage, output, false).await
        })
        .await
        .map_err(error)?
    }

    pub(crate) async fn reference(&self) -> Result<Option<QueryValue<FileReference>>> {
        let acquisition = self.owner.lock().await;
        let output = acquisition
            .native_outputs
            .get(self.path.as_str())
            .ok_or_else(|| error("followed output registration disappeared"))?;
        let production = output
            .production
            .as_ref()
            .ok_or_else(|| error("followed output was remapped"))?;
        if production.staged != *self.identity {
            return Err(error("followed output was replaced"));
        }
        if production.interrupted {
            return Ok(None);
        }
        let Some(reference) = production.reference.as_ref() else {
            return Ok(None);
        };
        let bytes = NonZeroUsize::new(reference.memory_size())
            .ok_or_else(|| error("sealed file reference memory is empty"))?;
        Ok(Some(
            self.storage
                .produce(bytes, |_| Ok((**reference).clone()))?
                .try_into_value()?,
        ))
    }

    pub(crate) async fn interrupted(&self) -> Result<bool> {
        let acquisition = self.owner.lock().await;
        let output = acquisition
            .native_outputs
            .get(self.path.as_str())
            .ok_or_else(|| error("followed output registration disappeared"))?;
        let production = output
            .production
            .as_ref()
            .ok_or_else(|| error("followed output was remapped"))?;
        if production.staged != *self.identity {
            return Err(error("followed output was replaced"));
        }
        Ok(production.interrupted)
    }
}

/* === Public Functions === */
pub(super) async fn capture(
    storage: &Manager,
    path: &str,
    output: &mut NativeOutput,
    state: Option<SavedStreamState>,
) -> Result<()> {
    let started = Instant::now();
    let identity = output
        .production
        .as_ref()
        .expect("registered original production");
    tracing::debug!(file_id = %identity.staged.id, retained_bytes = identity.length, "native follow capture started");
    loop {
        let production = output
            .production
            .as_ref()
            .expect("registered original production");
        let visible = output.visible_length(
            open_file(output)
                .await?
                .metadata()
                .await
                .map_err(error)?
                .len(),
        )?;
        let finalizing = production.length == visible;
        // Validate The Complete Prefix Once When The Original Producer Is Caught Up.
        if !synchronize(storage, output, finalizing).await? {
            return Err(error("native output changed after producer quiescence"));
        }
        if finalizing {
            break;
        }
    }
    seal(storage, path, output, state).await?;
    let production = output
        .production
        .as_ref()
        .expect("registered original production");
    tracing::debug!(file_id = %production.staged.id, retained_bytes = production.length,
        elapsed_ms = started.elapsed().as_millis() as u64, "native follow capture settled");
    Ok(())
}

pub(super) async fn interrupt(
    storage: &Manager,
    path: &str,
    output: &mut NativeOutput,
) -> Result<()> {
    // Preserve Only The Original Already Admitted Prefix After Source Failure.
    seal(storage, path, output, Some(SavedStreamState::Interrupted)).await
}

async fn seal(
    storage: &Manager,
    path: &str,
    output: &mut NativeOutput,
    state: Option<SavedStreamState>,
) -> Result<()> {
    let production = output
        .production
        .as_mut()
        .expect("registered original production");
    let started = Instant::now();
    tracing::debug!(file_id = %production.staged.id, retained_bytes = production.length, "native follow seal started");
    let reference = production.producer.finish().await?;
    tracing::debug!(file_id = %production.staged.id, retained_bytes = reference.length,
        elapsed_ms = started.elapsed().as_millis() as u64, "native follow seal settled");
    if reference.length != production.length
        || reference.sha256 != <[u8; 32]>::from(production.hash.clone().finalize())
    {
        return Err(error(
            "original sealed file differs from retained production",
        ));
    }
    let bytes = NonZeroUsize::new(reference.memory_size())
        .ok_or_else(|| error("sealed reference memory is empty"))?;
    let reference = storage
        .produce(bytes, |_| Ok(reference))?
        .try_into_value()?;
    let descriptor = storage
        .produce(
            descriptor_memory(path, &reference, &output.created_at)?,
            |_| {
                Ok(ExecutionOutput {
                    relative_path: path.to_owned(),
                    created_at: output.created_at.clone(),
                    file: (*reference).clone(),
                    stream_state: state,
                    content: ExecutionOutputContent::File,
                })
            },
        )?
        .try_into_value()?;
    production.interrupted = state == Some(SavedStreamState::Interrupted);
    production.reference = Some(reference);
    output.descriptor = Some(descriptor);
    output.captured = true;
    Ok(())
}

/* === Private Functions === */
async fn synchronize(
    storage: &Manager,
    output: &mut NativeOutput,
    finalizing: bool,
) -> Result<bool> {
    if output
        .production
        .as_ref()
        .is_some_and(|production| production.reference.is_some())
    {
        return Ok(true);
    }
    let mut file = open_file(output).await?;
    let before = file.metadata().await.map_err(error)?;
    let length = output.visible_length(before.len())?;
    if length > super::MAX_EXECUTION_FILE_SNAPSHOT_BYTES as u64 {
        return Err(crate::Error::ExecutionFileTooLarge);
    }
    let stamp = (before.len(), before.modified().map_err(error)?);
    let production = output
        .production
        .as_mut()
        .ok_or_else(|| error("output has no original production"))?;
    if let Some(reason) = production.failure {
        return Err(error(reason));
    }
    if production.reference.is_some() {
        return Ok(true);
    }
    if finalizing && length != production.length {
        return Err(error(
            "native output length changed after producer quiescence",
        ));
    }
    if length < production.length {
        production.failure = Some("followed output was truncated");
        return Err(error("followed output was truncated"));
    }
    let _buffer = storage.prepare_production(
        NonZeroUsize::new(std::mem::size_of::<Vec<u8>>() + COPY_BYTES).unwrap(),
    )?;
    let mut buffer = copy_buffer()?;
    // External Native Writers May Rewrite An Inode. Validate The Retained Prefix
    // On Each Observed Change Before Admitting Any Additional Bytes.
    if (finalizing && length == production.length) || production.observed != Some(stamp) {
        let mut remaining = production.length;
        let mut hash = Sha256::new();
        while remaining > 0 {
            let count = remaining.min(COPY_BYTES as u64) as usize;
            file.read_exact(&mut buffer[..count]).await.map_err(error)?;
            hash.update(&buffer[..count]);
            remaining -= count as u64;
        }
        if hash.finalize() != production.hash.clone().finalize() {
            production.failure = Some("followed output retained prefix was rewritten");
            return Err(error("followed output retained prefix was rewritten"));
        }
    }
    file.seek(std::io::SeekFrom::Start(production.length))
        .await
        .map_err(error)?;
    let count = (length - production.length).min(COPY_BYTES as u64) as usize;
    if count > 0 {
        file.read_exact(&mut buffer[..count]).await.map_err(error)?;
    }
    let after = file.metadata().await.map_err(error)?;
    if (after.len(), after.modified().map_err(error)?) != stamp {
        if finalizing {
            return Err(error("native output changed after producer quiescence"));
        }
        // Discard Only Unadmitted Bytes; The Original Producer And Prefix Survive.
        tracing::trace!(file_id = %production.staged.id, retained_bytes = production.length,
            "native follow read deferred while source changes");
        return Ok(false);
    }
    if count > 0 {
        if let Err(failure) = production.producer.write_all(&buffer[..count]).await {
            production.failure =
                Some("original native production write did not settle; retained identity refused");
            return Err(failure.into());
        }
        production.hash.update(&buffer[..count]);
        production.length += count as u64;
    }
    production.observed = Some(stamp);
    // A Finished Reference Is Produced Only By The Quiescent Capture Owner.
    let progress = storage.file_progress(&production.staged).await?;
    if progress.phase == FilePhase::Abandoned {
        production.interrupted = true;
    }
    Ok(true)
}

/* === Tests === */
