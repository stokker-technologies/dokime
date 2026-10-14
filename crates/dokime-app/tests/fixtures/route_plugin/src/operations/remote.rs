/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Exercise Independent Sidecar Wire Operations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use dokime::harmos;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::message]
#[derive(Clone)]
pub(crate) struct DynamicNumber {
    value: u32,
}

pub(crate) struct DynamicUnary;

pub(crate) struct DynamicUnaryHandler {
    calls: Arc<crate::Resources>,
}

#[dokime::operation(request = DynamicNumber, response = DynamicNumber, output = DynamicNumber,
    id = "route-fixture.remote.download", version = 1, policy = crate::POLICY,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "fixture.remote.v1", service = "Provider", method = "Download"))]
pub(crate) struct DynamicDownload;

pub(crate) struct DynamicDownloadHandler {
    calls: Arc<crate::Resources>,
}

#[dokime::operation(request = DynamicNumber, response = DynamicNumber, input = DynamicNumber,
    id = "route-fixture.remote.upload", version = 1, policy = crate::POLICY,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "fixture.remote.v1", service = "Provider", method = "Upload"))]
pub(crate) struct DynamicUpload;

pub(crate) struct DynamicUploadHandler {
    calls: Arc<crate::Resources>,
}

#[dokime::operation(request = DynamicNumber, response = DynamicNumber, input = DynamicNumber, output = DynamicNumber,
    id = "route-fixture.remote.duplex", version = 1, policy = crate::POLICY,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "fixture.remote.v1", service = "Provider", method = "Duplex"))]
pub(crate) struct DynamicDuplex;

pub(crate) struct DynamicDuplexHandler {
    calls: Arc<crate::Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
// Handlers Declare The Wire Contract Their Catalog Entry Provides
impl harmos::operations::Declares for DynamicUnaryHandler {
    type Declaration = DynamicUnary;
}

impl harmos::operations::Declares for DynamicDownloadHandler {
    type Declaration = DynamicDownload;
}

impl harmos::operations::Declares for DynamicUploadHandler {
    type Declaration = DynamicUpload;
}

impl harmos::operations::Declares for DynamicDuplexHandler {
    type Declaration = DynamicDuplex;
}

impl harmos::operations::OperationDeclaration for DynamicUnary {
    type Request = DynamicNumber;
    type Response = DynamicNumber;
    const ID: &'static str = "route-fixture.remote.unary";
    const POLICY: harmos::operations::OperationPolicy = crate::POLICY;
    const VISIBILITY: harmos::operations::Visibility = harmos::operations::Visibility::Host;

    fn visit_wire<V: harmos::operations::WireVisitor>(visitor: V) -> Option<V::Entry> {
        Some(visitor.unary::<Self>())
    }

    fn grant(request: &DynamicNumber) -> Option<harmos::plugin::contract::Grant> {
        match request.value {
            20 => Some(harmos::plugin::contract::Grant::Egress(
                "fixture.remote.allowed".into(),
            )),
            21 => Some(harmos::plugin::contract::Grant::Egress(
                "fixture.remote.refused".into(),
            )),
            _ => None,
        }
    }
}

impl harmos::operations::WireOperation for DynamicUnary {
    const VERSION: u32 = 1;
    const GRPC: Option<harmos::operations::Grpc> = Some(harmos::operations::Grpc::new(
        "fixture.remote.v1",
        "Provider",
        "Unary",
    ));

    fn visit_grpc<V: harmos::operations::GrpcVisitor>(visitor: V) -> Option<V::Entry> {
        Some(visitor.unary::<Self>())
    }
}

impl harmos::operations::Operation for DynamicUnaryHandler {
    type Declaration = DynamicUnary;
    type Resources = crate::Resources;
    type Error = dokime::Refusal;
    type Context<'a> = &'a dokime::Supervisor;
    type Endpoints = ();

    async fn initialize<'ctx>(
        calls: Arc<Self::Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut DynamicNumber,
        _: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<DynamicNumber>, Self::Error> {
        if request.value == 0 {
            return Ok(DynamicNumber {
                value: u32::try_from(self.calls.calls.load(std::sync::atomic::Ordering::SeqCst))
                    .expect("fixture mutation count fits u32"),
            }
            .into());
        }
        self.calls
            .calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(DynamicNumber {
            value: request.value + 1,
        }
        .into())
    }
}

impl harmos::operations::Operation for DynamicDownloadHandler {
    type Declaration = DynamicDownload;
    type Resources = crate::Resources;
    type Error = dokime::Refusal;
    type Context<'a> = &'a dokime::Supervisor;
    type Endpoints = harmos::operations::StreamOutput<DynamicNumber>;

    async fn initialize<'ctx>(
        calls: Arc<Self::Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut DynamicNumber,
        streams: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<DynamicNumber>, Self::Error> {
        self.calls
            .calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        for value in [request.value + 1, request.value + 2] {
            streams
                .send_with(std::mem::size_of::<DynamicNumber>(), || {
                    Ok(DynamicNumber { value })
                })
                .await
                .map_err(|error| dokime::Refusal::new(13, error))?;
        }
        Ok(DynamicNumber {
            value: request.value + 3,
        }
        .into())
    }
}

impl harmos::operations::Operation for DynamicUploadHandler {
    type Declaration = DynamicUpload;
    type Resources = crate::Resources;
    type Error = dokime::Refusal;
    type Context<'a> = &'a dokime::Supervisor;
    type Endpoints = harmos::operations::StreamInput<DynamicNumber>;

    async fn initialize<'ctx>(
        calls: Arc<Self::Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut DynamicNumber,
        streams: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<DynamicNumber>, Self::Error> {
        self.calls
            .calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut value = request.value;
        while let Some(item) = streams.recv().await {
            value += item.value;
        }
        Ok(DynamicNumber { value }.into())
    }
}

impl harmos::operations::Operation for DynamicDuplexHandler {
    type Declaration = DynamicDuplex;
    type Resources = crate::Resources;
    type Error = dokime::Refusal;
    type Context<'a> = &'a dokime::Supervisor;
    type Endpoints = (
        harmos::operations::StreamInput<DynamicNumber>,
        harmos::operations::StreamOutput<DynamicNumber>,
    );

    async fn initialize<'ctx>(
        calls: Arc<Self::Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut DynamicNumber,
        streams: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<DynamicNumber>, Self::Error> {
        self.calls
            .calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut value = request.value;
        while let Some(item) = streams.0.recv().await {
            value += item.value;
            streams
                .1
                .send_with(std::mem::size_of::<DynamicNumber>(), || {
                    Ok(DynamicNumber { value })
                })
                .await
                .map_err(|error| dokime::Refusal::new(13, error))?;
        }
        Ok(DynamicNumber { value }.into())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
