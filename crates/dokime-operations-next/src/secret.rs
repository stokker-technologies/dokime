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
use crate::{Error, Resources, Result};
use dokime_protocol::v1;

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
    resources.check_scope(&resources.execution)?;
    let mut declared = false;
    for section in resources.plugin.catalog.iter().flat_map(|catalog| &catalog.sections) {
        if section.r#type != "dokime.v1.SettingsDeclarations" {
            continue;
        }
        let declarations = v1::SettingsDeclarations::decode(section.payload.as_slice())?;
        declared |= declarations.settings.iter().any(|setting| {
            matches!(setting.kind, Some(v1::setting::Kind::Secret(_)))
                && setting.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.identifier)
        });
    }
    if !declared {
        return Err(Error::Invalid(format!("Read Secret: Declare Secret {:?} in the Calling Plugin", request.identifier)));
    }
    resources.check_action_scope(&resources.execution, ID)?;
    let value = crate::client::secrets_store::read_secret(&resources.store, &request.identifier)?;
    Ok(dokime_protocol::context::Secret { value })
}

/* === Private Functions === */

/* === Tests === */
