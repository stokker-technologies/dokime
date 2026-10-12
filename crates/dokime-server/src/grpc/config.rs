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
use std::net::SocketAddr;
// Third Party Imports
use zeroize::Zeroizing;
// Local Imports
use super::{
    Error,
    access::{BearerGrant, Policy},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct TlsIdentity {
    certificate: Vec<u8>,
    private_key: Zeroizing<Vec<u8>>,
}

#[derive(Debug)]
pub struct Config {
    pub(crate) bind_addr: SocketAddr,
    pub(crate) policy: Policy,
    pub(crate) tls: Option<TlsIdentity>,
    pub(crate) max_message_bytes: usize,
    remote: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl std::fmt::Debug for TlsIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TlsIdentity")
            .finish_non_exhaustive()
    }
}

impl TlsIdentity {
    pub fn from_pem(certificate: Vec<u8>, private_key: Vec<u8>) -> Self {
        Self {
            certificate,
            private_key: Zeroizing::new(private_key),
        }
    }

    pub(crate) fn into_tonic(self) -> tonic::transport::Identity {
        tonic::transport::Identity::from_pem(&self.certificate, self.private_key.as_slice())
    }
}

impl Config {
    pub fn max_message_bytes(&self) -> usize {
        self.max_message_bytes
    }

    pub fn local() -> Self {
        Self {
            bind_addr: ([127, 0, 0, 1], 0).into(),
            policy: Policy::TrustedLocal,
            tls: None,
            max_message_bytes: 1024 * 1024,
            remote: false,
        }
    }

    pub fn remote(
        bind_addr: SocketAddr,
        tls: TlsIdentity,
        grants: Vec<BearerGrant>,
    ) -> Result<Self, Error> {
        Ok(Self {
            bind_addr,
            policy: Policy::bearer(grants)?,
            tls: Some(tls),
            max_message_bytes: 1024 * 1024,
            remote: true,
        })
    }

    pub fn with_bind_addr(mut self, address: SocketAddr) -> Self {
        self.bind_addr = address;
        self
    }

    pub fn with_bearer_auth(mut self, grants: Vec<BearerGrant>) -> Result<Self, Error> {
        self.policy = Policy::bearer(grants)?;
        Ok(self)
    }

    pub fn with_max_message_bytes(mut self, bytes: usize) -> Self {
        self.max_message_bytes = bytes;
        self
    }

    pub(crate) fn local_only(&self) -> bool {
        !self.remote
    }

    pub(crate) fn validate(&self) -> Result<(), Error> {
        if !self.remote && !self.bind_addr.ip().is_loopback() {
            return Err(Error::Configuration(
                "non-loopback listeners require explicit remote configuration",
            ));
        }
        if self.remote && (self.tls.is_none() || !matches!(self.policy, Policy::Bearer(_))) {
            return Err(Error::Configuration(
                "remote listeners require TLS and bearer authentication",
            ));
        }
        if self.max_message_bytes == 0 || self.max_message_bytes > 16 * 1024 * 1024 {
            return Err(Error::Configuration(
                "message limit must be between 1 byte and 16 MiB",
            ));
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::local()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
