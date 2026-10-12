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
use std::{net::SocketAddr, path::Path};
// Third Party Imports
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};
// Local Imports
use super::NetworkArgs;
use dokime_server::grpc::{
    Config, TlsIdentity,
    access::{BearerGrant, Role},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Authentication {
    grants: Vec<Grant>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Grant {
    subject: String,
    role: Role,
    token: String,
}

/* === Enums === */
/* === Traits === */
pub(super) trait NetworkConfig {
    fn config(&self) -> crate::Result<Config>;
}

/* === Implementations === */
impl NetworkConfig for NetworkArgs {
    fn config(&self) -> crate::Result<Config> {
        let bytes = self
            .message_limit_mib
            .checked_mul(1024 * 1024)
            .filter(|bytes| (1..=16 * 1024 * 1024).contains(bytes))
            .ok_or_else(|| input("message limit must be between 1 and 16 MiB"))?;
        if !self.allow_remote && !self.host.is_loopback() {
            return Err(input(
                "non-loopback listeners require --allow-remote, TLS and --auth-file",
            ));
        }
        let grants = self.auth_file.as_deref().map(read_grants).transpose()?;
        let address = SocketAddr::new(self.host, self.port);
        let config = if self.allow_remote {
            let certificate = crate::input::read(
                self.tls_cert
                    .as_deref()
                    .ok_or_else(|| input("--tls-cert is required"))?,
                1024 * 1024,
            )?;
            let key = crate::input::read(
                self.tls_key
                    .as_deref()
                    .ok_or_else(|| input("--tls-key is required"))?,
                64 * 1024,
            )?;
            Config::remote(
                address,
                TlsIdentity::from_pem(certificate, key),
                grants.ok_or_else(|| input("--auth-file is required"))?,
            )?
        } else {
            let config = Config::local().with_bind_addr(address);
            match grants {
                Some(grants) => config.with_bearer_auth(grants)?,
                None => config,
            }
        };
        Ok(config.with_max_message_bytes(bytes))
    }
}

impl Drop for Grant {
    fn drop(&mut self) {
        self.token.zeroize();
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn read_grants(path: &Path) -> crate::Result<Vec<BearerGrant>> {
    let bytes = Zeroizing::new(crate::input::read(path, 1024 * 1024)?);
    parse_grants(&bytes)
}

fn parse_grants(bytes: &[u8]) -> crate::Result<Vec<BearerGrant>> {
    let file: Authentication = serde_json::from_slice(bytes).map_err(|error| {
        input(format!(
            "invalid authentication file at line {}, column {}",
            error.line(),
            error.column()
        ))
    })?;
    if file.grants.is_empty() || file.grants.len() > 128 {
        return Err(input("authentication file requires 1 to 128 grants"));
    }
    file.grants
        .iter()
        .map(|grant| {
            BearerGrant::new(grant.subject.clone(), grant.role, &grant.token).map_err(Into::into)
        })
        .collect()
}

fn input(message: impl Into<String>) -> crate::Error {
    crate::Error::Input(message.into())
}

/* === Tests === */
