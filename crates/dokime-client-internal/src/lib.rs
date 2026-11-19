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

mod wire { pub(crate) use dokime_protocol::host; }
mod schemas { include!(concat!(env!("OUT_DIR"), "/client_schemas.rs")); }


#[rustfmt::skip]
#[allow(dead_code, reason = "The Shared Generator Emits the Complete Internal Transport")]
mod resources {
    include!(concat!(env!("OUT_DIR"), "/client_resources.rs"));
}
#[rustfmt::skip]
#[allow(unused_imports, reason = "The Shared Generator Emits the Complete Internal Type Inventory")]
mod types {
    include!(concat!(env!("OUT_DIR"), "/client_types.rs"));
}

/* === Re-Exports === */

pub use ::harmos::{ClientSettings, client::CallOptions};
pub use schemas::FrozenSchemaCache;
pub use harmos_protocol::model::PageRequest;
pub use resources::*;
pub use types::*;

/* === Imports === */

use std::ops::Deref;
use tonic::transport::Channel;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Client {
    transport: ::harmos::Client,
    resources: resources::Root,
    context: CallOptions,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    pub fn new(transport: ::harmos::Client) -> Self {
        let resources = resources::Root::new(transport.transport_handle());
        Self { transport, resources, context: CallOptions::default() }
    }
    pub fn from_channel(channel: Channel, settings: ClientSettings) -> Result<Self, tonic::Status> {
        Ok(Self::new(::harmos::Client::new(channel, settings)?))
    }
    pub fn transport(&self) -> &::harmos::Client {
        &self.transport
    }
}

impl Deref for Client {
    type Target = resources::Root;
    fn deref(&self) -> &Self::Target {
        &self.resources
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
