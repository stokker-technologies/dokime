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

use super::engine::{Capture, Mode};
use crate::Result;
use crate::host::Context;
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

/// Takes the baseline and returns; the calling execution owns the capture until it is stopped or ends.
pub async fn handle(context: &Context, request: CapturesStartRequest) -> Result<CapturesStartResponse> {
    let mode = Mode::of(ID, request.mode.as_ref())?;
    let capture = Capture::open(context, ID, request.source.as_ref(), &request.output, mode).await?;
    let capture_id = context.captures.admit(capture)?;
    Ok(CapturesStartResponse { capture_id })
}

/* === Private Functions === */

/* === Tests === */
