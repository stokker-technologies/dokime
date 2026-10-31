/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Report Evidence From One Committed WAL Snapshot.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::storage::{StorageCursor, StoredRow};

// Local Imports
use super::{aggregates, previews};
use crate::storage::{Error, sqlite::State};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Request {
    pub(crate) execution: String,
    pub(crate) cycle: String,
    pub(crate) at: StorageCursor,
    pub(crate) current: bool,
    pub(crate) rows: usize,
    pub(crate) streams: Vec<String>,
    pub(crate) percentiles: std::collections::BTreeSet<(String, String, u8)>,
}

pub(crate) struct Evidence {
    pub(crate) through: StorageCursor,
    pub(crate) captured_at: dokime_domain::model::time::Timestamp,
    pub(crate) records: Vec<dokime_domain::model::records::evidence::RecordEntry>,
    pub(crate) samples: Vec<StoredRow>,
    pub(crate) fields: Vec<previews::FieldPreview>,
    pub(crate) exact: Vec<aggregates::NumericSummary>,
    pub(crate) plots: std::collections::BTreeMap<String, crate::plot::preview::index::Checkpoint>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(crate) async fn report_evidence(&self, requests: Vec<Request>) -> Result<Vec<Evidence>, Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        self.read(move |connection| {
            Ok((|| {
                // The First Read Pins The WAL Snapshot For Every Member And Every Index.
                let head = super::super::saved(connection)?;
                let head = StorageCursor {
                    history: head.history,
                    sequence: head.sequence,
                };
                let captured_at = crate::model::time::now_utc();
                let mut evidence = Vec::with_capacity(requests.len());
                for request in requests {
                    if !head.covers(&request.at)? {
                        return Err(Error::Invalid("report boundary is not durable"));
                    }
                    let through = if request.current { head.clone() } else { request.at };
                    let fields = previews::report_previews(connection, request.execution.clone(), through.clone())?;
                    let mut exact = aggregates::numeric_summaries(connection, request.execution.clone())?;
                    for (stream, field, rank) in request.percentiles {
                        for summary in exact
                            .iter_mut()
                            .filter(|summary| crate::plot::stream_matches_alias(&summary.stream, &stream) && summary.field == field)
                        {
                            let name = format!("p{rank}");
                            if summary.statistics.iter().any(|(key, _)| *key == name) {
                                continue;
                            }
                            if let Some(value) = aggregates::numeric_percentile(
                                connection,
                                request.execution.clone(),
                                summary.stream.clone(),
                                field.clone(),
                                rank,
                            )? {
                                summary.statistics.push((name, value));
                            }
                        }
                    }
                    let plots = crate::plot::preview::index::checkpoints(connection, &request.execution, through.sequence)?;
                    let records =
                        previews::report_record_rows(connection, request.execution.clone(), request.cycle, through.clone(), request.rows)?;
                    let mut samples = Vec::new();
                    for stream in request.streams {
                        samples.extend(previews::report_sample_rows(
                            connection,
                            &root,
                            request.execution.clone(),
                            stream,
                            through.clone(),
                            request.rows,
                        )?);
                    }
                    evidence.push(Evidence {
                        through,
                        captured_at: captured_at.clone(),
                        records,
                        samples,
                        fields,
                        exact,
                        plots,
                    });
                }
                Ok(evidence)
            })())
        })
        .await?
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
