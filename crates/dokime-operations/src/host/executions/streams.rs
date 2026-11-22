/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Frozen Execution Stream Owners

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos_protocol::host::{PluginEntry, PluginState};
use harmos_protocol::model::Declarations;
use harmos_protocol::schema::{merge_message_types, message_types};

// Local Imports
use crate::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Streams {
    pub catalog: Declarations,
    pub owners: BTreeMap<String, Vec<String>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Resolve Through the Installed Plugin Inventory and Freeze the Selected Row Types.
pub fn resolve(plugin: &PluginEntry, plugins: &[PluginEntry], streams: &[String]) -> Result<Streams> {
    let mut catalog = plugin
        .description
        .as_ref()
        .and_then(|description| description.declarations.clone())
        .ok_or_else(|| Error::Precondition("Capture Execution Streams: Discover the Template Plugin Catalog".into()))?;
    let mut owners = BTreeMap::<String, Vec<String>>::new();
    for stream in streams {
        let mut declaring = plugins.iter().filter(|plugin| {
            plugin.state == PluginState::Ready as i32
                && plugin
                    .description
                    .as_ref()
                    .and_then(|description| description.declarations.as_ref())
                    .is_some_and(|catalog| catalog.streams.iter().any(|declaration| declaration.id == *stream))
        });
        let owner = declaring.next().ok_or_else(|| {
            Error::Precondition(format!("Capture Execution Stream {stream}: Install or Enable the Plugin That Declares It"))
        })?;
        if declaring.next().is_some() {
            return Err(Error::Precondition(format!(
                "Capture Execution Stream {stream}: Remove the Duplicate Stream Declaration From the Installed Plugins"
            )));
        }
        let selected = owners.entry(owner.name.clone()).or_default();
        if selected.contains(stream) {
            continue;
        }
        selected.push(stream.clone());
        if owner.name == plugin.name {
            continue;
        }
        let peer = owner.description.as_ref().unwrap().declarations.as_ref().unwrap();
        let declaration = peer.streams.iter().find(|declaration| declaration.id == *stream).unwrap();
        let types = peer
            .types
            .as_ref()
            .ok_or_else(|| Error::Precondition(format!("Freeze Execution Stream {stream}: Rebuild Its Plugin With Row Descriptors")))?;
        let rows = message_types(types, &declaration.row).map_err(|source| {
            Error::Precondition(format!("Freeze Execution Stream {stream}: Rebuild Its Plugin With Valid Row Descriptors: {source}"))
        })?;
        merge_message_types(catalog.types.get_or_insert_default(), rows).map_err(|source| {
            Error::Precondition(format!("Freeze Execution Stream {stream}: Rebuild the Plugins With Matching Row Descriptors: {source}"))
        })?;
        catalog.streams.push(declaration.clone());
    }
    Ok(Streams { catalog, owners })
}

/* === Private Functions === */

/* === Tests === */
