/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};

/* === Constants === */

const TEXT_BYTES: usize = 16 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn epoch(context: &Context, epoch: &str) -> Result<()> {
    if epoch != context.host.identity().epoch {
        return Err(Error::Invalid("Write Note: Refresh Backend Identity After Restart".into()));
    }
    Ok(())
}
pub(super) fn message(context: &Context, message: &str) -> Result<()> {
    if message.trim().is_empty() || message.len() > TEXT_BYTES.min(context.settings.message_bytes / 2) {
        return Err(Error::Invalid("Write Note: Supply Nonempty Text Within the Message Budget".into()));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
