/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client

/* === Modules === */

mod connection;
mod files;
mod handles;
pub(crate) mod mutations;
pub(crate) mod queries;
mod schemas;
#[rustfmt::skip]
#[allow(dead_code, reason = "The Shared Generator Emits the Complete Internal Transport")]
pub(crate) mod resources;
#[rustfmt::skip]
#[allow(unused_imports, reason = "The Shared Generator Emits the Complete Internal Type Inventory")]
mod types;

/* === Re-Exports === */

pub(crate) use ::harmos::client::{CallOptions, Watching};
pub use connection::{ConnectionSettings, connect, connect_with};
pub use handles::{Attribution, Case, Cycle, CyclesQuery, Execution, ExecutionsQuery, Outcome, Progress, ProgressStream, RegisteredPlugin, PluginRegisterOptions};
pub use mutations::{
    ArtifactCreateOptions, CycleCreateOptions, CycleSealOptions, ExecutionCancelOptions, ExecutionCreateOptions, ExecutionSealOptions,
    ExecutionStartOptions, ExecutionStopOptions, NoteCreateOptions,
};
pub use queries::{Records, Samples};
pub(crate) use resources::*;
pub(crate) use types::*;
pub use dokime_protocol::host::{CyclePhase, ExecutionState, ResultStatus};

pub(crate) use schemas::FrozenSchemaCache;

pub(crate) use ::harmos::ClientSettings;

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use tonic::transport::Channel;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A Thin Typed Client over Harmos's Generated-Message Transport.
#[derive(Clone)]
pub struct Client {
    resources: resources::Root,
    context: CallOptions,
    lifetime: Arc<handles::Lifetime>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    pub(crate) fn from_channel(channel: Channel, settings: ClientSettings) -> Result<Self, tonic::Status> {
        let transport = ::harmos::Client::new(channel, settings)?;
        let resources = resources::Root::new(transport.transport_handle());
        Ok(Self { resources, context: CallOptions::default(), lifetime: Arc::new(handles::Lifetime::default()) })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
