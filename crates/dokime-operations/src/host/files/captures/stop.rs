/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Stop File Capture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use crate::host::Context;
use crate::{Error, Result};
use dokime_protocol::host::files::{CapturesStopRequest, CapturesStopResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Captures/Stop";

/* === Macros === */

crate::native::unary!(CapturesStopRequest, CapturesStopResponse, context, "dokime.host.v1.files", "Captures", "Stop");

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(_: &Context, _: CapturesStopRequest) -> Result<CapturesStopResponse> {
    Err(Error::Unimplemented(format!("{ID}: File Capture Is Not Implemented")))
}

/* === Private Functions === */

/* === Tests === */
