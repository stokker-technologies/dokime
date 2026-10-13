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
    Client, Error, ResponseStream, Result, TestExecutionHandle, contracts, invalid, poll_interval,
    validate_observed_cursor,
};
use dokime_contract::core::TestExecutionId;

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
    ) -> Result<ResponseStream<contracts::evidence::RecordPage>> {
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
    ) -> Result<ResponseStream<contracts::telemetry::TelemetrySamplesReply>> {
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
pub(crate) fn sample_interval(page_size: u32, interval: Option<Duration>) -> Result<Duration> {
    if !(1..=5000).contains(&page_size) {
        return Err(Error::Rpc(tonic::Status::invalid_argument(
            "page size must be between 1 and 5000",
        )));
    }
    poll_interval(interval)
}

pub(crate) async fn read_samples<F, Fut>(
    client: &Client,
    scope: contracts::common::EvidenceScope,
    streams: Vec<String>,
    page_size: u32,
    interval: Duration,
    mut completion: F,
    sender: &mpsc::Sender<Result<contracts::telemetry::TelemetrySamplesReply>>,
) -> Result<()>
where
    F: FnMut() -> Fut + Send,
    Fut: Future<Output = Result<bool>> + Send,
{
    let mut positions: HashMap<String, u64> = HashMap::new();
    let mut draining = false;
    loop {
        if !draining {
            draining = completion().await?;
        }
        client.verify(true).await?;
        let page = client
            .telemetry_list_samples(contracts::telemetry::ListTelemetrySamplesRequest {
                scope: Some(scope.clone()),
                streams: streams.clone(),
                limit: page_size,
                after_stream_cursors: positions
                    .iter()
                    .map(|(key, value)| {
                        contracts::telemetry::ListTelemetrySamplesRequestAfterStreamCursorsEntry {
                            key: key.clone(),
                            value: *value,
                        }
                    })
                    .collect(),
                latest: false,

                schema_versions: Vec::new(),
                sources: Vec::new(),
                signals: Vec::new(),
                start: None,
                end: None,
            })
            .await?;
        // Raw Telemetry Cursors Have No Epoch; Fence Both Sides Of The Read.
        client.verify(true).await?;
        if page.samples.len() > page_size as usize {
            return Err(invalid("telemetry page exceeds the requested limit"));
        }
        validate_samples(&scope, &streams, &positions, &page)?;
        positions.extend(
            page.through_stream_cursors
                .iter()
                .map(|entry| (entry.key.clone(), entry.value)),
        );
        let complete = page.complete;
        if sender.send(Ok(page)).await.is_err() || draining && complete {
            return Ok(());
        }
        if complete {
            tokio::time::sleep(interval).await;
        }
    }
}

pub(crate) async fn read_records<F>(
    client: &Client,
    scope: contracts::common::EvidenceScope,
    record_types: Vec<String>,
    completion: Option<F>,
    sender: &mpsc::Sender<Result<contracts::evidence::RecordPage>>,
) -> Result<()>
where
    F: Future<Output = Result<()>> + Send,
{
    let selection = contracts::evidence::CapturedRecordSelection {
        scope: Some(scope.clone()),
        record_types,
        sources: Vec::new(),
        severities: Vec::new(),
    };
    let mut after = None;
    if let Some(completion) = completion {
        let reader = client
            .evidence_watch_records(contracts::evidence::WatchRecordsRequest {
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
    let mut request = contracts::evidence::ListRecordsRequest {
        selection: Some(selection),
        after: None,
        through: None,
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
fn record_scope(id: &TestExecutionId) -> contracts::common::EvidenceScope {
    contracts::common::EvidenceScope {
        execution_id: Some(id.to_string()),

        session_id: None,
        cycle_id: None,
        case_id: None,
        step_id: None,
        criterion_id: None,
        template_id: None,
        suite_id: None,
        origin: None,
    }
}

fn validate_records(
    scope: &contracts::common::EvidenceScope,
    epoch: &str,
    page: &contracts::evidence::RecordPage,
    after: Option<&contracts::cursor::Cursor>,
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
    scope: &contracts::common::EvidenceScope,
    streams: &[String],
    positions: &HashMap<String, u64>,
    page: &contracts::telemetry::TelemetrySamplesReply,
) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    let mut advanced = false;
    for entry in &page.through_stream_cursors {
        if entry.key.is_empty()
            || !seen.insert(entry.key.as_str())
            || (!streams.is_empty() && !streams.contains(&entry.key))
        {
            return Err(invalid("telemetry cursor selection is invalid"));
        }
        match positions.get(&entry.key) {
            Some(previous) if entry.value < *previous => {
                return Err(invalid("telemetry continuation regressed"));
            }
            Some(previous) => advanced |= entry.value > *previous,
            None => advanced = true,
        }
    }
    if !page.complete && !advanced {
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
