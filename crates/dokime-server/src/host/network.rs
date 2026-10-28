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
        let port = self.port.unwrap_or(SERVER_LISTEN_PORT as u16);
        let address = SocketAddr::new(self.host, port);
        let config = Config::local().with_bind_addr(address).with_max_message_bytes(bytes);
        config.validate()?;
        Ok(config)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn input(message: impl Into<String>) -> crate::host::Error {
    crate::host::Error::Input(message.into())
}

/* === Tests === */
