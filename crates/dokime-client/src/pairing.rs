/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Pairing File Input And Certificate Trust Validation

/* === Modules === */

/* === Re-Exports === */

pub use dokime_domain::model::remote::PairingFile;

/* === Imports === */

// Standard Library Imports
use std::path::Path;

// Third Party Imports
use rustls::pki_types::{CertificateDer, pem::PemObject};

// Local Imports

/* === Constants === */

const PAIRING_BYTES: usize = 1_048_576;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn read(path: &Path) -> Result<PairingFile, super::profiles::Error> {
    let bytes = harmos::filesystem::input::read(path, PAIRING_BYTES)?;
    let pairing: PairingFile =
        serde_json::from_slice(&bytes).map_err(|_| super::profiles::Error::Invalid("invalid version 1 pairing file".into()))?;
    pairing.validate().map_err(super::profiles::Error::Invalid)?;
    if let Some(ca) = &pairing.ca_certificate_pem {
        validate_ca(ca)?;
    }
    Ok(pairing)
}

/* === Private Functions === */

pub(crate) fn validate_ca(pem: &str) -> Result<(), super::profiles::Error> {
    let mut roots = rustls::RootCertStore::empty();
    for certificate in CertificateDer::pem_slice_iter(pem.as_bytes()) {
        let certificate = certificate.map_err(|_| super::profiles::Error::Invalid("invalid CA certificate PEM".into()))?;
        roots.add(certificate).map_err(|_| super::profiles::Error::Invalid("invalid CA certificate DER".into()))?;
    }
    if roots.is_empty() {
        return Err(super::profiles::Error::Invalid("CA certificate PEM contains no certificates".into()));
    }
    Ok(())
}

/* === Tests === */
