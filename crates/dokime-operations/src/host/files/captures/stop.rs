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

/// Takes the final pass and seals once; a repeated stop returns the same outcome.
pub async fn handle(context: &Context, request: CapturesStopRequest) -> Result<CapturesStopResponse> {
    let caller = context.caller.as_ref().ok_or_else(|| Error::Permission(format!("{ID}: Call From a Plugin Execution")))?;
    context.captures.stop(&caller.execution, request.capture_id).await
}

/* === Private Functions === */

/* === Tests === */
