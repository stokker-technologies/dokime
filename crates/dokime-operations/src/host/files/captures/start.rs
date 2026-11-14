/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Start File Capture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use crate::host::Context;
use crate::{Error, Result};
use dokime_protocol::host::files::{CapturesStartRequest, CapturesStartResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Captures/Start";

/* === Macros === */

crate::native::unary!(CapturesStartRequest, CapturesStartResponse, context, "dokime.host.v1.files", "Captures", "Start");

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(_: &Context, _: CapturesStartRequest) -> Result<CapturesStartResponse> {
    Err(Error::Unimplemented(format!("{ID}: File Capture Is Not Implemented")))
}

/* === Private Functions === */

/* === Tests === */
