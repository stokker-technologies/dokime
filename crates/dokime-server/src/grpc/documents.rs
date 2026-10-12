/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod wire;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{collections::VecDeque, pin::Pin, sync::Arc};
// Third Party Imports
use futures_util::{Stream, stream};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};
use tokio_util::sync::{CancellationToken, DropGuard};
use tonic::{Request, Response, Status};
// Local Imports
use super::value;
use dokime_app::{
    RuntimeHandle,
    core::{DocumentId, TestCycleId, TestExecutionId},
};
use dokime_protocol::v1::{self, documents_server::Documents};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
type WorkResult = Result<(VecDeque<v1::DocumentFrame>, OwnedSemaphorePermit), Status>;

/* === Structs === */
pub(super) struct Service {
    pub runtime: RuntimeHandle,
    pub stopping: watch::Receiver<bool>,
    pub max_message_bytes: usize,
    pub admission: Arc<Semaphore>,
}

struct Delivery {
    work: Option<tokio::task::JoinHandle<WorkResult>>,
    frames: VecDeque<v1::DocumentFrame>,
    permit: Option<OwnedSemaphorePermit>,
    stopping: watch::Receiver<bool>,
    _cancel: DropGuard,
    done: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl Documents for Service {
    type CaptureCurrentDocumentStream =
        Pin<Box<dyn Stream<Item = Result<v1::DocumentFrame, Status>> + Send>>;
    async fn capture_current_document(
        &self,
        request: Request<v1::CaptureCurrentDocumentRequest>,
    ) -> Result<Response<Self::CaptureCurrentDocumentStream>, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("backend is draining"));
        }
        let permit = self
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| Status::resource_exhausted("document resolver is busy"))?;
        let request = request.into_inner();
        let (cycle, execution) = scope(request.scope)?;
        value::text(&request.document_id)?;
        let id = DocumentId::from_identifier(&request.document_id)
            .map_err(|_| Status::invalid_argument("invalid document ID"))?;
        let prepared = self
            .runtime
            .prepare_current_document(&cycle, &execution, &id)
            .map_err(super::execution::status)?;
        let cancel = CancellationToken::new();
        let stop = cancel.clone();
        let handle = tokio::runtime::Handle::current();
        let max_bytes = self.max_message_bytes;
        // The permit belongs to this worker through acquisition/conversion even
        // if the requesting client disconnects before it starts or completes.
        let work = tokio::task::spawn_blocking(move || {
            let result = handle.block_on(async {
                tokio::select! {
                    biased;
                    _ = stop.cancelled() => Err(Status::cancelled("document request cancelled")),
                    result = prepared.capture() => result.map_err(super::execution::status),
                }
            })?;
            if stop.is_cancelled() {
                return Err(Status::cancelled("document request cancelled"));
            }
            let frames = wire::frames(result, max_bytes)?;
            Ok((frames, permit))
        });
        let state = Delivery {
            work: Some(work),
            frames: VecDeque::new(),
            permit: None,
            stopping: self.stopping.clone(),
            _cancel: cancel.drop_guard(),
            done: false,
        };
        let stream = stream::unfold(state, |mut state| async move {
            if state.done {
                return None;
            }
            if *state.stopping.borrow() {
                state.done = true;
                return Some((Err(Status::unavailable("backend is draining")), state));
            }
            if let Some(work) = state.work.take() {
                let result = tokio::select! {
                    biased;
                    _ = state.stopping.changed() => Err(Status::unavailable("backend is draining")),
                    result = work => result.map_err(|_| Status::internal("document worker failed")).and_then(|result| result),
                };
                match result {
                    Ok((frames, permit)) => {
                        state.frames = frames;
                        state.permit = Some(permit);
                    }
                    Err(error) => {
                        state.done = true;
                        return Some((Err(error), state));
                    }
                }
            }
            state.frames.pop_front().map(|frame| (Ok(frame), state))
        });
        Ok(Response::new(Box::pin(stream)))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn scope(scope: Option<v1::DocumentScope>) -> Result<(TestCycleId, TestExecutionId), Status> {
    let scope = scope.ok_or_else(|| Status::invalid_argument("document scope required"))?;
    let cycle = TestCycleId::from_uuid(scope.cycle_id)
        .map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
    let execution = TestExecutionId::from_uuid(scope.execution_id)
        .map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    Ok((cycle, execution))
}

/* === Tests === */
