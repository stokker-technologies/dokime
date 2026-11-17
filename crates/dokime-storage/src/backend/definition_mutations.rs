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
use harmos_storage::DefinitionKind;
use rusqlite::{Connection, OptionalExtension};

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn replay(
    connection: &Connection, mutation: Option<&MutationHeader>, kind: DefinitionKind, name: &Identifier,
) -> Result<Option<DefinitionOutcome>> {
    let Some(mutation) = mutation else { return Ok(None) };
    let prior: Option<Vec<u8>> = connection
        .query_row("SELECT data FROM storage_mutations WHERE id=?1", [text(&mutation.mutation)?], |row| row.get(0))
        .optional()
        .map_err(db)?;
    prior.map(|data| mutation.replay_definition(&decode(&data)?, kind, name)).transpose()
}
pub(super) fn write(
    connection: &Connection, mutation: Option<&MutationHeader>, kind: DefinitionKind, name: &Identifier, outcome: &DefinitionOutcome,
) -> Result<()> {
    if let Some(mutation) = mutation {
        let receipt = mutation.definition(kind, name.clone(), outcome.clone());
        save_mutation_receipt(connection, &receipt)?;
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
