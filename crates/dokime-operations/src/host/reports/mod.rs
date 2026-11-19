/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

mod capture;
pub mod create;
pub mod get;
pub mod publish;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use dokime_protocol::host::ReportsCreateResponse;
use dokime_protocol::model::Document;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
pub(crate) struct PreparedReport {
    #[prost(string, tag = "1")]
    execution: String,

    #[prost(message, optional, tag = "2")]
    document: Option<Document>,

    #[prost(bytes = "vec", tag = "3")]
    execution_seal: Vec<u8>,

    #[prost(message, optional, tag = "5")]
    response: Option<ReportsCreateResponse>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
