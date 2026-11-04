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
mod preparation;
mod prompts;
mod query;
pub(crate) mod redaction;
mod remote;
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
pub use observation::WATCH_HEARTBEAT;
pub use preparation::PreparedService;
pub use server::{Server, Service};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use dokime::harmos::catalog::{Catalog, Projection};

// Local Imports
use dokime::harmos::operations::Contracts;
use dokime::operations::Operations;

/* === Constants === */

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/grpc/operations/descriptors.bin"));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn methods() -> impl Iterator<Item = (String, bool)> {
    let mut methods = Vec::new();
    let result = <Operations as Catalog<Contracts>>::ENTRIES.try_for_each(|entry| {
        let grpc = entry.grpc.as_ref().expect("product operations declare their gRPC binding");
        methods.push((grpc.path(), dokime::operations::local_only(entry.operation.id)));
        Ok::<_, std::convert::Infallible>(())
    });
    match result {
        Ok(()) => methods.into_iter(),
        Err(impossible) => match impossible {},
    }
}

/* === Private Functions === */

/* === Tests === */
