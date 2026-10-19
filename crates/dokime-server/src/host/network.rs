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

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl NetworkArgs {
    pub fn config(&self) -> crate::host::Result<Config> {
        let bytes = self
            .message_limit_mib
            .checked_mul(1024 * 1024)
            .filter(|bytes| (1..=16 * 1024 * 1024).contains(bytes))
            .ok_or_else(|| input("message limit must be between 1 and 16 MiB"))?;
        if !self.host.is_loopback() {
            return Err(input("the native listener requires loopback; configure remote access in desktop settings"));
        }
        if self.allowed_origins.len() > 32 {
            return Err(input("at most 32 exact browser origins are allowed"));
        }
        let origins = self
            .allowed_origins
            .iter()
            .map(|value| {
                harmos::grpc::Origin::parse(value)
                    .map_err(|_| input("browser origins require a scheme and authority without path, credentials or wildcard"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if origins.iter().enumerate().any(|(index, origin)| origins[..index].contains(origin)) {
            return Err(input("each browser origin may be allowed only once"));
        }
        let address = SocketAddr::new(self.host, self.port);
        let config = Config::local().with_bind_addr(address).with_max_message_bytes(bytes);
        if self.grpc_web { Ok(config.with_grpc_web(origins)?) } else { Ok(config) }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn input(message: impl Into<String>) -> crate::host::Error {
    crate::host::Error::Input(message.into())
}

/* === Tests === */
