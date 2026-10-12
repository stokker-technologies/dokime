/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
include!(concat!(env!("OUT_DIR"), "/runtime-operations.rs"));

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, time::Duration};

// Third Party Imports
use harmos::grpc::{
    DecodeSettings, EncodeSettings,
    server::{Authority, Settings},
};
use harmos::operations::StreamSettings;

// Local Imports
use super::Error;
use dokime_app::RuntimeHandle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn runtime(
    handle: &RuntimeHandle,
    message_bytes: usize,
) -> Result<RuntimeServer, Error> {
    let bytes = NonZeroUsize::new(message_bytes).ok_or(Error::Configuration(
        "operation message limit must be positive",
    ))?;
    let stream = StreamSettings {
        items: NonZeroUsize::new(1).unwrap(),
        bytes,
    };
    let settings = Settings {
        calls: NonZeroUsize::new(32).unwrap(),
        decode: DecodeSettings {
            message_bytes: bytes,
            retained_bytes: bytes,
        },
        encode: EncodeSettings {
            message_bytes: bytes,
            buffer_bytes: NonZeroUsize::new(message_bytes.min(8192)).unwrap(),
        },
        input: stream,
        output: stream,
        opening_timeout: Duration::from_secs(5),
    };
    RuntimeServer::new(
        Authority::TrustedHost(handle.operations().clone()),
        settings,
    )
    .map_err(Error::OperationService)
}

/* === Private Functions === */

/* === Tests === */
