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
use crate::runtime::Result;
use crate::storage::objects::execution::Execution;
use dokime_domain::model::execution::result::TestExecution;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn executions(
    session: &crate::storage::Manager, at: harmos::storage::StorageCursor,
) -> Result<crate::storage::projection::Pages<ObjectRevision<TestExecution>>> {
    let rows = session.read_objects::<Execution>(ObjectRead { at, key: None, versions: ObjectVersions::Latest }).await?.collect().await?;
    let selected = crate::storage::projection::pages(rows, |row| {
        Ok(ObjectRevision { value: row.value.into_execution()?, position: row.position, version: row.version })
    })?;
    for page in selected.iter() {
        for row in page.iter() {
            if let Some(seal) = &row.value.saved {
                check_seal_position(&row.position, &seal.dependencies_saved)?;
            }
        }
    }
    Ok(selected)
}

/* === Private Functions === */

/* === Tests === */
