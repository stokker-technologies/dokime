/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{collections::BTreeMap, path::PathBuf};
// Third Party Imports
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginBundle {
    pub path: PathBuf,
    #[serde(default)]
    pub programs: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub provenance: PluginProvenance,
}

/* === Enums === */
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PluginProvenance {
    #[default]
    Configured,
    Installed {
        install_id: Uuid,
        sha256: String,
    },
}

/* === Traits === */

/* === Implementations === */
impl PluginBundle {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            programs: BTreeMap::new(),
            provenance: PluginProvenance::Configured,
        }
    }

    pub(crate) fn all(paths: &[PathBuf]) -> Vec<Self> {
        paths.iter().map(Self::new).collect()
    }

    pub fn configured(
        mut self,
        owner: impl Into<String>,
        config: impl Serialize,
    ) -> Result<Self, serde_json::Error> {
        self.programs
            .insert(owner.into(), serde_json::to_value(config)?);
        Ok(self)
    }
}

/* === Public Functions === */
pub(crate) fn empty_object() -> serde_json::Value {
    serde_json::Value::Object(Default::default())
}

/* === Private Functions === */

/* === Tests === */
