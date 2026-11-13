/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Desktop Pairing Files

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io::Read, path::Path};

// Third Party Imports
use rustls::pki_types::{CertificateDer, pem::PemObject};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

// Local Imports
use super::Error;

/* === Constants === */

const PAIRING_BYTES: u64 = 1_048_576;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// A Product-Owned Secret File, Populated By The Library Access Service.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pairing {
    pub version: u32,
    pub backend_identifier: String,
    pub label: String,
    pub endpoints: Vec<String>,
    pub certificate_pem: String,
    pub access_key: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Pairing {
    pub fn read(path: &Path) -> Result<Self, Error> {
        if path.extension().and_then(|extension| extension.to_str()) != Some("pair") {
            return Err(Error::Target("import a .pair file".into()));
        }
        let file = std::fs::File::open(path).map_err(|error| Error::Target(format!("read pairing: {error}")))?;
        let mut bytes = Vec::new();
        file.take(PAIRING_BYTES + 1).read_to_end(&mut bytes).map_err(|error| Error::Target(format!("read pairing: {error}")))?;
        if bytes.len() as u64 > PAIRING_BYTES {
            bytes.zeroize();
            return Err(Error::Target("pairing file exceeds its input limit".into()));
        }
        let result = serde_json::from_slice(&bytes).map_err(|_| Error::Target("decode a valid v1 pairing file".into()));
        bytes.zeroize();
        let pairing: Self = result?;
        pairing.validate()?;
        Ok(pairing)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.version != 1 || self.backend_identifier.is_empty() || self.endpoints.is_empty() || self.access_key.is_empty() {
            return Err(Error::Target("pairing requires v1, a host identity, an endpoint and a credential".into()));
        }
        for endpoint in &self.endpoints {
            let url = url::Url::parse(endpoint).map_err(|_| Error::Target("pairing endpoint is not a URL".into()))?;
            if url.scheme() != "https"
                || url.host().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || url.path() != "/"
                || url.query().is_some()
                || url.fragment().is_some()
            {
                return Err(Error::Target("pairing requires an HTTPS endpoint without credentials, a path or query".into()));
            }
        }
        let mut roots = rustls::RootCertStore::empty();
        for certificate in CertificateDer::pem_slice_iter(self.certificate_pem.as_bytes()) {
            let certificate = certificate.map_err(|_| Error::Target("pairing certificate PEM is invalid".into()))?;
            roots.add(certificate).map_err(|_| Error::Target("pairing certificate is invalid".into()))?;
        }
        if roots.is_empty() {
            return Err(Error::Target("pairing has no pinned certificate".into()));
        }
        self.authorization()?;
        Ok(())
    }

    pub(super) fn authorization(&self) -> Result<tonic::metadata::MetadataValue<tonic::metadata::Ascii>, Error> {
        format!("Bearer {}", self.access_key).parse().map_err(|_| Error::Target("pairing credential is invalid".into()))
    }
}

impl std::fmt::Debug for Pairing {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Pairing")
            .field("backend_identifier", &self.backend_identifier)
            .field("endpoints", &self.endpoints)
            .finish_non_exhaustive()
    }
}

impl Drop for Pairing {
    fn drop(&mut self) {
        self.access_key.zeroize();
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
