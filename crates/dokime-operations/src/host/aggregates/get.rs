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
use dokime_protocol::host::{AggregatesGetRequest, AggregatesGetResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Aggregates/Get";

/* === Macros === */

crate::native::unary!(AggregatesGetRequest, AggregatesGetResponse, context, "dokime.host.v1", "Aggregates", "Get");

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(_: &Context, _: AggregatesGetRequest) -> Result<AggregatesGetResponse> {
    Err(Error::Unimplemented(format!("{ID}: Operation Is Not Implemented")))
}

/* === Private Functions === */

/* === Tests === */
