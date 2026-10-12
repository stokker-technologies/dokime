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
use std::{collections::HashMap, future::Future, time::Duration};

// Third Party Imports
use tokio::sync::mpsc;

// Local Imports
use super::{
    Client, Error, ResponseStream, Result, TestExecutionHandle, invalid, poll_interval,
    validate_observed_cursor, wire,
};
use crate::core::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl TestExecutionHandle {
    pub fn records(
        &self,
        record_types: Vec<String>,
        follow: bool,
        interval: Option<Duration>,
    ) -> Result<ResponseStream<wire::RecordPage>> {
        let interval = poll_interval(interval)?;
        let handle = self.clone();
        ResponseStream::spawn(&self.client, move |sender| async move {
            let result = async {
                handle.snapshot().await?;
                let completion =
                    follow.then_some(async { handle.wait(None, Some(interval)).await.map(|_| ()) });
                read_records(
                    &handle.client,
                    record_scope(&handle.id),
                    record_types,
                    completion,
                    &sender,
                )
                .await
            }
            .await;
            if let Err(error) = result {
                let _ = sender.send(Err(error)).await;
            }
        })
    }

    pub fn samples(
        &self,
        streams: Vec<String>,
        follow: bool,
        page_size: u32,
        interval: Option<Duration>,
    ) -> Result<ResponseStream<wire::TelemetrySamplesReply>> {
        let interval = sample_interval(page_size, interval)?;
        let handle = self.clone();
        ResponseStream::spawn(&self.client, move |sender| async move {
            let result = async {
                handle.snapshot().await?;
                let completion = || async {
                    if !follow {
                        return Ok(true);
                    }
                    match handle.checkpoint().await {
                        Ok(_) => Ok(true),
                        Err(Error::Rpc(status)) if status.code() == tonic::Code::NotFound => {
                            Ok(false)
                        }
                        Err(error) => Err(error),
                    }
                };
                read_samples(
                    &handle.client,
                    record_scope(&handle.id),
                    streams,
                    page_size,
                    interval,
                    completion,
                    &sender,
                )
                .await
            }
            .await;
            if let Err(error) = result {
                let _ = sender.send(Err(error)).await;
            }
        })
    }
}

/* === Public Functions === */
pub(in crate::client) fn sample_interval(
    page_size: u32,
    interval: Option<Duration>,
) -> Result<Duration> {
    if !(1..=5000).contains(&page_size) {
        return Err(Error::Rpc(tonic::Status::invalid_argument(
            "page size must be between 1 and 5000",
        )));
    }
    poll_interval(interval)
}

pub(in crate::client) async fn read_samples<F, Fut>(
    client: &Client,
    scope: wire::EvidenceScope,
    streams: Vec<String>,
    page_size: u32,
    interval: Duration,
    mut completion: F,
    sender: &mpsc::Sender<Result<wire::TelemetrySamplesReply>>,
) -> Result<()>
where
    F: FnMut() -> Fut + Send,
    Fut: Future<Output = Result<bool>> + Send,
{
    let mut positions = HashMap::new();
    let mut draining = false;
    loop {
        if !draining {
            draining = completion().await?;
        }
        client.verify(true).await?;
        let page = client
            .telemetry_list_samples(wire::ListTelemetrySamplesRequest {
                scope: Some(scope.clone()),
                streams: streams.clone(),
                limit: page_size,
                after_stream_cursors: positions.clone(),
                latest: false,
                ..Default::default()
            })
            .await?;
        // Raw Telemetry Cursors Have No Epoch; Fence Both Sides Of The Read.
        client.verify(true).await?;
        if page.samples.len() > page_size as usize {
            return Err(invalid("telemetry page exceeds the requested limit"));
        }
        validate_samples(&scope, &streams, &positions, &page)?;
        positions.extend(page.through_stream_cursors.clone());
        let complete = page.complete;
        if sender.send(Ok(page)).await.is_err() || draining && complete {
            return Ok(());
        }
        if complete {
            tokio::time::sleep(interval).await;
        }
    }
}

