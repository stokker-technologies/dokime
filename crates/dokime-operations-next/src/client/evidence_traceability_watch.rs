/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Watch Scoped Durable Traceability Changes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{
    Result,
    client::{Context, Output},
};
use dokime_protocol::v1::evidence::{TraceabilityGetRequest, TraceabilityWatchRequest, TraceabilityWatchResponse};

/* === Constants === */

pub const ID: &str = "dokime.v1.evidence.Traceability/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<TraceabilityWatchRequest, TraceabilityWatchResponse>(ID, settings.watch_timeout)
}

pub async fn handle(context: &Context, request: TraceabilityWatchRequest) -> Result<Output<TraceabilityWatchResponse>> {
    super::complete(ID, context.settings.watch_timeout, async {
        if request.page.as_ref().is_some_and(|page| !page.page_token.is_empty()) {
            return Err(crate::Error::Invalid("Watch Traceability: Open a Watch with a Read Cut and No Page Token".into()));
        }
        let mut changes = context.host.store().subscribe();
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer.saturating_add(1));
        let terminal =
            sender.clone().reserve_owned().await.map_err(|_| crate::Error::Invalid("Watch Traceability: Reconnect the Observer".into()))?;
        let context = context.clone();
        tokio::spawn(async move {
            let work = async {
                let mut opening = true;
                let mut previous = None;
                let pinned = request.at.is_some();
                let mut at = request.at;
                loop {
                    changes.borrow_and_update();
                    let mut page = request.page.clone().unwrap_or_default();
                    loop {
                        let result = super::evidence_traceability_get::read_page(
                            &context,
                            TraceabilityGetRequest {
                                scope: request.scope.clone(),
                                page: Some(page.clone()),
                                at: at.clone(),
                            },
                            previous.as_ref(),
                        );
                        let response = match result {
                            Ok(response) => response,
                            Err(error) => {
                                let _ = sender.send(Err(super::status(error))).await;
                                return;
                            }
                        };
                        page.page_token = response.next_page_token;
                        at = response.resolved_at.clone();
                        if sender
                            .send(Ok(TraceabilityWatchResponse {
                                snapshots: response.snapshots,
                                resolved_at: response.resolved_at,
                                opening,
                            }))
                            .await
                            .is_err()
                        {
                            return;
                        }
                        if page.page_token.is_empty() {
                            break;
                        }
                    }
                    previous = at.take();
                    let catch_up = opening && pinned;
                    opening = false;
                    if catch_up {
                        continue;
                    }
                    tokio::select! {
                        _ = sender.closed() => return,
                        changed = changes.changed() => if changed.is_err() {
                            let _ = sender.send(Err(tonic::Status::unavailable("Watch Traceability: Storage Closed; Resynchronize"))).await;
                            return;
                        },
                    }
                }
            };
            if tokio::time::timeout(context.settings.watch_timeout, work).await.is_err() {
                terminal.send(Err(tonic::Status::deadline_exceeded("Watch Traceability: Operation Deadline Expired")));
            }
        });
        Ok(tokio_stream::wrappers::ReceiverStream::new(output))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
