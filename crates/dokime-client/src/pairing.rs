/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Pairing File Input and Certificate Trust Validation

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
    if path.extension().and_then(|extension| extension.to_str()) != Some("pair") {
        return Err(super::profiles::Error::Invalid("remote access files require .pair".into()));
    }
    let bytes = harmos::filesystem::input::read(path, PAIRING_BYTES)?;
    let pairing: PairingFile =
        serde_json::from_slice(&bytes).map_err(|_| super::profiles::Error::Invalid("invalid version 1 pairing file".into()))?;
    pairing.validate().map_err(super::profiles::Error::Invalid)?;
    if let Some(ca) = &pairing.certificate_pem {
        validate_certificate(ca)?;
    }
    Ok(pairing)
}

pub fn resolve(path: Option<&Path>) -> Result<super::profiles::Selection, super::profiles::Error> {
    let path = connection_path(path, std::env::var_os("DOKIME_REMOTE"))?;
    selection(&read(&path)?)
}

pub fn selection(pairing: &PairingFile) -> Result<super::profiles::Selection, super::profiles::Error> {
    pairing.validate().map_err(super::profiles::Error::Invalid)?;
    let target = super::Target::remote(&pairing.endpoints[0], pairing.certificate_pem.clone())
        .and_then(|target| target.with_endpoints(&pairing.endpoints))
        .map_err(|error| super::profiles::Error::Invalid(error.to_string()))?;
    Ok(super::profiles::Selection {
        identifier: None,
        target,
        expected_backend_id: Some(pairing.backend_identifier.clone()),
        access_key: pairing.access_key.clone().map(zeroize::Zeroizing::new),
    })
}

/* === Private Functions === */

fn connection_path(path: Option<&Path>, environment: Option<std::ffi::OsString>) -> Result<std::path::PathBuf, super::profiles::Error> {
    path.map(Path::to_owned)
        .or_else(|| environment.filter(|path| !path.is_empty()).map(Into::into))
        .ok_or_else(|| super::profiles::Error::Invalid("name a .pair file or set DOKIME_REMOTE".into()))
}

pub(crate) fn validate_certificate(pem: &str) -> Result<(), super::profiles::Error> {
    let mut roots = rustls::RootCertStore::empty();
    for certificate in CertificateDer::pem_slice_iter(pem.as_bytes()) {
        let certificate = certificate.map_err(|_| super::profiles::Error::Invalid("invalid server certificate PEM".into()))?;
        roots.add(certificate).map_err(|_| super::profiles::Error::Invalid("invalid server certificate DER".into()))?;
    }
    if roots.is_empty() {
        return Err(super::profiles::Error::Invalid("server certificate PEM contains no certificates".into()));
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_pairing_precedes_environment_and_missing_input_is_rejected() {
        let explicit = Path::new("explicit.pair");
        assert_eq!(connection_path(Some(explicit), Some("environment.pair".into())).unwrap(), explicit);
        assert_eq!(connection_path(None, Some("environment.pair".into())).unwrap(), Path::new("environment.pair"));
        assert!(connection_path(None, None).is_err());
        assert!(connection_path(None, Some("".into())).is_err());
    }
}
