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
use prost::Message;

// Local Imports
use super::super::value::{Budget, projection};
use dokime_app::runtime::query::aggregate::AggregateOutcome;
use dokime_protocol::v1;

/* === Constants === */
include!(concat!(env!("OUT_DIR"), "/aggregate-output-fields.rs"));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn encode(
    outcome: &AggregateOutcome,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    let count = count(outcome)?;
    let required = count
        .memory
        .checked_add(std::mem::size_of::<Vec<u8>>())
        .and_then(|bytes| bytes.checked_add(count.encoded))
        .ok_or_else(exhausted)?;
    if required > allowance.get() || count.encoded > maximum {
        return Err(exhausted());
    }

    let mut columns = projection::slots(outcome.columns.len())?;
    for column in &outcome.columns {
        columns.push(projection::text(column)?);
    }
    let mut budget = Budget::new();
    let mut rows = projection::slots(outcome.rows.len())?;
    for row in &outcome.rows {
        let mut values = projection::slots(row.len())?;
        for value in row {
            values.push(projection::project(value, &mut budget, 0)?);
        }
        rows.push(v1::QueryRow { values });
    }
    let wire = v1::AggregateReply {
        columns,
        rows,
        complete: outcome.complete,
    };
    if wire.encoded_len() != count.encoded {
        return Err(invalid());
    }
    let mut target = projection::slots(count.encoded)?;
    wire.encode(&mut target).map_err(|_| invalid())?;
    if target.len() != count.encoded {
        return Err(invalid());
    }
    Ok(target)
}

/* === Private Functions === */
fn count(outcome: &AggregateOutcome) -> harmos::Result<projection::Count> {
    let mut memory = std::mem::size_of::<v1::AggregateReply>()
        .checked_add(
            outcome
                .columns
                .len()
                .checked_mul(std::mem::size_of::<String>())
                .ok_or_else(exhausted)?,
        )
        .and_then(|bytes| {
            bytes.checked_add(
                outcome
                    .rows
                    .len()
                    .checked_mul(std::mem::size_of::<v1::QueryRow>())?,
            )
        })
        .ok_or_else(exhausted)?;
    let mut encoded = v1::AggregateReply {
        complete: outcome.complete,
        ..Default::default()
    }
    .encoded_len();
    for column in &outcome.columns {
        super::super::value::text(column).map_err(|_| exhausted())?;
        memory = memory.checked_add(column.len()).ok_or_else(exhausted)?;
        encoded = encoded
            .checked_add(prost::encoding::string::encoded_len(COLUMNS, column))
            .ok_or_else(exhausted)?;
    }
    let mut budget = Budget::new();
    for row in &outcome.rows {
        let mut row_length = 0usize;
        for value in row {
            let count = projection::count(value, &mut budget, 0)?;
            memory = memory.checked_add(count.memory).ok_or_else(exhausted)?;
            row_length = row_length
                .checked_add(projection::framed(VALUES, count.encoded)?)
                .ok_or_else(exhausted)?;
        }
        encoded = encoded
            .checked_add(projection::framed(ROWS, row_length)?)
            .ok_or_else(exhausted)?;
    }
    Ok(projection::Count { memory, encoded })
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical aggregate response",
    }
}

fn invalid() -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "canonical aggregate response",
        reason: "encoded length differs",
    }
}

/* === Tests === */
