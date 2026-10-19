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
use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;

#[allow(unused_imports)]
use crate::model::definition::wire::*;
use crate::model::execution::wire_cursor::Cursor;

use crate::model::evidence::wire::*;
#[allow(unused_imports)]
use crate::model::identity::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos_domain::catalog]
pub enum Operations {
    ListRecords(ListRecords),
    WatchRecords(WatchRecords),
    GetCapturedRecordSchema(GetCapturedRecordSchema),
}

/* === Traits === */

/* === Implementations === */

impl Endpoint for ListRecords {
    const PATH: &'static str = "/dokime.v1.Evidence/ListRecords";
}

impl Endpoint for WatchRecords {
    const PATH: &'static str = "/dokime.v1.Evidence/WatchRecords";
}

impl Endpoint for GetCapturedRecordSchema {
    const PATH: &'static str = "/dokime.v1.Evidence/GetCapturedRecordSchema";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
