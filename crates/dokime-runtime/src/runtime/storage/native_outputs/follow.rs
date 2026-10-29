/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{sync::Arc, time::Instant};

use harmos::storage::{ByteRange, CaptureConsistency, FilePhase, FileReference, OpenedCapture, ProductionFollower, ReadBatch, StagedFile};

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

pub(super) struct Production {
    pub(crate) staged: StagedFile,
    capture: OpenedCapture,
    pub(crate) reference: Option<FileReference>,
    pub(crate) interrupted: bool,
}

#[derive(Clone)]
pub(crate) struct LiveOutput {
    owner: Arc<tokio::sync::Mutex<Acquisition>>,
    session: Manager,
    path: Arc<String>,
    pub(crate) identity: Arc<StagedFile>,
}

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
        let Some(output) = acquisition.native_outputs.get_mut(path) else {
            return Ok(None);
        };
        // Decide Captured Or Live Under The Owner Lock That Capture Settles Under.
        if output.captured {
            let descriptor = output.descriptor.as_ref().ok_or_else(|| error("captured output has no original descriptor"))?;
            let reference = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, crate::runtime::Error>(descriptor.file.clone()))?;
            let interrupted = descriptor.stream_state == Some(SavedStreamState::Interrupted);
            return Ok(Some(NativeAdmission::Captured { reference, interrupted }));
        }
        if output.production.is_none() && expected.is_some() {
            return Err(error("resume identity has no registered original production"));
        }
        if output.production.is_none() {
            let metadata = file_metadata(execution, path, None, None)?;
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
                capture,
                reference: None,
                interrupted: false,
            });
        }
        if expected.is_some_and(|identity| output.production.as_ref().is_none_or(|production| &production.staged != identity)) {
            return Err(error("resume identity differs from registered output production"));
        }
        // Admission Must Catch Up To The Requested Committed Offset Before Opening.
        loop {
            let stable = synchronize(session, output).await?;
            let production = output.production.as_ref().expect("registered original production");
            if production.capture.acknowledged() >= offset {
                break;
            }
            let visible = output.visible_length(open_file(output).await?.metadata().await.map_err(error)?.len())?;
            if production.capture.acknowledged() >= visible {
                return Err(error("resume offset exceeds retained output"));
            }
            if !stable {
                tokio::time::sleep(self.storage.settings().filesystem.poll_interval).await;
            }
        }
        let production = output.production.as_ref().expect("registered original production");
        let identity = Arc::new(production.staged.clone());
        let follower = session.follow_production(&production.staged, offset).await?;
        let abandoned = production.interrupted;
        let path = Arc::new(path.to_owned());
        drop(acquisition);
        Ok(Some(NativeAdmission::Live(Box::new((
            follower,
            LiveOutput {
                owner,
                session: session.clone(),
                path,
                identity,
            },
            abandoned,
        )))))
    }
}

impl LiveOutput {
    pub(crate) fn file_id(&self) -> &str {
        self.identity.id.as_str()
    }

    pub(crate) async fn caught_up(&self) -> Result<bool> {
        let acquisition = self.owner.lock().await;
        let output = acquisition.native_outputs.get(self.path.as_str()).ok_or_else(|| error("followed output registration disappeared"))?;
        let production = output.production.as_ref().ok_or_else(|| error("followed output was remapped"))?;
        if production.staged != *self.identity {
            return Err(error("followed output was replaced"));
        }
        if production.reference.is_some() {
            return Ok(true);
        }
        let physical = open_file(output).await?.metadata().await.map_err(error)?.len();
        Ok(production.capture.acknowledged() == output.visible_length(physical)?)
    }

    pub(crate) async fn synchronize(&self) -> Result<bool> {
        // An Accepted Native Write Settles In Its Original Owner If The Viewer Cancels.
        let owner = self.owner.clone();
        let path = self.path.clone();
        let identity = self.identity.clone();
        let session = self.session.clone();
        tokio::spawn(async move {
            let mut acquisition = owner.lock().await;
            let output =
                acquisition.native_outputs.get_mut(path.as_str()).ok_or_else(|| error("followed output registration disappeared"))?;
            let production = output.production.as_ref().ok_or_else(|| error("followed output was remapped"))?;
            if production.staged != *identity {
                return Err(error("followed output was replaced"));
            }
            synchronize(&session, output).await
        })
        .await
        .map_err(error)?
    }

    pub(crate) async fn settlement(&self) -> Result<Settlement> {
        // Read Abandonment And The Seal Together; Capture Sets Both Under This Lock.
        let acquisition = self.owner.lock().await;
        let output = acquisition.native_outputs.get(self.path.as_str()).ok_or_else(|| error("followed output registration disappeared"))?;
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

pub(super) async fn capture(path: &str, output: &mut NativeOutput, state: Option<SavedStreamState>) -> Result<()> {
    let started = Instant::now();
    let identity = output.production.as_ref().expect("registered original production");
    tracing::debug!(file_id = %identity.staged.id, retained_bytes = identity.capture.acknowledged(), "native follow capture started");
    let visible = output.visible_length(open_file(output).await?.metadata().await.map_err(error)?.len())?;
    seal(path, output, state, Some(visible)).await?;
    let production = output.production.as_ref().expect("registered original production");
    tracing::debug!(file_id = %production.staged.id, retained_bytes = production.capture.acknowledged(),
        elapsed_ms = started.elapsed().as_millis() as u64, "native follow capture settled");
    Ok(())
}

pub(super) async fn interrupt(path: &str, output: &mut NativeOutput) -> Result<()> {
    // Preserve Only The Original Already Admitted Prefix After Source Failure.
    seal(path, output, Some(SavedStreamState::Interrupted), None).await
}

async fn seal(path: &str, output: &mut NativeOutput, state: Option<SavedStreamState>, visible: Option<u64>) -> Result<()> {
    let production = output.production.as_mut().expect("registered original production");
    let started = Instant::now();
    tracing::debug!(file_id = %production.staged.id, retained_bytes = production.capture.acknowledged(), "native follow seal started");
    let reference = match visible {
        Some(length) => production.capture.finish_at(length).await.map_err(capture_error)?,
        None => production.capture.interrupt().await.map_err(capture_error)?,
    };
    tracing::debug!(file_id = %production.staged.id, retained_bytes = reference.length,
        elapsed_ms = started.elapsed().as_millis() as u64, "native follow seal settled");
    if reference.length != production.capture.acknowledged() {
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
        file: reference.clone(),
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

async fn synchronize(session: &Manager, output: &mut NativeOutput) -> Result<bool> {
    if output.production.as_ref().is_some_and(|production| production.reference.is_some()) {
        return Ok(true);
    }
    // Keep The Product's Original-Path Access And Visible-Log-Prefix Policy.
    // The Native Owner Retains The Opened Inode And Exact Accepted Copy State.
    let physical = open_file(output).await?.metadata().await.map_err(error)?.len();
    let visible = output.visible_length(physical)?;
    let production = output.production.as_mut().ok_or_else(|| error("output has no original production"))?;
    let stable = production.capture.synchronize(visible).await.map_err(capture_error)?;
    let progress = session.file_progress(&production.staged).await?;
    if progress.phase == FilePhase::Abandoned {
        production.interrupted = true;
    }
    Ok(stable)
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
