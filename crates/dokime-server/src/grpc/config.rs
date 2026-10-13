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
use std::{net::SocketAddr, num::NonZeroUsize, time::Duration};
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
    allowed_origins: Vec<harmos::grpc::Origin>,
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
            allowed_origins: Vec::new(),
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
            allowed_origins: Vec::new(),
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

    pub fn with_allowed_origins(
        mut self,
        origins: Vec<harmos::grpc::Origin>,
    ) -> Result<Self, Error> {
        if origins.len() > 32
            || origins
                .iter()
                .enumerate()
                .any(|(index, origin)| origins[..index].contains(origin))
        {
            return Err(Error::Configuration(
                "allowed origins require at most 32 distinct exact origins",
            ));
        }
        self.allowed_origins = origins;
        Ok(self)
    }

    pub(crate) fn into_listener_settings(mut self) -> Result<harmos::grpc::Settings, Error> {
        self.validate()?;
        let bytes = NonZeroUsize::new(self.max_message_bytes)
            .ok_or(Error::Configuration("message limit must be positive"))?;
        let stream = harmos::operations::StreamSettings {
            items: NonZeroUsize::new(1).unwrap(),
            bytes,
        };
        let tls = self.tls.take().map(|identity| {
            // Preserve The Host's Chosen TLS Provider When One Is Already Installed.
            let _ = rustls::crypto::ring::default_provider().install_default();
            tonic::transport::ServerTlsConfig::new()
                .identity(identity.into_tonic())
                .timeout(Duration::from_secs(10))
        });
        Ok(harmos::grpc::Settings {
            address: self.bind_addr,
            operations: harmos::grpc::server::Settings {
                calls: NonZeroUsize::new(32).unwrap(),
                decode: harmos::grpc::DecodeSettings {
                    message_bytes: bytes,
                    retained_bytes: bytes,
                },
                encode: harmos::grpc::EncodeSettings {
                    message_bytes: bytes,
                    buffer_bytes: NonZeroUsize::new(self.max_message_bytes.min(8192)).unwrap(),
                },
                input: stream,
                output: stream,
                opening_timeout: Duration::from_secs(5),
            },
            reflection: false,
            web: true,
            allowed_origins: self.allowed_origins,
            tls,
            connections: NonZeroUsize::new(64).unwrap(),
            calls_per_connection: NonZeroUsize::new(32).unwrap(),
            // Individual Operations Enforce Their Own Shorter Policies.
            // Do Not Cut Off A Valid Follow Stream At The Library's Ordinary Default.
            request_timeout: Duration::from_secs(86_400),
            shutdown_timeout: Duration::from_secs(5),
        })
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
