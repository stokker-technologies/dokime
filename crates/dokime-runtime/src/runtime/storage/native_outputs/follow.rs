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
use std::{sync::Arc, time::Instant};

// Third Party Imports
use harmos::storage::{ByteRange, CaptureConsistency, FilePhase, FileReference, OpenedCapture, ProductionFollower, ReadBatch, StagedFile};

// Local Imports
use super::{NativeOutput, error, file_metadata, open_file};
use crate::storage::Manager;
use crate::{
    runtime::Result,
    runtime::storage::executions::{Acquisition, ExecutionStorage},
};
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionOutputContent;
use dokime_domain::model::execution::saved::SavedStreamState;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Production {
    pub(crate) staged: StagedFile,
    capture: Arc<tokio::sync::Mutex<OpenedCapture>>,
    pub(crate) reference: Option<FileReference>,
    pub(crate) interrupted: bool,
}

#[derive(Clone)]
pub(crate) struct LiveOutput {
    owner: Arc<tokio::sync::Mutex<Acquisition>>,
    session: Manager,
    path: Arc<String>,
    pub(crate) identity: Arc<StagedFile>,
    pub(crate) media_type: Arc<str>,
}

/* === Enums === */

// How A Follow Was Admitted, Decided Under The Original Owner's Lock.
pub(crate) enum NativeAdmission {
    // Capture Settled First; Its Descriptor Alone Decides Finished Or Abandoned.
    Captured { reference: ReadBatch<FileReference>, interrupted: bool },
    // The Original Production Was Still Retained When The Follower Opened.
    Live(Box<(ProductionFollower, LiveOutput, bool)>),
}

// The Dokime-Owned State Of A Followed Production. A Sealed Harmos File Is
// Finished Even When Its Production Was Abandoned, So Only This Decides.
pub(crate) enum Settlement {
    Producing,
    Abandoned,
    Finished(ReadBatch<FileReference>),
}

/* === Traits === */

/* === Implementations === */

impl ExecutionStorage {
    pub(crate) async fn native_follower(
        self: &Arc<Self>, session: &Manager, execution: &TestExecutionId, path: &str, offset: u64, expected: Option<&StagedFile>,
    ) -> Result<Option<NativeAdmission>> {
        // Retain Accepted Admission And Writes If The Selecting Viewer Cancels.
        let storage = self.clone();
        let selection = harmos::storage::ReadBatch::new(())
            .try_map(|_| Ok::<_, crate::runtime::Error>((execution.clone(), path.to_owned(), expected.cloned())))?;
        let session = session.clone();
        tokio::spawn(async move {
            let (execution, path, expected) = &*selection;
            storage.admit_native_follower(&session, execution, path, offset, expected.as_ref()).await
        })
        .await
        .map_err(error)?
    }

