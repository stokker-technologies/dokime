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

// Third Party Imports

// Local Imports
use super::Map;
use crate::storage::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    id: String,
    version: u32,
    parameters: Option<Map>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl TryFrom<&crate::analysis::Selection> for Selection {
    type Error = Error;
    fn try_from(value: &crate::analysis::Selection) -> Result<Self> {
        valid_identity(&value.id, value.version)?;
        Ok(Self {
            id: value.id.clone(),
            version: value.version,
            parameters: Some(Map::try_from(&value.parameters)?),
        })
    }
}

impl TryFrom<Selection> for crate::analysis::Selection {
    type Error = Error;
    fn try_from(value: Selection) -> Result<Self> {
        valid_identity(&value.id, value.version)?;
        Ok(Self {
            id: value.id,
            version: value.version,
            parameters: value.parameters.ok_or(Error::Invalid("analysis parameters are missing"))?.try_into()?,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn valid_identity(id: &str, version: u32) -> Result<()> {
    if id.is_empty() || version == 0 {
        return Err(Error::Invalid("invalid analysis identity"));
    }
    Ok(())
}

/* === Tests === */
