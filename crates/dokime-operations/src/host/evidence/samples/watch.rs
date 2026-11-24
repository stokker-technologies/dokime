/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Samples Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::Failure;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

// Local Imports
use crate::{
    Result,
    host::{Context, Output},
};
use dokime_protocol::host::evidence::SamplesListRequest;
use dokime_protocol::host::evidence::SamplesWatchRequest;
use dokime_protocol::host::evidence::SamplesWatchResponse;
use dokime_storage::ExecutionState;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Samples/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: SamplesWatchRequest) -> Result<Output<SamplesWatchResponse>> {
    let mut changes = context.host.store().subscribe();
    let mut runtime = context.host.changes();
    let mut selection = SamplesListRequest {
        scope: request.scope,
        streams: request.streams,
        versions: request.versions,
        fields: request.fields,
        sources: request.sources,
        signals: request.signals,
        between: request.between,
        bucket: request.bucket,
        latest: request.latest,
        at: request.at,
        page: request.page,
    };
    let first = crate::host::evidence::samples::list::read_page(context, selection.clone(), None)?;
    let context = context.clone();
    let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let work = async {
            let mut page = first;
            let mut opening = true;
            let mut after = None;
            loop {
                let cut = page.resolved_at.clone();
                let token = page.next_page_token;
                let frame = SamplesWatchResponse {
                    samples: page.samples,
                    schema: page.schema,
                    resolved_at: cut.clone(),
                    opening,
                    resync_required: None,
                };
                if sender.send(Ok(frame)).await.is_err() {
                    return;
                }
                if !token.is_empty() {
                    selection.at = cut;
                    selection.page.get_or_insert_default().page_token = token;
                } else {
                    let execution = selection.scope.as_ref().and_then(|scope| scope.execution_id.clone());
                    if execution
                        .as_deref()
                        .is_some_and(|id| context.host.store().metadata(id).is_ok_and(|cut| cut.state != ExecutionState::Open))
                    {
                        return;
                    }
                    // Every Selected Sample Through This Cut Was Sent; Later Passes Read Only Beyond It.
                    after = cut;
                    opening = false;
                    selection.at = None;
                    selection.page.get_or_insert_default().page_token.clear();
                    // The Host's Shutdown Ends the Watch Even While Storage Stays Unchanged.
                    loop {
                        tokio::select! {
                            _ = sender.closed() => return,
                            changed = changes.changed() => if changed.is_err() { return; },
                            changed = runtime.changed() => if changed.is_err() || context.host.shutting_down() { return; } else { continue },
                        }
                        if context.host.shutting_down() {
                            return;
                        }
                        // While the Open Execution Saved Nothing Past the Cut Sent, No Selected Sample Can Appear.
                        let sent = after.as_ref().and_then(|cut| cut.positions.first()).map(|position| position.sequence);
                        if !execution.as_deref().is_some_and(|id| {
                            context
                                .host
                                .store()
                                .frontier(id)
                                .is_ok_and(|(state, frontier)| state == ExecutionState::Open && Some(frontier) == sent)
                        }) {
                            break;
                        }
                    }
                }
                match crate::host::evidence::samples::list::read_page(&context, selection.clone(), after.as_ref()) {
                    Ok(next) => page = next,
                    Err(error) => {
                        let _ = sender
                            .send(Ok(SamplesWatchResponse {
                                resync_required: Some(Failure {
                                    code: "resync_required".into(),
                                    message: format!("Watch Samples: {error}"),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }))
                            .await;
                        return;
                    }
                }
            }
        };
        if tokio::time::timeout(context.settings.watch_timeout, work).await.is_err() {
            let _ = sender.try_send(Err(Status::deadline_exceeded("Watch Samples: Reopen After Its Deadline")));
        }
    });
    Ok(ReceiverStream::new(output))
}

/* === Private Functions === */

/* === Tests === */

crate::native::download!(SamplesWatchRequest, SamplesWatchResponse, "dokime.host.v1.evidence", "Samples", "Watch");
