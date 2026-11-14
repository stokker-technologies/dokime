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
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.settings.read";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<dokime_protocol::context::ReadSettings, dokime_protocol::context::SavedSettings>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(
    resources: &Resources, request: dokime_protocol::context::ReadSettings,
) -> Result<dokime_protocol::context::SavedSettings> {
    let _ = request;
    resources.check_action_scope(&resources.execution, ID)?;
    let snapshot = resources.store.cut(&resources.execution)?;
    let plugin = match snapshot.execution.origin.and_then(|origin| origin.kind) {
        Some(v1::origin::Kind::PluginRun(run)) => run.plugin,
        _ => return Err(crate::Error::Invalid("Read Settings: Use the Calling Plugin's Active Execution".into())),
    };
    let description = harmos_protocol::plugin::InstanceGetResponse::decode(resources.store.plugin(&plugin)?.as_slice())?;
    let settings = crate::client::settings_preferences::for_plugin(
        &resources.store,
        dokime_protocol::client::Plugin {
            entry: plugin,
            description: Some(description),
            ..Default::default()
        },
    )?;
    Ok(dokime_protocol::context::SavedSettings {
        values: settings.value.unwrap_or_default().values,
    })
}

/* === Private Functions === */

/* === Tests === */