    async fn admit_native_follower(
        self: &Arc<Self>, session: &Manager, execution: &TestExecutionId, path: &str, offset: u64, expected: Option<&StagedFile>,
    ) -> Result<Option<NativeAdmission>> {
        let Some(owner) = self.acquisition(execution).await else {
            return Ok(None);
        };
        let mut acquisition = owner.lock().await;
        let Some(output) = acquisition.output_mut(path) else {
            return Ok(None);
        };
        // Decide Captured Or Live Under The Owner Lock That Capture Settles Under.
        if output.captured {
            let descriptor = output.descriptor.as_ref().ok_or_else(|| error("captured output has no original descriptor"))?;
            let reference = harmos::storage::ReadBatch::new(())
                .try_map(|_| Ok::<_, crate::runtime::Error>(descriptor.file().map_err(crate::storage::Error::from)?.clone()))?;
            let interrupted = descriptor.stream_state == Some(SavedStreamState::Interrupted);
            return Ok(Some(NativeAdmission::Captured { reference, interrupted }));
        }
        if output.production.is_none() && expected.is_some() {
            return Err(error("resume identity has no registered original production"));
        }
        if output.production.is_none() {
            let metadata = match &output.metadata {
                Some(metadata) => metadata.clone(),
                None => file_metadata(execution, path, None, None)?,
            };
            let producer = self.storage.produce_file(metadata).await?;
            let token = producer.session().ok_or_else(|| error("original production has no resumable session"))?;
            let staged = StagedFile {
                history: token.history.clone(),
                id: token.id.clone(),
            };
            let file = open_file(output).await?.into_std().await;
            let capture = session.capture_opened_file(file, producer, ByteRange { offset: 0, limit: None }, CaptureConsistency::Stable);
            output.production = Some(Production {
                staged,
                capture: Arc::new(tokio::sync::Mutex::new(capture)),
                reference: None,
                interrupted: false,
            });
        }
        if expected.is_some_and(|identity| output.production.as_ref().is_none_or(|production| &production.staged != identity)) {
            return Err(error("resume identity differs from registered output production"));
        }
        let production = output.production.as_ref().expect("registered original production");
        let identity = Arc::new(production.staged.clone());
        let abandoned = production.interrupted;
        let media_type = Arc::from(output.content_type(path).unwrap_or("application/octet-stream"));
        let path = Arc::new(path.to_owned());
        drop(acquisition);
        let live = LiveOutput {
            owner,
            session: session.clone(),
            path,
            identity,
            media_type,
        };
        // Viewer Budget Waits Never Hold The Producer's Acquisition Lock.
        loop {
            live.synchronize().await?;
            let progress = session.file_progress(&live.identity).await?;
            if progress.length >= offset {
                break;
            }
            if live.caught_up().await? {
                return Err(error("resume offset exceeds retained output"));
            }
            tokio::time::sleep(self.storage.settings().filesystem.poll_interval).await;
        }
        let follower = session.follow_production(&live.identity, offset).await?;
        Ok(Some(NativeAdmission::Live(Box::new((follower, live, abandoned)))))
    }
}

impl LiveOutput {
    pub(crate) fn file_id(&self) -> &str {
        self.identity.id.as_str()
    }

    pub(crate) async fn caught_up(&self) -> Result<bool> {
        let acquisition = self.owner.lock().await;
        let output = acquisition.output(self.path.as_str()).ok_or_else(|| error("followed output registration disappeared"))?;
        let production = output.production.as_ref().ok_or_else(|| error("followed output was remapped"))?;
        if production.staged != *self.identity {
            return Err(error("followed output was replaced"));
        }
        if production.reference.is_some() {
            return Ok(true);
        }
        let physical = open_file(output).await?.metadata().await.map_err(error)?.len();
        let visible = output.visible_length(physical)?;
        let capture = production.capture.clone();
        drop(acquisition);
        Ok(capture.lock().await.acknowledged() == visible)
    }

    pub(crate) async fn synchronize(&self) -> Result<bool> {
        // Keep Accepted Copy State If A Selecting Viewer Cancels.
        let live = self.clone();
        tokio::spawn(async move {
            let acquisition = live.owner.lock().await;
            let output = acquisition.output(live.path.as_str()).ok_or_else(|| error("followed output registration disappeared"))?;
            let production = output.production.as_ref().ok_or_else(|| error("followed output was remapped"))?;
            if production.staged != *live.identity {
                return Err(error("followed output was replaced"));
            }
            if production.reference.is_some() {
                return Ok(true);
            }
            let file = open_file(output).await?;
            let acknowledged = output.log.as_ref().map(|log| log.acknowledged);
            let capture = production.capture.clone();
            drop(acquisition);
            // Storage Admission May Wait For Read Capacity; Capture Appends Do Not.
            let mut capture = capture.lock().await;
            if live.session.file_progress(&live.identity).await?.phase == FilePhase::Finished {
                return Ok(true);
            }
            let physical = file.metadata().await.map_err(error)?.len();
            // A Later Follower May Have Captured A Newer Acknowledged Log Prefix.
            let visible = acknowledged.map_or(physical, |length| length.max(capture.acknowledged()));
            let stable = capture.synchronize(visible).await.map_err(capture_error)?;
            let abandoned = live.session.file_progress(&live.identity).await?.phase == FilePhase::Abandoned;
            drop(capture);
            if abandoned {
                let mut acquisition = live.owner.lock().await;
                let output = acquisition.output_mut(live.path.as_str()).ok_or_else(|| error("followed output registration disappeared"))?;
                output.production.as_mut().ok_or_else(|| error("followed output was remapped"))?.interrupted = true;
            }
            Ok(stable)
        })
        .await
        .map_err(error)?
    }

