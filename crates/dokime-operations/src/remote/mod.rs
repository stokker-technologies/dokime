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
mod read_listener;
mod regenerate_certificate;
mod update_listener;

/* === Re-Exports === */

pub use create_pairing::CreatePairing;
pub use read_listener::ReadListener;
pub use regenerate_certificate::RegenerateCertificate;
pub use update_listener::UpdateListener;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{ORDINARY, TRANSFER};
use dokime_domain::model::remote::{CreatedPairing, RemoteListenerSettings, RemoteListenerStatus};

/* === Constants === */

/* === Macros === */

#[cfg(feature = "runtime")]
crate::hosted!(CreatePairing, ReadListener, RegenerateCertificate, UpdateListener);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    ReadListener(ReadListener),
    UpdateListener(UpdateListener),
    CreatePairing(CreatePairing),
    RegenerateCertificate(RegenerateCertificate),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
