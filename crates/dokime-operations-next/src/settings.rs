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
    crate::declaration::<v1::ReadSettings, v1::SavedSettings>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: v1::ReadSettings) -> Result<v1::SavedSettings> {
    let _ = request;
    resources.check_action_scope(&resources.execution, ID)?;
    let snapshot = resources.store.cut(&resources.execution)?;
    let plugin = match snapshot.execution.origin.and_then(|origin| origin.kind) {
        Some(v1::origin::Kind::PluginRun(run)) => run.plugin,
        _ => return Err(crate::Error::Invalid("Read Settings: Use the Calling Plugin's Active Execution".into())),
    };
    let description = harmos_protocol::v1::Description::decode(resources.store.plugin(&plugin)?.as_slice())?;
    let settings = crate::client::settings_preferences::for_plugin(
        &resources.store,
        v1::Plugin {
            entry: plugin,
            description: Some(description),
            ..Default::default()
        },
    )?;
    Ok(v1::SavedSettings {
        values: settings.values.unwrap_or_default().values,
    })
}

/* === Private Functions === */

/* === Tests === */
