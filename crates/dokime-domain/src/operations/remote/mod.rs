/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Remote Listener Operation Catalog

/* === Modules === */

mod create_pairing;
mod list_paired_clients;
mod list_sessions;
mod read_listener;
mod regenerate_certificate;
mod revoke_paired_client;
mod update_listener;

/* === Re-Exports === */

pub use create_pairing::CreatePairing;
pub use list_paired_clients::ListPairedClients;
pub use list_sessions::ListSessions;
pub use read_listener::ReadListener;
pub use regenerate_certificate::RegenerateCertificate;
pub use revoke_paired_client::RevokePairedClient;
pub use update_listener::UpdateListener;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{Endpoint, ORDINARY, TRANSFER};
use crate::model::remote::{CreatedPairing, PairedClientPage, RemoteListenerSettings, RemoteListenerStatus, RemoteSessionPage};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos_domain::catalog]
pub enum Operations {
    ReadListener(ReadListener),
    UpdateListener(UpdateListener),
    CreatePairing(CreatePairing),
    ListPairedClients(ListPairedClients),
    RevokePairedClient(RevokePairedClient),
    RegenerateCertificate(RegenerateCertificate),
    ListSessions(ListSessions),
}

/* === Traits === */

/* === Implementations === */

impl Endpoint for ReadListener {
    const PATH: &'static str = "/dokime.v1.Remote/ReadListener";
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for UpdateListener {
    const PATH: &'static str = "/dokime.v1.Remote/UpdateListener";
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for CreatePairing {
    const PATH: &'static str = "/dokime.v1.Remote/CreatePairing";
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for ListPairedClients {
    const PATH: &'static str = "/dokime.v1.Remote/ListPairedClients";
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for RevokePairedClient {
    const PATH: &'static str = "/dokime.v1.Remote/RevokePairedClient";
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for ListSessions {
    const PATH: &'static str = "/dokime.v1.Remote/ListSessions";
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for RegenerateCertificate {
    const PATH: &'static str = "/dokime.v1.Remote/RegenerateCertificate";
    const LOCAL_ONLY: bool = true;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
