/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

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

pub use config::{Config, TlsIdentity};
pub use error::Error;
pub use observation::WATCH_HEARTBEAT;
pub use preparation::PreparedService;
pub use server::{Server, Service};

use dokime::harmos::catalog::{Catalog, Projection};

use dokime::operations::Endpoints;
use dokime::operations::Operations;

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/grpc/operations/descriptors.bin"));

pub fn methods() -> impl Iterator<Item = (&'static str, bool)> {
    let mut methods = Vec::new();
    let result = <Operations as Catalog<Endpoints>>::ENTRIES.try_for_each(|entry| {
        methods.push((entry.path, entry.local_only));
        Ok::<_, std::convert::Infallible>(())
    });
    match result {
        Ok(()) => methods.into_iter(),
        Err(impossible) => match impossible {},
    }
}
