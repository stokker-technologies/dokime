/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Resources, Result};

/* === Constants === */

pub const ID: &str = "dokime.secrets.read";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<dokime_protocol::context::ReadSecret, dokime_protocol::context::Secret>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: dokime_protocol::context::ReadSecret) -> Result<dokime_protocol::context::Secret> {
    resources.check_action_scope(&resources.execution, ID)?;
    let value = crate::client::secrets_store::read_secret(&resources.store, &request.identifier)?;
    Ok(dokime_protocol::context::Secret { value })
}

/* === Private Functions === */

/* === Tests === */
