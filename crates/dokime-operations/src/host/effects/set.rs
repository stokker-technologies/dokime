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
use dokime_protocol::host::{EffectsSetRequest, EffectsSetResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Effects/Set";

/* === Macros === */

crate::native::unary!(EffectsSetRequest, EffectsSetResponse, context, "dokime.host.v1", "Effects", "Set");

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(_: &Context, _: EffectsSetRequest) -> Result<EffectsSetResponse> {
    Err(Error::Unimplemented(format!("{ID}: Operation Is Not Implemented")))
}

/* === Private Functions === */

/* === Tests === */
