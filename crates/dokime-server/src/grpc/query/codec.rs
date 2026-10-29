/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::super::value::{Budget, canonical};
use crate::grpc::transcode::Transcode;
use dokime::runtime::query::aggregate::AggregateOutcome;

pub(super) fn project(
    outcome: &AggregateOutcome, maximum: usize,
) -> dokime::harmos::Result<dokime::domain::model::query::wire::AggregateReply> {
    let mut columns = canonical::slots(outcome.columns.len()).map_err(failure)?;
    for column in &outcome.columns {
        columns.push(column.clone());
    }
    let mut rows = canonical::slots(outcome.rows.len()).map_err(failure)?;
    let mut budget = Budget::new().map_err(failure)?;
    for row in &outcome.rows {
        let mut values = canonical::slots(row.len()).map_err(failure)?;
        for item in row {
            values.push(canonical::encode(item, &mut budget, 0).map_err(failure)?);
        }
        rows.push(dokime::domain::model::query::wire::QueryRow { values });
    }
    let result = dokime::domain::model::query::wire::AggregateReply {
        columns,
        rows,
        complete: outcome.complete,
    };
    if result.wire_len()? > maximum {
        return Err(exhausted());
    }
    Ok(result)
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted { resource: "canonical aggregate response" }
}

fn failure(error: tonic::Status) -> dokime::harmos::Error {
    if error.code() == tonic::Code::ResourceExhausted {
        return exhausted();
    }
    dokime::harmos::Error::backend("project original aggregate", dokime::harmos::BackendFailure::Corrupt, error)
}
