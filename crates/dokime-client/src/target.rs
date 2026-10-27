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
use std::time::Duration;

// Third Party Imports
use tonic::transport::{Certificate, ClientTlsConfig, Endpoint};
use url::{Host, Url};

// Local Imports
use super::{Error, Result};
use dokime_domain::model::settings::application::ClientPreferences;

/* === Constants === */

const CA_BYTES: usize = 1_048_576;
const CONNECTION_WINDOW_BYTES: u64 = 16_777_216;
const KEEPALIVE_INTERVAL_MS: u64 = 10_000;
const KEEPALIVE_TIMEOUT_MS: u64 = 5_000;
const STREAM_WINDOW_BYTES: u64 = 4_194_304;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Eq)]
pub struct Target {
    endpoint: String,
    remote: bool,
    ca: Option<String>,
    settings: ClientPreferences,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Target {
    pub fn new(endpoint: impl Into<String>, ca_certificate_pem: Option<String>) -> Result<Self> {
        Self::parse(endpoint.into(), None, ca_certificate_pem)
    }

    pub fn local(endpoint: impl Into<String>) -> Result<Self> {
        Self::parse(endpoint.into(), Some(false), None)
    }

    pub fn remote(endpoint: impl Into<String>, ca_certificate_pem: Option<String>) -> Result<Self> {
        Self::parse(endpoint.into(), Some(true), ca_certificate_pem)
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
    pub fn is_remote(&self) -> bool {
        self.remote
    }

    pub fn with_settings(mut self, settings: ClientPreferences) -> Self {
        self.settings = settings;
        self
    }

    pub fn settings(&self) -> &ClientPreferences {
        &self.settings
    }

    fn parse(endpoint: String, remote: Option<bool>, ca: Option<String>) -> Result<Self> {
        let mut url = Url::parse(&endpoint).map_err(|_| invalid_target("An absolute backend endpoint is required."))?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid_target("Use an HTTP(S) origin without user information, path, query, or fragment."));
        }
        // Preserve the public Python client's localhost spelling without DNS.
        if remote.is_none() && url.scheme() == "http" && url.host_str() == Some("localhost") {
            url.set_host(Some("127.0.0.1")).map_err(|_| invalid_target("Invalid loopback endpoint."))?;
        }
        let loopback = matches!(url.host(), Some(Host::Ipv4(ip)) if ip.is_loopback())
            || matches!(url.host(), Some(Host::Ipv6(ip)) if ip.is_loopback());
        let remote = remote.unwrap_or(url.scheme() == "https" || !loopback);
        if ca.as_ref().is_some_and(|pem| pem.is_empty() || pem.len() > CA_BYTES || pem.contains('\0')) {
            return Err(invalid_target("Invalid CA certificate."));
        }
        if let Some(pem) = &ca {
            super::pairing::validate_ca(pem).map_err(|_| invalid_target("Invalid CA certificate."))?;
        }
        if ca.is_some() && url.scheme() != "https" {
            return Err(invalid_target("CA certificates require HTTPS."));
        }
        if !remote && !loopback {
            return Err(invalid_target("Local endpoints require a literal loopback IP address."));
        }
        Ok(Self { endpoint: url.to_string(), remote, ca, settings: ClientPreferences::default() })
    }

    pub(super) async fn connect(&self) -> Result<tonic::transport::Channel> {
        self.settings.validate().map_err(|_| invalid_target("Invalid client configuration."))?;
        let mut endpoint = Endpoint::from_shared(self.endpoint.clone())
            .map_err(|_| invalid_target("Invalid backend endpoint."))?
            .connect_timeout(Duration::from_millis(self.settings.connect_timeout_ms))
            .http2_keep_alive_interval(Duration::from_millis(KEEPALIVE_INTERVAL_MS))
            .keep_alive_timeout(Duration::from_millis(KEEPALIVE_TIMEOUT_MS))
            .keep_alive_while_idle(true)
            .initial_stream_window_size(STREAM_WINDOW_BYTES as u32)
            .initial_connection_window_size(CONNECTION_WINDOW_BYTES as u32);
        if self.endpoint.starts_with("https:") {
            // Select Ring when the host has not
            // already selected a process provider, as the native server does.
            let _ = rustls::crypto::ring::default_provider().install_default();
            let mut tls = ClientTlsConfig::new().with_native_roots();
            if let Some(ca) = &self.ca {
                tls = tls.ca_certificate(Certificate::from_pem(ca));
            }
            endpoint = endpoint.tls_config(tls).map_err(Error::Transport)?;
        }
        endpoint.connect().await.map_err(Error::Transport)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn invalid_target(message: &'static str) -> Error {
    Error::InvalidTarget { message }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_transport_policy() {
        for (endpoint, remote) in [
            ("http://localhost:8765", false),
            ("http://127.0.0.1:8765", false),
            ("http://[::1]:8765", false),
            ("http://lab.example:8765", true),
            ("http://192.0.2.1:8765", true),
            ("https://127.0.0.1:8765", true),
            ("https://lab.example:8765", true),
        ] {
            assert_eq!(Target::new(endpoint, None).unwrap().is_remote(), remote, "{endpoint}");
        }
        assert_eq!(Target::new("http://localhost:8765", None).unwrap().endpoint(), "http://127.0.0.1:8765/");
        assert!(Target::remote("http://127.0.0.1:8765", None).unwrap().is_remote());
        assert!(Target::local("http://lab.example:8765").is_err());
        assert!(Target::new("http://lab.example:8765", Some("certificate".into())).is_err());
        for endpoint in
            ["https://user:password@lab.example", "http://lab.example/path", "http://lab.example/?key=value", "ftp://lab.example"]
        {
            assert!(Target::new(endpoint, None).is_err(), "{endpoint}");
        }
    }
}
