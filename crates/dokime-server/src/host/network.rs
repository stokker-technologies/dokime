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

// Local Imports
use super::NetworkArgs;
use crate::grpc::Config;

/* === Constants === */

const SERVER_ALLOWED_ORIGINS: u64 = 32;
const SERVER_LISTEN_PORT: u64 = 8_765;
const SERVER_MAXIMUM_MESSAGE_BYTES: u64 = 16_777_216;
const SERVER_MESSAGE_BYTES: u64 = 1_048_576;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl NetworkArgs {
    pub fn config(&self) -> crate::host::Result<Config> {
        let bytes = match self.message_limit_mib {
            Some(mib) => mib.checked_mul(1024 * 1024).ok_or_else(|| input("message limit exceeds the native address space"))?,
            None => SERVER_MESSAGE_BYTES as usize,
        };
        if bytes == 0 || bytes as u64 > SERVER_MAXIMUM_MESSAGE_BYTES {
            return Err(input("message limit must be positive and within the configured maximum"));
        }
        if !self.host.is_loopback() {
            return Err(input("the native listener requires loopback; configure remote access in desktop settings"));
        }
        if self.allowed_origins.len() as u64 > SERVER_ALLOWED_ORIGINS {
            return Err(input("browser origins exceed the configured maximum"));
        }
        let origins = self
            .allowed_origins
            .iter()
            .map(|value| {
                dokime::harmos::grpc::Origin::parse(value)
                    .map_err(|_| input("browser origins require a scheme and authority without path, secrets or wildcard"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if origins.iter().enumerate().any(|(index, origin)| origins[..index].contains(origin)) {
            return Err(input("each browser origin may be allowed only once"));
        }
        let port = self.port.unwrap_or(SERVER_LISTEN_PORT as u16);
        let address = SocketAddr::new(self.host, port);
        let config = Config::local().with_bind_addr(address).with_max_message_bytes(bytes);
        config.validate()?;
        if self.grpc_web { Ok(config.with_grpc_web(origins)?) } else { Ok(config) }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn input(message: impl Into<String>) -> crate::host::Error {
    crate::host::Error::Input(message.into())
}

/* === Tests === */
