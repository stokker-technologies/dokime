/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::{TestExecutionHandle, invalid, validate_checkpoint};
use crate::{ResponseStream, Result, contracts};
use dokime_domain::model::execution::ExecutionCheckpoint;
use dokime_domain::model::execution::PageCursor;

const PAGE_ITEMS: u64 = 100;

struct Pages {
    checkpoint: ExecutionCheckpoint,
    position: u64,
    after: String,
}

impl TestExecutionHandle {
    pub async fn context_at(
        &self, checkpoint: &dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint,
    ) -> Result<dokime_domain::model::history::wire::CheckpointContext> {
        validate_checkpoint(&self.id, checkpoint)?;
        self.client.verify(false).await?;
        let context = self
            .client
            .history_get_checkpoint_context(contracts::history::GetCheckpointContext { checkpoint: Some(checkpoint.clone()) })
            .await?;
        same_checkpoint(checkpoint, context.checkpoint.as_ref())?;
        Ok(context)
    }

    pub async fn evidence_at(
        &self, checkpoint: &dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint,
    ) -> Result<dokime_domain::model::history::wire::CheckpointEvidence> {
        validate_checkpoint(&self.id, checkpoint)?;
        self.client.verify(false).await?;
        let evidence = self
            .client
            .history_get_checkpoint_evidence(contracts::history::GetCheckpointEvidence {
                checkpoint: Some(checkpoint.clone()),
                sections: vec![
                    dokime_domain::model::history::wire::EvidenceSection::Artifacts,
                    dokime_domain::model::history::wire::EvidenceSection::RecordSchemas,
                    dokime_domain::model::history::wire::EvidenceSection::TelemetryTables,
                ],
            })
            .await?;
        same_checkpoint(checkpoint, evidence.checkpoint.as_ref())?;
        Ok(evidence)
    }

    pub fn records_at(
        &self, checkpoint: dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint,
    ) -> Result<ResponseStream<dokime_domain::model::history::wire::CheckpointRecordPage>> {
        let original = validate_checkpoint(&self.id, &checkpoint)?;
        let handle = self.clone();
        ResponseStream::spawn(&self.client, move |sender| async move {
            let mut pages = Pages {
                checkpoint: original,
                position: 0,
                after: String::new(),
            };
            loop {
                let result = async {
                    handle.client.verify(false).await?;
                    let page = handle
                        .client
                        .history_list_checkpoint_records(contracts::history::ListCheckpointRecords {
                            checkpoint: Some(checkpoint.clone()),
                            limit: PAGE_ITEMS as u32,
                            after: pages.after.clone(),
                        })
                        .await?;
                    same_checkpoint(&checkpoint, page.checkpoint.as_ref())?;
                    records(&page, PAGE_ITEMS as usize)?;
                    let more = pages.advance("records", "", page.next_after.as_deref())?;
                    Ok((page, more))
                }
                .await;
                match result {
                    Ok((page, more)) => {
                        if sender.send(Ok(page)).await.is_err() || !more {
                            return;
                        }
                    }
                    Err(error) => {
                        let _ = sender.send(Err(error)).await;
                        return;
                    }
                }
            }
        })
    }

    pub fn samples_at(
        &self, checkpoint: dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint,
    ) -> Result<ResponseStream<dokime_domain::model::history::wire::CheckpointTelemetryPage>> {
        let original = validate_checkpoint(&self.id, &checkpoint)?;
        let handle = self.clone();
        ResponseStream::spawn(&self.client, move |sender| async move {
            let read = async {
                handle.client.verify(false).await?;
                let inventory = handle
                    .client
                    .history_get_checkpoint_evidence(contracts::history::GetCheckpointEvidence {
                        checkpoint: Some(checkpoint.clone()),
                        sections: vec![dokime_domain::model::history::wire::EvidenceSection::TelemetryTables],
                    })
                    .await?;
                same_checkpoint(&checkpoint, inventory.checkpoint.as_ref())?;
                let mut tables: Vec<_> = inventory.telemetry_tables.into_iter().map(|table| table.table).collect();
                // The Inventory Is Response-Bounded; No Visited-Page Set Is Retained.
                tables.sort();
                if tables.iter().any(|table| table.is_empty()) || tables.windows(2).any(|pair| pair[0] == pair[1]) {
                    return Err(invalid("saved telemetry inventory has empty or duplicate streams"));
                }
                for table in tables {
                    let mut pages = Pages {
                        checkpoint: original.clone(),
                        position: 0,
                        after: String::new(),
                    };
                    loop {
                        handle.client.verify(false).await?;
                        let page = handle
                            .client
                            .history_list_checkpoint_telemetry(contracts::history::ListCheckpointTelemetry {
                                checkpoint: Some(checkpoint.clone()),
                                table: table.clone(),
                                limit: PAGE_ITEMS as u32,
                                after: pages.after.clone(),
                            })
                            .await?;
                        same_checkpoint(&checkpoint, page.checkpoint.as_ref())?;
                        samples(&table, &page, PAGE_ITEMS as usize)?;
                        let more = pages.advance("telemetry", &table, page.next_after.as_deref())?;
                        if sender.send(Ok(page)).await.is_err() {
                            return Ok(());
                        }
                        if !more {
                            break;
                        }
                    }
                }
                Ok(())
            }
            .await;
            if let Err(error) = read {
                let _ = sender.send(Err(error)).await;
            }
        })
    }
}

impl Pages {
    fn advance(&mut self, kind: &str, source: &str, next: Option<&str>) -> Result<bool> {
        let Some(next) = next else {
            return Ok(false);
        };
        let cursor = PageCursor::open(&self.checkpoint, kind, source, Some(next))
            .map_err(|_| invalid("saved evidence continuation differs from its checkpoint or source"))?;
        if cursor.position() <= self.position {
            return Err(invalid("saved evidence continuation did not advance"));
        }
        self.position = cursor.position();
        self.after = next.to_owned();
        Ok(true)
    }
}

fn same_checkpoint(
    expected: &dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint,
    actual: Option<&dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint>,
) -> Result<()> {
    if actual != Some(expected) {
        return Err(invalid("history returned a different checkpoint"));
    }
    Ok(())
}

fn records(page: &dokime_domain::model::history::wire::CheckpointRecordPage, limit: usize) -> Result<()> {
    if !page.issues.is_empty() || page.issues_truncated {
        return Err(invalid("historical records are incomplete; history reported issues"));
    }
    if page.records.len() > limit || page.records.iter().any(|row| row.epoch.is_empty() || row.record.is_none()) {
        return Err(invalid("saved record page exceeds its bound or lacks source identity"));
    }
    Ok(())
}

fn samples(table: &str, page: &dokime_domain::model::history::wire::CheckpointTelemetryPage, limit: usize) -> Result<()> {
    if page.table != table {
        return Err(invalid("history returned a different telemetry stream"));
    }
    if !page.issues.is_empty() || page.issues_truncated {
        return Err(invalid("historical samples are incomplete; history reported issues"));
    }
    if page.rows.len() > limit
        || page.rows.iter().any(|row| row.epoch.is_empty() || row.sample.as_ref().is_none_or(|sample| sample.stream != table))
    {
        return Err(invalid("saved sample page exceeds its bound or lacks original stream identity"));
    }
    Ok(())
}
