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
mod schemas;
#[rustfmt::skip]
mod resources;
#[rustfmt::skip]
mod types;

/* === Re-Exports === */

pub use ::harmos::client::{CallOptions, ClientTransport, Watching};
pub use connection::{ConnectionSettings, connect, connect_with};
pub use handles::{Attribution, Case, Cycle, Execution, Outcome, Progress, ProgressStream, RegisteredPlugin};
pub use resources::*;
pub use types::*;

pub use schemas::FrozenSchemaCache;

pub use ::harmos::ClientSettings;
pub use ::harmos::client::{Input, Method, NoItems, Running};
/* === Imports === */

// Standard Library Imports
use std::{ops::Deref, sync::Arc};

// Third Party Imports
use prost_reflect::ReflectMessage;
use tonic::transport::Channel;

// Local Imports
use crate::DiscoveredContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A Thin Typed Client over Harmos's Generated-Message Transport.
#[derive(Clone)]
pub struct Client {
    transport: ::harmos::Client,
    resources: resources::Root,
    context: CallOptions,
    lifetime: Arc<handles::Lifetime>,
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
            context: CallOptions::default(),
            lifetime: Arc::new(handles::Lifetime::default()),
        }
    }
    pub fn from_channel(channel: Channel, settings: ClientSettings) -> Result<Self, tonic::Status> {
        Ok(Self::new(::harmos::Client::new(channel, settings)?))
    }
    pub fn transport(&self) -> &::harmos::Client {
        &self.transport
    }

    // Bind Authoritative Host Discovery to Generated Dokime Messages, Checking Their Schema.
    pub fn method<Q: ReflectMessage + Default, R: ReflectMessage + Default>(
        &self, contract: DiscoveredContract,
    ) -> Result<Method<Q, R>, tonic::Status> {
        Method::new(contract)
    }

    pub async fn call<Q, R>(&self, method: &Method<Q, R>, request: Q) -> Result<R, tonic::Status>
    where
        Q: prost::Message + Default + Send + 'static,
        R: prost::Message + Default + Send + 'static,
    {
        self.transport.call(method, request).await
    }
    pub async fn call_with<Q, R>(&self, method: &Method<Q, R>, request: Q, context: CallOptions) -> Result<R, tonic::Status>
    where
        Q: prost::Message + Default + Send + 'static,
        R: prost::Message + Default + Send + 'static,
    {
        self.transport.call_with(method, request, context).await
    }

    // Upload, Download, Watch and Duplex Calls Retain Harmos's Explicit Finish and Join.
    pub async fn start<Q, R, I, O>(&self, method: &Method<Q, R, I, O>, request: Q) -> Result<Running<R, I, O>, tonic::Status>
    where
        Q: prost::Message + Default + Send + 'static,
        R: prost::Message + Default + Send + 'static,
        I: prost::Message + Default + Send + 'static,
        O: prost::Message + Default + Send + 'static,
    {
        self.transport.start(method, request).await
    }
    pub async fn start_with<Q, R, I, O>(
        &self, method: &Method<Q, R, I, O>, request: Q, context: CallOptions,
    ) -> Result<Running<R, I, O>, tonic::Status>
    where
        Q: prost::Message + Default + Send + 'static,
        R: prost::Message + Default + Send + 'static,
        I: prost::Message + Default + Send + 'static,
        O: prost::Message + Default + Send + 'static,
    {
        self.transport.start_with(method, request, context).await
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
