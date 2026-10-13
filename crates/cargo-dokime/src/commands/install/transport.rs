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

// Local Imports
use crate::error::Error;
use dokime_client::{Target, contracts::plugins};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Client {
    inner: dokime_client::Client,
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
        if ca_cert.is_some() && url.scheme() != "https" {
            return Err(Error::Invalid(
                "--ca-cert requires an HTTPS endpoint".into(),
            ));
        }
        let ca = ca_cert
            .map(|path| {
                String::from_utf8(read(path, 1024 * 1024)?).map_err(|_| {
                    Error::Invalid("CA certificate must contain UTF-8 PEM text".into())
                })
            })
            .transpose()?;
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
                Ok(token.to_owned())
            })
            .transpose()?;
        let target = Target::new(url.to_string(), ca)?;
        let inner = dokime_client::Client::connect(target, token.as_deref(), None).await?;
        Ok(Self { inner })
    }

    pub(super) async fn stage(
        &mut self,
        bytes: Vec<u8>,
        sha256: String,
    ) -> Result<plugins::PluginInstallation, Error> {
        let size = bytes.len() as u64;
        let frames = futures_util::stream::unfold(
            (bytes, 0usize, Some(sha256)),
            move |(bytes, offset, header)| async move {
                let (frame, next) = if let Some(sha256) = header {
                    (
                        plugins::StagePluginFrameFrame::Header(plugins::StagePluginHeader {
                            size_bytes: size,
                            sha256,
                        }),
                        offset,
                    )
                } else if offset < bytes.len() {
                    let next = (offset + 256 * 1024).min(bytes.len());
                    (
                        plugins::StagePluginFrameFrame::Chunk(plugins::StagePluginChunk {
                            offset: offset as u64,
                            data: bytes[offset..next].to_vec(),
                        }),
                        next,
                    )
                } else {
                    return None;
                };
                Some((
                    plugins::StagePluginFrame { frame: Some(frame) },
                    (bytes, next, None),
                ))
            },
        );
        let installation = self
            .inner
            .with_timeout(Duration::from_secs(300))
            .plugins_stage(frames)
            .await?;
        Ok(installation)
    }

    pub(super) async fn consent(
        &mut self,
        id: &str,
        digest: &str,
    ) -> Result<plugins::PluginInstallation, Error> {
        let request = plugins::ConsentPluginRequest {
            install_id: id.into(),
            expected_sha256: digest.into(),
        };
        let installation = self
            .inner
            .with_timeout(Duration::from_secs(10))
            .plugins_consent(request)
            .await?;
        Ok(installation)
    }

    pub(super) async fn remove(&mut self, id: &str) -> Result<(), Error> {
        let request = plugins::RemovePluginRequest {
            install_id: id.into(),
        };
        let reply = self
            .inner
            .with_timeout(Duration::from_secs(10))
            .plugins_remove(request)
            .await?;
        if reply.installation.is_some() {
            return Err(Error::Invalid("installation remains authorized or pending removal; staged cleanup was not confirmed".into()));
        }
        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.inner.close();
    }
}

/* === Public Functions === */
pub(super) fn endpoint(value: &str) -> Result<url::Url, Error> {
    Target::new(value, None)?;
    url::Url::parse(value)
        .map_err(|error| Error::Invalid(format!("invalid backend endpoint: {error}")))
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
