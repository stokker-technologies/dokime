/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Remote Listener Management Operations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::operations::{Context, Operation, Response};
use tonic::Status;

// Local Imports
use super::{Change, Controller, Error};
use dokime::RuntimeHandle;
use dokime::domain::{
    model::remote::{CreatedPairing, PairedClientPage, RemoteListenerStatus, RemoteSessionPage},
    operations::remote as wire,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct ReadListener {
    controller: Arc<Controller>,
}

struct UpdateListener {
    controller: Arc<Controller>,
}

struct CreatePairing {
    controller: Arc<Controller>,
}

struct RegenerateCertificate {
    controller: Arc<Controller>,
}

struct ListSessions {
    controller: Arc<Controller>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for ReadListener {
    type Declaration = wire::ReadListener;
    type Resources = Controller;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(controller: Arc<Controller>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { controller })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _: &mut wire::ReadListener, _: &mut (),
    ) -> Result<Response<RemoteListenerStatus>, Status> {
        Ok(self.controller.status().await.into())
    }
}

impl Operation for UpdateListener {
    type Declaration = wire::UpdateListener;
    type Resources = Controller;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(controller: Arc<Controller>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { controller })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::UpdateListener, _: &mut (),
    ) -> Result<Response<RemoteListenerStatus>, Status> {
        let settings = request.settings.take().ok_or_else(|| Status::invalid_argument("listener settings are required"))?;
        let result = self.controller.change(Change::Settings(settings)).await.map_err(status)?;
        Ok(result.into())
    }
}

impl Operation for CreatePairing {
    type Declaration = wire::CreatePairing;
    type Resources = Controller;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(controller: Arc<Controller>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { controller })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::CreatePairing, _: &mut (),
    ) -> Result<Response<CreatedPairing>, Status> {
        let result = self.controller.create_pairing(request.label.clone(), request.endpoint.clone()).await.map_err(status)?;
        Ok(result.into())
    }
}

impl Operation for RegenerateCertificate {
    type Declaration = wire::RegenerateCertificate;
    type Resources = Controller;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(controller: Arc<Controller>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { controller })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::RegenerateCertificate, _: &mut (),
    ) -> Result<Response<RemoteListenerStatus>, Status> {
        let names = std::mem::take(&mut request.certificate_names);
        let result = self.controller.change(Change::Certificate(names)).await.map_err(status)?;
        Ok(result.into())
    }
}

impl Operation for ListSessions {
    type Declaration = wire::ListSessions;
    type Resources = Controller;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(controller: Arc<Controller>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { controller })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::ListSessions, _: &mut (),
    ) -> Result<Response<RemoteSessionPage>, Status> {
        self.controller.sessions(request.cursor.as_deref(), request.limit).map(Response::new).map_err(status)
    }
}

struct ListPairedClients {
    controller: Arc<Controller>,
}
struct RevokePairedClient {
    controller: Arc<Controller>,
}

impl Operation for ListPairedClients {
    type Declaration = wire::ListPairedClients;
    type Resources = Controller;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(controller: Arc<Controller>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { controller })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::ListPairedClients, _: &mut (),
    ) -> Result<Response<PairedClientPage>, Status> {
        self.controller.clients.list(request.cursor.as_deref(), request.limit).map(Response::new).map_err(status)
    }
}

impl Operation for RevokePairedClient {
    type Declaration = wire::RevokePairedClient;
    type Resources = Controller;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(controller: Arc<Controller>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { controller })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, request: &mut wire::RevokePairedClient, _: &mut ()) -> Result<Response<()>, Status> {
        self.controller.revoke_client(request.identifier.clone()).await.map(Response::new).map_err(status)
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, controller: Arc<Controller>) -> Result<(), Error> {
    let operations = runtime.operations();
    operations.register_wire_operation::<ReadListener>("dokime", dokime::app::Lanes::Open, controller.clone())?;
    operations.register_wire_operation::<UpdateListener>("dokime", dokime::app::Lanes::Open, controller.clone())?;
    operations.register_wire_operation::<CreatePairing>("dokime", dokime::app::Lanes::Open, controller.clone())?;
    operations.register_wire_operation::<RegenerateCertificate>("dokime", dokime::app::Lanes::Open, controller.clone())?;
    operations.register_wire_operation::<ListPairedClients>("dokime", dokime::app::Lanes::Open, controller.clone())?;
    operations.register_wire_operation::<RevokePairedClient>("dokime", dokime::app::Lanes::Open, controller.clone())?;
    operations.register_wire_operation::<ListSessions>("dokime", dokime::app::Lanes::Open, controller)?;
    Ok(())
}

/* === Private Functions === */

fn status(error: Error) -> Status {
    match error {
        Error::Configuration(message) => Status::invalid_argument(message),
        Error::Listener(message) => Status::invalid_argument(message),
        Error::Settings(error) => crate::grpc::execution::status(error),
        other => Status::unavailable(other.to_string()),
    }
}

/* === Tests === */
