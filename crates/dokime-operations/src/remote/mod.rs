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
mod rename_paired_client;
mod revoke_paired_client;
mod update_listener;

/* === Re-Exports === */

pub use create_pairing::CreatePairing;
pub use list_paired_clients::ListPairedClients;
pub use list_sessions::ListSessions;
pub use read_listener::ReadListener;
pub use regenerate_certificate::RegenerateCertificate;
pub use rename_paired_client::RenamePairedClient;
pub use revoke_paired_client::RevokePairedClient;
pub use update_listener::UpdateListener;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{ORDINARY, TRANSFER};
use dokime_domain::model::remote::{CreatedPairing, PairedClientPage, RemoteListenerSettings, RemoteListenerStatus, RemoteSessionPage};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    ReadListener(ReadListener),
    UpdateListener(UpdateListener),
    CreatePairing(CreatePairing),
    ListPairedClients(ListPairedClients),
    RevokePairedClient(RevokePairedClient),
    RegenerateCertificate(RegenerateCertificate),
    ListSessions(ListSessions),
    RenamePairedClient(RenamePairedClient),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
