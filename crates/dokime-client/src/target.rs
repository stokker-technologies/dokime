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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Eq)]
pub struct Target {
    endpoint: String,
    remote: bool,
    ca: Option<String>,
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
        if ca.is_some() && url.scheme() != "https" {
            return Err(invalid_target("CA certificates require HTTPS."));
        }
        if !remote && !loopback {
            return Err(invalid_target("Local endpoints require a literal loopback IP address."));
        }
        Ok(Self { endpoint: url.to_string(), remote, ca })
    }

    pub(super) async fn connect(&self) -> Result<tonic::transport::Channel> {
        let mut endpoint = Endpoint::from_shared(self.endpoint.clone())
            .map_err(|_| invalid_target("Invalid backend endpoint."))?
            .connect_timeout(Duration::from_secs(10))
            // HTTP/2 PINGs are the actual liveness bound: a black-holed
            // connection (no bytes flowing either direction, so no server
            // deadline or client timeout ever fires) is detected within
            // ~15s (one 10s keepalive interval, then a 5s wait for the PING
            // ACK) and the connection is torn down, failing any request in
            // flight on it. `keep_alive_while_idle` keeps that detection
            // running even between calls, so a connection that died while
            // idle is never handed a new request.
            .http2_keep_alive_interval(Duration::from_secs(10))
            .keep_alive_timeout(Duration::from_secs(5))
            .keep_alive_while_idle(true)
            // Every RPC on this channel shares one deadline too; the
            // backend's own per-route `Deadlines` layer already fails fast
            // (10s) for ordinary calls and allows up to 300s for uploads and
            // report generation (crates/dokime-server/src/grpc/deadline.rs).
            // Setting this below that longest server-side budget would let
            // the client sever a report generation the server was still
            // willing to finish, so it stays a generous backstop above the
            // server's own ceiling — not the primary bound, and no longer
            // the thing standing between a hung remote backend and the user
            // (keepalive now is).
            .timeout(Duration::from_secs(310));
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
