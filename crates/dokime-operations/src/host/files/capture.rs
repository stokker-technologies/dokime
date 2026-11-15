/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Capture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::Context;
use crate::host::files::captures::engine::{Capture, Mode};
use crate::{Error, Result};
use dokime_protocol::host::{FilesCaptureRequest, FilesCaptureResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Files/Capture";

/* === Macros === */

crate::native::unary!(FilesCaptureRequest, FilesCaptureResponse, context, "dokime.host.v1", "Files", "Capture");

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Copies the source once, as it is now; the execution owns the copy, so a cancelled reply leaves
/// sealing running and the execution waits for it before closing its evidence.
pub async fn handle(context: &Context, request: FilesCaptureRequest) -> Result<FilesCaptureResponse> {
    let caller = context.caller.as_ref().ok_or_else(|| Error::Permission(format!("{ID}: Call From a Plugin Execution")))?;
    let capture = Capture::open(context, ID, request.source.as_ref(), &request.output, Mode::Copy).await?;
    let identifier = context.captures.admit(capture)?;
    let sealed = context.captures.stop(&caller.execution, identifier).await?;
    Ok(FilesCaptureResponse {
        artifacts: sealed.artifacts,
        gaps: sealed.gaps,
    })
}

/* === Private Functions === */

/* === Tests === */
