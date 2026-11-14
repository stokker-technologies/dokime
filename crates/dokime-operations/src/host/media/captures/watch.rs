/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Unsupported Host Operation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use crate::host::{Context, Output};
use crate::{Error, Result};
use dokime_protocol::host::media::{CapturesWatchRequest, CapturesWatchResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.media.Captures/Watch";

/* === Macros === */

crate::native::download!(CapturesWatchRequest, CapturesWatchResponse, "dokime.host.v1.media", "Captures", "Watch");

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(_: &Context, _: CapturesWatchRequest) -> Result<Output<CapturesWatchResponse>> {
    Err(Error::Unimplemented(format!("{ID}: Operation Is Not Implemented")))
}

/* === Private Functions === */

/* === Tests === */
