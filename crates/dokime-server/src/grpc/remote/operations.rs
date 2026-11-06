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
use dokime::harmos::{Context, Response};
use tonic::Status;

// Local Imports
use super::{Change, Error};
use dokime::domain::model::remote::{CreatedPairing, PairedClientPage, RemoteListenerStatus, RemoteSessionPage};
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::remote as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Remote(Arc<crate::grpc::preparation::Owner>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<wire::ReadListener> for Remote {
    fn serve<'a>(&'a self, _: wire::ReadListener, _: Context<'a, wire::ReadListener>) -> Served<'a, RemoteListenerStatus> {
        Box::pin(async move { Ok::<_, Status>(crate::grpc::preparation::controller(&self.0).await?.status().await.into()) })
    }
}

impl Serve<wire::UpdateListener> for Remote {
    fn serve<'a>(&'a self, mut request: wire::UpdateListener, _: Context<'a, wire::UpdateListener>) -> Served<'a, RemoteListenerStatus> {
        Box::pin(async move {
            let settings = request.settings.take().ok_or_else(|| Status::invalid_argument("listener settings are required"))?;
            let result = crate::grpc::preparation::controller(&self.0).await?.change(Change::Settings(settings)).await.map_err(status)?;
            Ok::<_, Status>(result.into())
        })
    }
}

impl Serve<wire::CreatePairing> for Remote {
    fn serve<'a>(&'a self, request: wire::CreatePairing, _: Context<'a, wire::CreatePairing>) -> Served<'a, CreatedPairing> {
        Box::pin(async move {
            let result =
                crate::grpc::preparation::controller(&self.0).await?.create_pairing(request.label.clone()).await.map_err(status)?;
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
            let result = crate::grpc::preparation::controller(&self.0).await?.change(Change::Certificate(names)).await.map_err(status)?;
            Ok::<_, Status>(result.into())
        })
    }
}

impl Serve<wire::ListSessions> for Remote {
    fn serve<'a>(&'a self, request: wire::ListSessions, _: Context<'a, wire::ListSessions>) -> Served<'a, RemoteSessionPage> {
        Box::pin(async move {
            crate::grpc::preparation::controller(&self.0)
                .await?
                .sessions(request.cursor.as_deref(), request.limit)
                .map(Response::new)
                .map_err(status)
        })
    }
}

impl Serve<wire::ListPairedClients> for Remote {
    fn serve<'a>(&'a self, request: wire::ListPairedClients, _: Context<'a, wire::ListPairedClients>) -> Served<'a, PairedClientPage> {
        Box::pin(async move {
            crate::grpc::preparation::controller(&self.0)
                .await?
                .clients
                .list(request.cursor.as_deref(), request.limit)
                .map(Response::new)
                .map_err(status)
        })
    }
}

impl Serve<wire::RevokePairedClient> for Remote {
    fn serve<'a>(&'a self, request: wire::RevokePairedClient, _: Context<'a, wire::RevokePairedClient>) -> Served<'a, ()> {
        Box::pin(async move {
            crate::grpc::preparation::controller(&self.0)
                .await?
                .revoke_client(request.identifier.clone())
                .await
                .map(Response::new)
                .map_err(status)
        })
    }
}

impl Serve<wire::RenamePairedClient> for Remote {
    fn serve<'a>(&'a self, request: wire::RenamePairedClient, _: Context<'a, wire::RenamePairedClient>) -> Served<'a, ()> {
        Box::pin(async move {
            crate::grpc::preparation::controller(&self.0)
                .await?
                .rename_client(request.identifier.clone(), request.label.clone())
                .await
                .map(Response::new)
                .map_err(status)
        })
    }
}

/* === Public Functions === */

pub(in crate::grpc) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, owner: Arc<crate::grpc::preparation::Owner>,
) -> Result<(), Error> {
    let served = Arc::new(Remote(owner));
    setup.resource::<Host<wire::ReadListener>>(Host::<wire::ReadListener>::new(served.clone()));
    setup.resource::<Host<wire::UpdateListener>>(Host::<wire::UpdateListener>::new(served.clone()));
    setup.resource::<Host<wire::CreatePairing>>(Host::<wire::CreatePairing>::new(served.clone()));
    setup.resource::<Host<wire::RegenerateCertificate>>(Host::<wire::RegenerateCertificate>::new(served.clone()));
    setup.resource::<Host<wire::ListPairedClients>>(Host::<wire::ListPairedClients>::new(served.clone()));
    setup.resource::<Host<wire::RevokePairedClient>>(Host::<wire::RevokePairedClient>::new(served.clone()));
    setup.resource::<Host<wire::ListSessions>>(Host::<wire::ListSessions>::new(served.clone()));

    setup.resource::<Host<wire::RenamePairedClient>>(Host::<wire::RenamePairedClient>::new(served));

    setup.operations::<dokime::operations::remote::Operations>(dokime::app::Lanes::Open);
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
