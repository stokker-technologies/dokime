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

mod generate_key;
mod read_listener;
mod regenerate_certificate;
mod update_listener;

/* === Re-Exports === */

pub use generate_key::GenerateKey;
pub use read_listener::ReadListener;
pub use regenerate_certificate::RegenerateCertificate;
pub use update_listener::UpdateListener;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{Endpoint, ORDINARY, TRANSFER};
use crate::model::remote::{GeneratedAccessKey, RemoteListenerSettings, RemoteListenerStatus};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos_domain::catalog]
pub enum Operations {
    ReadListener(ReadListener),
    UpdateListener(UpdateListener),
    GenerateKey(GenerateKey),
    RegenerateCertificate(RegenerateCertificate),
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

impl Endpoint for GenerateKey {
    const PATH: &'static str = "/dokime.v1.Remote/GenerateKey";
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for RegenerateCertificate {
    const PATH: &'static str = "/dokime.v1.Remote/RegenerateCertificate";
    const LOCAL_ONLY: bool = true;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
