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
impl Selection {
    pub fn encoding_memory(value: &crate::analysis::Selection) -> Result<usize> {
        if value.id.is_empty() || value.version == 0 {
            return Err(Error::Invalid("invalid analysis identity"));
        }
        let parameters = Map::encoding_memory(&value.parameters)?;
        Ok(std::mem::size_of::<Self>()
            .saturating_add(value.id.len())
            .saturating_add(parameters.saturating_sub(std::mem::size_of::<Map>())))
    }

    pub fn projection_memory(&self) -> Result<usize> {
        if self.id.is_empty() || self.version == 0 {
            return Err(Error::Invalid("invalid analysis identity"));
        }
        let parameters = self
            .parameters
            .as_ref()
            .ok_or(Error::Invalid("analysis parameters are missing"))?
            .projection_memory()?;
        Ok(std::mem::size_of::<crate::analysis::Selection>()
            .saturating_add(self.id.capacity())
            .saturating_add(parameters))
    }
}

impl TryFrom<&crate::analysis::Selection> for Selection {
    type Error = Error;
    fn try_from(value: &crate::analysis::Selection) -> Result<Self> {
        Self::encoding_memory(value)?;
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
        value.projection_memory()?;
        Ok(Self {
            id: value.id,
            version: value.version,
            parameters: value
                .parameters
                .ok_or(Error::Invalid("analysis parameters are missing"))?
                .try_into()?,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
