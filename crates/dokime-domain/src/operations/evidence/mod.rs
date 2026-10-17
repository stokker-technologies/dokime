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
use super::Access;
use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;
#[allow(unused_imports)]
use crate::model::definition::wire::*;
use crate::model::execution::wire_cursor::Cursor;
#[allow(unused_imports)]
use crate::model::identity::wire::*;

use crate::model::evidence::wire::*;

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
impl Endpoint for ListRecords {
    const PATH: &'static str = "/dokime.v1.Evidence/ListRecords";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for WatchRecords {
    const PATH: &'static str = "/dokime.v1.Evidence/WatchRecords";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetCapturedRecordSchema {
    const PATH: &'static str = "/dokime.v1.Evidence/GetCapturedRecordSchema";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