    pub(crate) async fn settlement(&self) -> Result<Settlement> {
        // Read Abandonment And The Seal Together; Capture Sets Both Under This Lock.
        let acquisition = self.owner.lock().await;
        let output = acquisition.output(self.path.as_str()).ok_or_else(|| error("followed output registration disappeared"))?;
        let production = output.production.as_ref().ok_or_else(|| error("followed output was remapped"))?;
        if production.staged != *self.identity {
            return Err(error("followed output was replaced"));
        }
        if production.interrupted {
            return Ok(Settlement::Abandoned);
        }
        let Some(reference) = production.reference.as_ref() else {
            return Ok(Settlement::Producing);
        };
        let reference = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, crate::runtime::Error>(reference.clone()))?;
        Ok(Settlement::Finished(reference))
    }
}

/* === Public Functions === */

pub(in crate::runtime::storage) async fn capture(path: &str, output: &mut NativeOutput, state: Option<SavedStreamState>) -> Result<()> {
    let started = Instant::now();
    let identity = output.production.as_ref().expect("registered original production");
    let retained = identity.capture.lock().await.acknowledged();
    tracing::debug!(file_id = %identity.staged.id, retained_bytes = retained, "native follow capture started");
    let visible = output.visible_length(open_file(output).await?.metadata().await.map_err(error)?.len())?;
    seal(path, output, state, Some(visible)).await?;
    let production = output.production.as_ref().expect("registered original production");
    let retained = production.capture.lock().await.acknowledged();
    tracing::debug!(file_id = %production.staged.id, retained_bytes = retained,
        elapsed_ms = started.elapsed().as_millis() as u64, "native follow capture settled");
    Ok(())
}

pub(in crate::runtime::storage) async fn interrupt(path: &str, output: &mut NativeOutput) -> Result<()> {
    // Preserve Only The Original Already Admitted Prefix After Source Failure.
    seal(path, output, Some(SavedStreamState::Interrupted), None).await
}

/* === Private Functions === */

async fn seal(path: &str, output: &mut NativeOutput, state: Option<SavedStreamState>, visible: Option<u64>) -> Result<()> {
    let production = output.production.as_mut().expect("registered original production");
    let started = Instant::now();
    let retained = production.capture.lock().await.acknowledged();
    tracing::debug!(file_id = %production.staged.id, retained_bytes = retained, "native follow seal started");
    let reference = match visible {
        Some(length) => production.capture.lock().await.finish_at(length).await.map_err(capture_error)?,
        None => production.capture.lock().await.interrupt().await.map_err(capture_error)?,
    };
    tracing::debug!(file_id = %production.staged.id, retained_bytes = reference.length,
        elapsed_ms = started.elapsed().as_millis() as u64, "native follow seal settled");
    if reference.length != production.capture.lock().await.acknowledged() {
        return Err(error("original sealed file differs from retained production"));
    }
    if let Some(length) = visible {
        let physical = open_file(output).await?.metadata().await.map_err(error)?.len();
        if output.visible_length(physical)? != length {
            return Err(error("native output changed after producer quiescence"));
        }
    }
    let descriptor = ExecutionOutput {
        relative_path: path.to_owned(),
        created_at: output.created_at.clone(),
        file: Some(reference.clone()),
        stream_state: state,
        content: ExecutionOutputContent::File,
    };
    let production = output.production.as_mut().expect("registered original production");
    production.interrupted = state == Some(SavedStreamState::Interrupted);
    production.reference = Some(reference);
    output.descriptor = Some(descriptor);
    output.captured = true;
    Ok(())
}

fn capture_error(failure: harmos::storage::Error) -> crate::runtime::Error {
    match failure {
        harmos::storage::Error::InvalidInput { reason, .. } => error(reason),
        source @ harmos::storage::Error::Backend {
            operation: "local filesystem" | "local filesystem task",
            ..
        } => error(source),
        other => other.into(),
    }
}

/* === Tests === */