pub(in crate::client) async fn read_records<F>(
    client: &Client,
    scope: wire::EvidenceScope,
    record_types: Vec<String>,
    completion: Option<F>,
    sender: &mpsc::Sender<Result<wire::RecordPage>>,
) -> Result<()>
where
    F: Future<Output = Result<()>> + Send,
{
    let selection = wire::CapturedRecordSelection {
        scope: Some(scope.clone()),
        record_types,
        ..Default::default()
    };
    let mut after = None;
    if let Some(completion) = completion {
        let reader = client
            .evidence_watch_records(wire::WatchRecordsRequest {
                selection: Some(selection.clone()),
                after: None,
            })
            .await?;
        tokio::pin!(completion);
        let result: Result<bool> = async {
            loop {
                let page = tokio::select! {
                    outcome = &mut completion => {
                        outcome?;
                        break;
                    }
                    page = reader.message() => page?
                        .ok_or_else(|| invalid("record reader ended before completion"))?,
                };
                validate_records(&scope, &client.backend().epoch, &page, after.as_ref())?;
                let scanned = page.scanned.clone();
                if sender.send(Ok(page)).await.is_err() {
                    return Ok(false);
                }
                after = scanned;
            }
            Ok(true)
        }
        .await;
        reader.close().await;
        if !result? {
            return Ok(());
        }
    }

    // Freeze The Final Live Frontier; Saved Storage Positions Are Separate.
    client.verify(true).await?;
    let mut request = wire::ListRecordsRequest {
        selection: Some(selection),
        ..Default::default()
    };
    let mut page = client.evidence_list_records(request.clone()).await?;
    validate_records(&scope, &client.backend().epoch, &page, None)?;
    let frontier = page.through.clone();
    if let Some(after) = after {
        request.after = Some(after);
        request.through = frontier.clone();
        page = client.evidence_list_records(request.clone()).await?;
    }
    loop {
        validate_records(
            &scope,
            &client.backend().epoch,
            &page,
            request.after.as_ref(),
        )?;
        if page.through != frontier {
            return Err(invalid("record replay changed its fixed frontier"));
        }
        let complete = page.replay_complete;
        request.after = page.scanned.clone();
        request.through = frontier.clone();
        if sender.send(Ok(page)).await.is_err() || complete {
            return Ok(());
        }
        page = client.evidence_list_records(request.clone()).await?;
    }
}

/* === Private Functions === */
fn record_scope(id: &TestExecutionId) -> wire::EvidenceScope {
    wire::EvidenceScope {
        execution_id: Some(id.to_string()),
        ..Default::default()
    }
}

fn validate_records(
    scope: &wire::EvidenceScope,
    epoch: &str,
    page: &wire::RecordPage,
    after: Option<&wire::Cursor>,
) -> Result<()> {
    validate_observed_cursor(epoch, page.scanned.as_ref())?;
    validate_observed_cursor(epoch, page.through.as_ref())?;
    let scanned = page.scanned.as_ref().expect("validated scanned cursor");
    let through = page.through.as_ref().expect("validated frontier cursor");
    if scanned.stream.is_empty()
        || scanned.stream != through.stream
        || scanned.position > through.position
        || page.retained_after != 0
        || page.replay_complete && scanned.position != through.position
    {
        return Err(invalid("record page has an invalid frontier"));
    }
    if let Some(after) = after
        && (after.stream != scanned.stream
            || scanned.position < after.position
            || !page.replay_complete && scanned.position == after.position)
    {
        return Err(invalid("record continuation did not advance"));
    }
    let mut previous = after.map(|cursor| cursor.position);
    for record in &page.records {
        if record.scope.as_ref().is_none_or(|actual| {
            scope
                .execution_id
                .as_ref()
                .is_some_and(|id| actual.execution_id.as_ref() != Some(id))
                || scope
                    .cycle_id
                    .as_ref()
                    .is_some_and(|id| actual.cycle_id.as_ref() != Some(id))
        }) || record.sequence > scanned.position
            || previous.is_some_and(|position| record.sequence <= position)
        {
            return Err(invalid("record identity or journal position differs"));
        }
        previous = Some(record.sequence);
    }
    Ok(())
}

fn validate_samples(
    scope: &wire::EvidenceScope,
    streams: &[String],
    positions: &HashMap<String, u64>,
    page: &wire::TelemetrySamplesReply,
) -> Result<()> {
    if positions.iter().any(|(stream, position)| {
        page.through_stream_cursors
            .get(stream)
            .is_some_and(|next| next < position)
    }) || !page.complete
        && !page
            .through_stream_cursors
            .iter()
            .any(|(stream, position)| {
                positions
                    .get(stream)
                    .is_none_or(|previous| position > previous)
            })
    {
        return Err(invalid("telemetry continuation did not advance"));
    }
    for sample in &page.samples {
        if sample.scope.as_ref().is_none_or(|actual| {
            scope
                .execution_id
                .as_ref()
                .is_some_and(|id| actual.execution_id.as_ref() != Some(id))
                || scope
                    .cycle_id
                    .as_ref()
                    .is_some_and(|id| actual.cycle_id.as_ref() != Some(id))
        }) || !streams.is_empty() && !streams.contains(&sample.stream)
        {
            return Err(invalid("sample identity or native cursor differs"));
        }
    }
    Ok(())
}

/* === Tests === */
