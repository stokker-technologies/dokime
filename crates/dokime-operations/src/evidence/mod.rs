/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod get_captured_record_schema;
mod list_records;
mod watch_records;

/* === Re-Exports === */

pub use get_captured_record_schema::GetCapturedRecordSchema;
pub use list_records::ListRecords;
pub use watch_records::WatchRecords;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::FOLLOW;
use super::ORDINARY;

use dokime_domain::model::definition::wire::*;
use dokime_domain::model::execution::wire_cursor::Cursor;

use dokime_domain::model::evidence::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    ListRecords(ListRecords),
    WatchRecords(WatchRecords),
    GetCapturedRecordSchema(GetCapturedRecordSchema),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
