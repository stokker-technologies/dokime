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

mod host;

/* === Re-Exports === */

pub use host::{
    Backend, Cycle, Cycles, Executions, Health, HostClient, Plugins, Progress, Prompts, Records, Reports, Samples, Suites, Templates,
};

pub use harmos::client::{ClientSettings, Input, Method, NoItems, Running};
pub use harmos_protocol::host::{Catalog, CatalogKind, CatalogPage, DiscoveredOperation, OperationContract};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost_reflect::ReflectMessage;
use tonic::{
    IntoRequest, Request, Response, Status,
    metadata::{Ascii, MetadataValue},
    transport::Channel,
};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A Thin Typed Client over Harmos's Generated-Message Transport.
#[derive(Clone)]
pub struct Client {
    transport: harmos::Client,
    authorization: Option<MetadataValue<Ascii>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    pub fn new(transport: harmos::Client) -> Self {
        Self { transport, authorization: None }
    }
    pub fn from_channel(channel: Channel, settings: ClientSettings) -> Result<Self, Status> {
        Ok(Self::new(harmos::Client::new(channel, settings)?))
    }
    pub fn authorization(mut self, value: MetadataValue<Ascii>) -> Self {
        self.authorization = Some(value);
        self
    }
    pub fn transport(&self) -> &harmos::Client {
        &self.transport
    }

    // Bind Authoritative Host Discovery to Generated Dokime Messages, Checking Their Schema.
    pub fn method<Q: ReflectMessage + Default, R: ReflectMessage + Default>(
        &self, contract: OperationContract,
    ) -> Result<Method<Q, R>, Status> {
        Method::new(contract)
    }

    pub async fn catalog(&self, request: impl IntoRequest<Catalog>) -> Result<Response<CatalogPage>, Status> {
        self.transport.catalog(self.request(request)).await
    }

    pub async fn call<Q, R>(&self, method: &Method<Q, R>, request: impl IntoRequest<Q>) -> Result<Response<R>, Status>
    where
        Q: prost::Message + Default + Send + 'static,
        R: prost::Message + Default + Send + 'static,
    {
        self.transport.call(method, self.request(request)).await
    }

    // Upload, Download, Watch and Duplex Calls Retain Harmos's Explicit Finish and Join.
    pub async fn start<Q, R, I, O>(&self, method: &Method<Q, R, I, O>, request: impl IntoRequest<Q>) -> Result<Running<R, I, O>, Status>
    where
        Q: prost::Message + Default + Send + 'static,
        R: prost::Message + Default + Send + 'static,
        I: prost::Message + Default + Send + 'static,
        O: prost::Message + Default + Send + 'static,
    {
        self.transport.start(method, self.request(request)).await
    }

    fn request<T>(&self, request: impl IntoRequest<T>) -> Request<T> {
        let mut request = request.into_request();
        if let Some(authorization) = &self.authorization {
            request.metadata_mut().insert("authorization", authorization.clone());
        }
        request
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
