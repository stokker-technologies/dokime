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

pub async fn handle(_: &Context, _: FilesCaptureRequest) -> Result<FilesCaptureResponse> {
    Err(Error::Unimplemented(format!("{ID}: File Capture Is Not Implemented")))
}

/* === Private Functions === */

/* === Tests === */
