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
use dokime_protocol::host::{EffectsCreateRequest, EffectsCreateResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Effects/Create";

/* === Macros === */

crate::native::unary!(EffectsCreateRequest, EffectsCreateResponse, context, "dokime.host.v1", "Effects", "Create");

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(_: &Context, _: EffectsCreateRequest) -> Result<EffectsCreateResponse> {
    Err(Error::Unimplemented(format!("{ID}: Operation Is Not Implemented")))
}

/* === Private Functions === */

/* === Tests === */
