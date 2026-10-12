/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod access;
mod appearance;
mod backend;
mod bundle;
mod catalog;
mod charged;
mod chunked;
mod config;
mod content;
mod deadline;
mod documents;
mod enums;
mod error;
mod evidence;
mod execution;
mod history;
mod logs;
mod method;
mod notes;
mod observation;
mod operations;
mod planning;
mod plots;
mod plugins;
mod prompts;
mod query;
mod redaction;
mod reports;
mod server;
mod settings;
mod telemetry;
mod transport;
mod value;

/* === Re-Exports === */
pub use config::{Config, TlsIdentity};
pub use error::Error;
pub use server::Server;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */
pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/served-descriptor.bin"));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn methods() -> impl Iterator<Item = (&'static str, access::Role)> {
    method::METHODS
        .iter()
        .map(|method| (method.path, method.role))
}

/* === Private Functions === */

/* === Tests === */
