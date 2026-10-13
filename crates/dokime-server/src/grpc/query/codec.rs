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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::Protobuf;

// Local Imports
use super::super::value::{Budget, canonical};
use dokime_app::runtime::query::aggregate::AggregateOutcome;
use dokime_contract::operations::query as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn project(
    outcome: &AggregateOutcome,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<wire::AggregateReply> {
    let mut budget = Budget::new();
    let mut required = std::mem::size_of::<wire::AggregateReply>()
        .saturating_add(
            outcome
                .columns
                .len()
                .saturating_mul(4 * std::mem::size_of::<String>()),
        )
        .saturating_add(
            outcome
                .rows
                .len()
                .saturating_mul(4 * std::mem::size_of::<wire::QueryRow>()),
        );
    for column in &outcome.columns {
        super::super::value::text(column).map_err(failure)?;
        required = required.saturating_add(column.len());
    }
    for row in &outcome.rows {
        required =
            required.saturating_add(row.len().saturating_mul(
                4 * std::mem::size_of::<dokime_contract::operations::value::Value>(),
            ));
        for item in row {
            required =
                required.saturating_add(canonical::memory(item, &mut budget, 0).map_err(failure)?);
        }
    }
    if required > allowance.get() {
        return Err(exhausted());
    }
    let mut columns = canonical::slots(outcome.columns.len()).map_err(failure)?;
    for column in &outcome.columns {
        columns.push(column.clone());
    }
    let mut rows = canonical::slots(outcome.rows.len()).map_err(failure)?;
    let mut budget = Budget::new();
    for row in &outcome.rows {
        let mut values = canonical::slots(row.len()).map_err(failure)?;
        for item in row {
            values.push(canonical::encode(item, &mut budget, 0).map_err(failure)?);
        }
        rows.push(wire::QueryRow { values });
    }
    let result = wire::AggregateReply {
        columns,
        rows,
        complete: outcome.complete,
    };
    if result.encoded_len() > maximum {
        return Err(exhausted());
    }
    Ok(result)
}

/* === Private Functions === */
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical aggregate response",
    }
}
fn failure(error: tonic::Status) -> harmos::Error {
    if error.code() == tonic::Code::ResourceExhausted {
        return exhausted();
    }
    harmos::Error::backend(
        "project original aggregate",
        harmos::BackendFailure::Corrupt,
        error,
    )
}

/* === Tests === */
