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
mod authorization;
mod backend;
mod bundle;
mod catalog;
mod chunked;
mod config;
mod content;
mod documents;
mod enums;
mod error;
mod evidence;
mod execution;
mod files;
mod history;
mod logs;
mod media;
mod notes;
mod observation;
mod planning;
mod plots;
mod plugins;
mod prompts;
mod query;
pub(crate) mod redaction;
mod reports;
mod server;
mod settings;
mod telemetry;
mod trace;
mod transcode;
mod value;

/* === Re-Exports === */
pub use config::{Config, TlsIdentity};
pub use error::Error;
pub use server::{Server, Service};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use harmos::catalog::{Catalog, Projection};

// Local Imports
use dokime_domain::operations::Endpoints;
use dokime_domain::operations::Operations;

/* === Constants === */
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/grpc/operations/descriptors.bin"));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn methods() -> impl Iterator<Item = (&'static str, access::Role)> {
    let mut methods = Vec::new();
    let result = <Operations as Catalog<Endpoints>>::ENTRIES.try_for_each(|entry| {
        methods.push((entry.path, entry.access));
        Ok::<_, std::convert::Infallible>(())
    });
    match result {
        Ok(()) => methods.into_iter(),
        Err(impossible) => match impossible {},
    }
}

/* === Private Functions === */

/* === Tests === */
