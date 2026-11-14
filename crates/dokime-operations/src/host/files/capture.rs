/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Capture

use crate::host::Context;
use crate::{Error, Result};
use dokime_protocol::host::{FilesCaptureRequest, FilesCaptureResponse};

pub const ID: &str = "dokime.host.v1.Files/Capture";

pub async fn handle(_: &Context, _: FilesCaptureRequest) -> Result<FilesCaptureResponse> {
    Err(Error::Unimplemented(format!("{ID}: File Capture Is Not Implemented")))
}

crate::native::unary!(FilesCaptureRequest, FilesCaptureResponse, context, "dokime.host.v1", "Files", "Capture");
