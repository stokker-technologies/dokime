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
use harmos::storage::{ObjectRead, ObjectRevision, query::ObjectVersions};

// Local Imports
use super::execution::check_seal_position;
use crate::Result;
use dokime::{
    execution::TestExecution,
    storage::{Manager, objects::execution::Execution},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// The latest saved revision of every stored execution.
pub(crate) async fn executions(storage: &Manager) -> Result<Vec<ObjectRevision<TestExecution>>> {
    let mut selected = Vec::new();
    let rows = storage
        .read_objects::<Execution>(ObjectRead {
            at: storage.cursor_saved(),
            key: None,
            versions: ObjectVersions::Latest,
            filters: Vec::new(),
        })
        .await?
        .collect()
        .await?;
    for row in rows {
        let execution = row.value.into_execution()?;
        if let Some(seal) = &execution.saved {
            check_seal_position(&row.position, &seal.dependencies_saved)?;
        }
        selected.push(ObjectRevision {
            value: execution,
            position: row.position,
            version: row.version,
        });
    }
    Ok(selected)
}

/* === Private Functions === */

/* === Tests === */
