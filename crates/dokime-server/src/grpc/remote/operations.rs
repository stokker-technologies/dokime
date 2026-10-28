/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::{Context, RegistrationExt, Response};
use tonic::Status;

// Local Imports
use super::{Change, Controller, Error};
use dokime::RuntimeHandle;
use dokime::domain::model::remote::{CreatedPairing, PairedClientPage, RemoteListenerStatus, RemoteSessionPage};
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::remote as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Remote(Arc<Controller>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<wire::ReadListener> for Remote {
    fn serve<'a>(&'a self, _: wire::ReadListener, _: Context<'a, wire::ReadListener>) -> Served<'a, RemoteListenerStatus> {
        Box::pin(async move { Ok::<_, Status>(self.0.status().await.into()) })
    }
}

impl Serve<wire::UpdateListener> for Remote {
    fn serve<'a>(&'a self, mut request: wire::UpdateListener, _: Context<'a, wire::UpdateListener>) -> Served<'a, RemoteListenerStatus> {
        Box::pin(async move {
            let settings = request.settings.take().ok_or_else(|| Status::invalid_argument("listener settings are required"))?;
            let result = self.0.change(Change::Settings(settings)).await.map_err(status)?;
            Ok::<_, Status>(result.into())
        })
    }
}

impl Serve<wire::CreatePairing> for Remote {
    fn serve<'a>(&'a self, request: wire::CreatePairing, _: Context<'a, wire::CreatePairing>) -> Served<'a, CreatedPairing> {
        Box::pin(async move {
            let result = self.0.create_pairing(request.label.clone(), request.endpoint.clone()).await.map_err(status)?;
            Ok::<_, Status>(result.into())
        })
    }
}

impl Serve<wire::RegenerateCertificate> for Remote {
    fn serve<'a>(
        &'a self, mut request: wire::RegenerateCertificate, _: Context<'a, wire::RegenerateCertificate>,
    ) -> Served<'a, RemoteListenerStatus> {
        Box::pin(async move {
            let names = std::mem::take(&mut request.certificate_names);
            let result = self.0.change(Change::Certificate(names)).await.map_err(status)?;
            Ok::<_, Status>(result.into())
        })
    }
}

impl Serve<wire::ListSessions> for Remote {
    fn serve<'a>(&'a self, request: wire::ListSessions, _: Context<'a, wire::ListSessions>) -> Served<'a, RemoteSessionPage> {
        Box::pin(async move { self.0.sessions(request.cursor.as_deref(), request.limit).map(Response::new).map_err(status) })
    }
}

impl Serve<wire::ListPairedClients> for Remote {
    fn serve<'a>(&'a self, request: wire::ListPairedClients, _: Context<'a, wire::ListPairedClients>) -> Served<'a, PairedClientPage> {
        Box::pin(async move { self.0.clients.list(request.cursor.as_deref(), request.limit).map(Response::new).map_err(status) })
    }
}

impl Serve<wire::RevokePairedClient> for Remote {
    fn serve<'a>(&'a self, request: wire::RevokePairedClient, _: Context<'a, wire::RevokePairedClient>) -> Served<'a, ()> {
        Box::pin(async move { self.0.revoke_client(request.identifier.clone()).await.map(Response::new).map_err(status) })
    }
}

/* === Public Functions === */

pub(super) async fn register(runtime: &RuntimeHandle, controller: Arc<Controller>) -> Result<(), Error> {
    let served = Arc::new(Remote(controller));
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::ReadListener>::new(served.clone()))?;
    dependencies.insert(Host::<wire::UpdateListener>::new(served.clone()))?;
    dependencies.insert(Host::<wire::CreatePairing>::new(served.clone()))?;
    dependencies.insert(Host::<wire::RegenerateCertificate>::new(served.clone()))?;
    dependencies.insert(Host::<wire::ListPairedClients>::new(served.clone()))?;
    dependencies.insert(Host::<wire::RevokePairedClient>::new(served.clone()))?;
    dependencies.insert(Host::<wire::ListSessions>::new(served.clone()))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies)
        .operation::<wire::ReadListener>()
        .operation::<wire::UpdateListener>()
        .operation::<wire::CreatePairing>()
        .operation::<wire::RegenerateCertificate>()
        .operation::<wire::ListPairedClients>()
        .operation::<wire::RevokePairedClient>()
        .operation::<wire::ListSessions>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
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
