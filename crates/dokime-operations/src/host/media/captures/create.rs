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

use crate::host::Context;
use crate::{Error, Result};
use dokime_protocol::host::media::{CapturesCreateRequest, CapturesCreateResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.media.Captures/Create";

/* === Macros === */

crate::native::unary!(CapturesCreateRequest, CapturesCreateResponse, context, "dokime.host.v1.media", "Captures", "Create");

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(_: &Context, _: CapturesCreateRequest) -> Result<CapturesCreateResponse> {
    Err(Error::Unimplemented(format!("{ID}: Operation Is Not Implemented")))
}

/* === Private Functions === */

/* === Tests === */
