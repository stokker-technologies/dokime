/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// List Scoped Records at an Issued Durable Position

use harmos_protocol::v1 as h;
use dokime_protocol::v1::evidence::{RecordsListRequest, RecordsWatchRequest, RecordsWatchResponse};
use crate::{Result, client::{Context, Output}};

pub const ID: &str = "dokime.v1.evidence.Records/Watch";

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<RecordsWatchRequest, RecordsWatchResponse>(ID, settings.watch_timeout)
}

pub async fn handle(context: &Context, request: RecordsWatchRequest) -> Result<Output<RecordsWatchResponse>> {
    let mut changes = context.host.store().subscribe();
    let mut selection = RecordsListRequest {
        scope: request.scope, record_type: request.record_type, versions: request.versions,
        between: request.between, step: request.step, selection: request.selection,
        at: request.at, page: request.page,
    };
    let first = super::evidence_records_list::read_page(context, selection.clone(), None)?;
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
                let frame = RecordsWatchResponse { records: page.records, schema: page.schema, resolved_at: cut.clone(), opening, resync_required: None };
                if sender.send(Ok(frame)).await.is_err() { return Ok::<(), crate::Error>(()); }
                if !token.is_empty() {
                    selection.at = cut;
                    selection.page.get_or_insert_default().page_token = token;
                } else {
                    after = cut;
                    opening = false;
                    selection.at = None;
                    selection.page.get_or_insert_default().page_token.clear();
                    tokio::select! {
                        _ = sender.closed() => return Ok(()),
                        changed = changes.changed() => if changed.is_err() { return Ok(()); }
                    }
                    if context.host.shutting_down() { return Ok(()); }
                }
                match super::evidence_records_list::read_page(&context, selection.clone(), after.as_ref()) {
                    Ok(next) => page = next,
                    Err(error) => {
                        let _ = sender.send(Ok(RecordsWatchResponse {
                            resync_required: Some(h::Failure { code: "resync_required".into(), message: format!("Watch Records: {error}"), ..Default::default() }),
                            ..Default::default()
                        })).await;
                        return Ok(());
                    }
                }
            }
        };
        match tokio::time::timeout(context.settings.watch_timeout, work).await {
            Ok(Err(error)) => { let _ = sender.send(Err(super::status(error))).await; }
            Err(_) => { let _ = sender.try_send(Err(tonic::Status::deadline_exceeded("Watch Records: Reopen After Its Deadline"))); }
            _ => ()
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}
