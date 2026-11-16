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

// Third Party Imports
use rusqlite::{OptionalExtension, params};

// Local Imports
use super::*;
use harmos_query::columnar::ColumnKind;
use harmos_query::columnar::ColumnSource;
use harmos_storage::{CompareExchange, ObjectExchange, ObjectReceipt};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn exchange(state: &mut State, exchange: &ObjectExchange) -> Result<CompareExchange> {
    state.check()?;
    exchange.validate()?;
    if harmos_storage::encoded_size(exchange)? > state.settings.chunk_bytes.get() {
        return Err(exhausted());
    }
    let fingerprint = exchange.fingerprint()?;
    let tx = state.connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(db)?;
    let position = exchange.value.position.sequence;
    let replay = tx
        .query_row("SELECT fingerprint,outcome FROM object_receipts WHERE position=?1", [integer(position)?], |row| {
            Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .optional()
        .map_err(db)?;
    if let Some((prior, outcome)) = replay {
        if prior.as_slice() != fingerprint {
            return Err(conflict("conditional replay request differs"));
        }
        return decode(&outcome);
    }
    let floor = meta(&tx, "object_receipt_floor")?;
    if position <= floor {
        return Err(Error::Gap {
            requested: position,
            available: floor.saturating_add(1),
        });
    }
    if exchange.schema.position.sequence <= meta(&tx, "saved")? {
        return Err(conflict("conditional admission positions are already committed"));
    }
    let current = compound::current(&tx, &exchange.value.value.schema.id, &exchange.value.value.item)?;
    let outcome = if current == exchange.expected {
        rows::put_schema(&tx, &state.history, &exchange.schema)?;
        let value = &exchange.value;
        rows::insert(
            &tx,
            &state.history,
            rows::WriteRow {
                position: &value.position,
                kind: 1,
                schema: Some(&value.value.schema),
                item: Some(value.value.item.as_str()),
                scope: value.value.scope.as_ref().map(Identifier::as_str),
                stream: None,
                time: None,
                data: encode(value)?,
            },
        )?;
        let batch = harmos_query::source::objects(&exchange.schema, std::slice::from_ref(value))?;
        columns::store(
            &tx,
            &ColumnSource {
                kind: ColumnKind::Objects,
                schema: value.value.schema.clone(),
            },
            &batch,
            state.settings,
        )?;
        CompareExchange::Applied(ObjectReceipt {
            position: value.position.clone(),
            version: value.version.clone(),
        })
    } else {
        CompareExchange::Conflict { current }
    };
    tx.execute("INSERT INTO object_receipts VALUES(?1,?2,?3)", params![integer(position)?, fingerprint.as_slice(), encode(&outcome)?])
        .map_err(db)?;
    rows::save(&tx, position)?;
    tx.commit().map_err(db)?;
    Ok(outcome)
}

pub(super) fn retire(state: &mut State, through: &StorageCursor) -> Result<()> {
    state.check()?;
    state.history.check(&through.history)?;
    if through.sequence > state.cursor()?.sequence {
        return Err(invalid("receipt retention is ahead of durable data"));
    }
    let tx = state.connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(db)?;
    tx.execute("DELETE FROM object_receipts WHERE position<=?1", [integer(through.sequence)?]).map_err(db)?;
    set_meta(&tx, "object_receipt_floor", meta(&tx, "object_receipt_floor")?.max(through.sequence))?;
    tx.commit().map_err(db)
}

/* === Private Functions === */

/* === Tests === */
