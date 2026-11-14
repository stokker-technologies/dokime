/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Records Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use crate::{
    Result,
    host::{Context, Output},
};
use dokime_protocol::host::evidence::RecordsListRequest;
use dokime_protocol::host::evidence::RecordsWatchRequest;
use dokime_protocol::host::evidence::RecordsWatchResponse;

// Local Imports

use dokime_protocol::host::evidence::record_scope::Kind as RecordScopeKind;

use crate::host::ClientSettings;
use harmos_protocol::model::Failure;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Records/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<RecordsWatchRequest, RecordsWatchResponse>(ID, settings.watch_timeout)
}

pub async fn handle(context: &Context, request: RecordsWatchRequest) -> Result<Output<RecordsWatchResponse>> {
    let mut changes = context.host.store().subscribe();
    let mut selection = RecordsListRequest {
        scope: request.scope,
        record_type: request.record_type,
        versions: request.versions,
        between: request.between,
        step: request.step,
        selection: request.selection,
        at: request.at,
        page: request.page,
    };
    let first = crate::host::evidence::records::list::read_page(context, selection.clone(), None)?;
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
                let frame = RecordsWatchResponse {
                    records: page.records,
                    schema: page.schema,
                    resolved_at: cut.clone(),
                    opening,
                    resync_required: None,
                };
                if sender.send(Ok(frame)).await.is_err() {
                    return Ok::<(), crate::Error>(());
                }
                if !token.is_empty() {
                    selection.at = cut;
                    selection.page.get_or_insert_default().page_token = token;
                } else {
                    let execution = selection
                        .selection
                        .as_ref()
                        .and_then(|selection| selection.scope.as_ref())
                        .and_then(|scope| scope.execution_id.as_deref())
                        .or_else(|| match selection.scope.as_ref().and_then(|scope| scope.kind.as_ref()) {
                            Some(RecordScopeKind::Execution(id)) => Some(id.as_str()),
                            _ => None,
                        });
                    if execution.is_some_and(|id| {
                        context.host.store().metadata(id).is_ok_and(|cut| cut.state != dokime_storage::ExecutionState::Open)
                    }) {
                        return Ok(());
                    }
                    after = cut;
                    opening = false;
                    selection.at = None;
                    selection.page.get_or_insert_default().page_token.clear();
                    tokio::select! {
                        _ = sender.closed() => return Ok(()),
                        changed = changes.changed() => if changed.is_err() { return Ok(()); }
                    }
                    if context.host.shutting_down() {
                        return Ok(());
                    }
                }
                match crate::host::evidence::records::list::read_page(&context, selection.clone(), after.as_ref()) {
                    Ok(next) => page = next,
                    Err(error) => {
                        let _ = sender
                            .send(Ok(RecordsWatchResponse {
                                resync_required: Some(Failure {
                                    code: "resync_required".into(),
                                    message: format!("Watch Records: {error}"),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }))
                            .await;
                        return Ok(());
                    }
                }
            }
        };
        match tokio::time::timeout(context.settings.watch_timeout, work).await {
            Ok(Err(error)) => {
                let _ = sender.send(Err(crate::host::status(error))).await;
            }
            Err(_) => {
                let _ = sender.try_send(Err(tonic::Status::deadline_exceeded("Watch Records: Reopen After Its Deadline")));
            }
            _ => (),
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}

/* === Private Functions === */

/* === Tests === */
