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
use std::{io::Read, path::Path, time::Duration};
// Third Party Imports
use tonic::{
    Request,
    metadata::{Ascii, MetadataValue},
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint},
};
// Local Imports
use crate::error::Error;
use dokime_protocol::v1::{self, plugins_client::PluginsClient};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Client {
    inner: PluginsClient<Channel>,
    token: Option<MetadataValue<Ascii>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Client {
    pub(super) async fn connect(
        url: &url::Url,
        token_file: Option<&Path>,
        ca_cert: Option<&Path>,
    ) -> Result<Self, Error> {
        let mut endpoint =
            Endpoint::from_shared(url.to_string())?.connect_timeout(Duration::from_secs(10));
        if url.scheme() == "https" {
            let mut tls = ClientTlsConfig::new().with_native_roots();
            if let Some(path) = ca_cert {
                tls = tls.ca_certificate(Certificate::from_pem(read(path, 1024 * 1024)?));
            }
            endpoint = endpoint.tls_config(tls)?;
        } else if ca_cert.is_some() {
            return Err(Error::Invalid(
                "--ca-cert requires an HTTPS endpoint".into(),
            ));
        }
        let token = token_file
            .map(|path| {
                let bytes = read(path, 16 * 1024)?;
                let token = std::str::from_utf8(&bytes)
                    .map_err(|_| Error::Invalid("token file must contain UTF-8 text".into()))?
                    .trim();
                if token.is_empty() || token.bytes().any(|byte| byte.is_ascii_whitespace()) {
                    return Err(Error::Invalid(
                        "token file must contain one nonempty token".into(),
                    ));
                }
                let mut value =
                    MetadataValue::try_from(format!("Bearer {token}")).map_err(|_| {
                        Error::Invalid("token file contains invalid metadata bytes".into())
                    })?;
                value.set_sensitive(true);
                Ok(value)
            })
            .transpose()?;
        Ok(Self {
            inner: PluginsClient::new(endpoint.connect().await?),
            token,
        })
    }

    fn request<T>(&self, body: T, seconds: u64) -> Request<T> {
        let mut request = Request::new(body);
        request.set_timeout(Duration::from_secs(seconds));
        if let Some(token) = &self.token {
            request
                .metadata_mut()
                .insert("authorization", token.clone());
        }
        request
    }

    pub(super) async fn stage(
        &mut self,
        bytes: Vec<u8>,
        sha256: String,
    ) -> Result<v1::PluginInstallation, Error> {
        let size = bytes.len() as u64;
        let frames = futures_util::stream::unfold(
            (bytes, 0usize, Some(sha256)),
            move |(bytes, offset, header)| async move {
                let (frame, next) = if let Some(sha256) = header {
                    (
                        v1::stage_plugin_frame::Frame::Header(v1::StagePluginHeader {
                            size_bytes: size,
                            sha256,
                        }),
                        offset,
                    )
                } else if offset < bytes.len() {
                    let next = (offset + 256 * 1024).min(bytes.len());
                    (
                        v1::stage_plugin_frame::Frame::Chunk(v1::StagePluginChunk {
                            offset: offset as u64,
                            data: bytes[offset..next].to_vec(),
                        }),
                        next,
                    )
                } else {
                    return None;
                };
                Some((
                    v1::StagePluginFrame { frame: Some(frame) },
                    (bytes, next, None),
                ))
            },
        );
        let request = self.request(frames, 300);
        Ok(self.inner.stage(request).await?.into_inner())
    }

    pub(super) async fn consent(
        &mut self,
        id: &str,
        digest: &str,
    ) -> Result<v1::PluginInstallation, Error> {
        let request = self.request(
            v1::ConsentPluginRequest {
                install_id: id.into(),
                expected_sha256: digest.into(),
            },
            10,
        );
        Ok(self.inner.consent(request).await?.into_inner())
    }

    pub(super) async fn remove(&mut self, id: &str) -> Result<(), Error> {
        let request = self.request(
            v1::RemovePluginRequest {
                install_id: id.into(),
            },
            10,
        );
        let reply = self.inner.remove(request).await?.into_inner();
        if reply.installation.is_some() {
            return Err(Error::Invalid("installation remains authorized or pending removal; staged cleanup was not confirmed".into()));
        }
        Ok(())
    }
}

/* === Public Functions === */
pub(super) fn endpoint(value: &str) -> Result<url::Url, Error> {
    let url = url::Url::parse(value)
        .map_err(|error| Error::Invalid(format!("invalid backend endpoint: {error}")))?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
        || url.host_str().is_none()
    {
        return Err(Error::Invalid(
            "backend endpoint must be an HTTP(S) origin without credentials, path, query, or fragment".into(),
        ));
    }
    let loopback = match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain(name)) => name.eq_ignore_ascii_case("localhost"),
        None => false,
    };
    if url.scheme() == "http" && !loopback {
        return Err(Error::Invalid(
            "remote backend endpoints require HTTPS".into(),
        ));
    }
    Ok(url)
}

/* === Private Functions === */
fn read(path: &Path, limit: usize) -> Result<Vec<u8>, Error> {
    let file = std::fs::File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(Error::Invalid(
            "connection material must be a regular file".into(),
        ));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(Error::Output("connection material".into(), limit));
    }
    Ok(bytes)
}

/* === Tests === */
