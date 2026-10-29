/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod get_captured_record_schema;
mod list_records;
mod watch_records;

pub use get_captured_record_schema::GetCapturedRecordSchema;
pub use list_records::ListRecords;
pub use watch_records::WatchRecords;

use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;

#[allow(unused_imports)]
use dokime_domain::model::definition::wire::*;
use dokime_domain::model::execution::wire_cursor::Cursor;

use dokime_domain::model::evidence::wire::*;
#[allow(unused_imports)]
use dokime_domain::model::identity::wire::*;

#[harmos::catalog]
pub enum Operations {
    ListRecords(ListRecords),
    WatchRecords(WatchRecords),
    GetCapturedRecordSchema(GetCapturedRecordSchema),
}

impl Endpoint for ListRecords {
    const PATH: &'static str = "/dokime.v1.Evidence/ListRecords";
}

impl Endpoint for WatchRecords {
    const PATH: &'static str = "/dokime.v1.Evidence/WatchRecords";
}

impl Endpoint for GetCapturedRecordSchema {
    const PATH: &'static str = "/dokime.v1.Evidence/GetCapturedRecordSchema";
}
