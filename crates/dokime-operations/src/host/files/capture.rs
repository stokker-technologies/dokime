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

/// Copies the source once, as it is now; a cancelled reply leaves sealing running to completion.
pub async fn handle(context: &Context, request: FilesCaptureRequest) -> Result<FilesCaptureResponse> {
    let capture = Capture::open(context, ID, request.source.as_ref(), &request.output, Mode::Copy).await?;
    let sealed = tokio::spawn(capture.seal()).await.map_err(|error| Error::Unavailable(format!("{ID}: Retry the Copy: {error}")))??;
    Ok(FilesCaptureResponse {
        artifacts: sealed.artifacts,
        gaps: sealed.gaps,
    })
}

/* === Private Functions === */

/* === Tests === */
