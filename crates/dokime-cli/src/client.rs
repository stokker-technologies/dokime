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

pub(crate) use resources::*;
pub(crate) use types::*;
pub(crate) use ::harmos::{ClientSettings, client::CallOptions};

/* === Imports === */

use std::ops::Deref;
use tonic::transport::Channel;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct Client {
    transport: ::harmos::Client,
    resources: resources::Root,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    pub fn new(transport: ::harmos::Client) -> Self {
        let resources = resources::Root::new(transport.transport_handle());
        Self {
            transport,
            resources,
        }
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
