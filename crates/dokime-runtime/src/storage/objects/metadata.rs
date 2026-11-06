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
use crate::storage::{Error, Result, values};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Row {
    pub(crate) identifier: String,
    pub(crate) label: String,
    pub(crate) description: Option<String>,
    pub(crate) tags: Vec<String>,

    #[serde(with = "values::map")]
    pub(crate) attributes: Vec<values::Cell>,

    pub(crate) created_at: Option<String>,
    pub(crate) updated_at: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl TryFrom<&Metadata> for Row {
    type Error = Error;

    fn try_from(value: &Metadata) -> Result<Self> {
        let mut tags = Vec::new();
        tags.try_reserve_exact(value.tags.len()).map_err(|_| Error::Invalid("metadata tag allocation failed"))?;
        tags.extend(value.tags.iter().map(String::as_str));
        tags.sort_unstable();
        if tags.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(Error::Invalid("metadata tags must be unique"));
        }
        drop(tags);

        Ok(Self {
            identifier: value.identifier.as_str().to_owned(),
            label: value.label.clone(),
            description: value.description.clone(),
            tags: value.tags.clone(),
            attributes: values::encode(&value.attributes)?,
            created_at: value.created_at.map(values::encode_datetime).transpose()?,
            updated_at: value.updated_at.map(values::encode_datetime).transpose()?,
        })
    }
}

impl TryFrom<Row> for Metadata {
    type Error = Error;

    fn try_from(value: Row) -> Result<Self> {
        for timestamp in [value.created_at.as_deref(), value.updated_at.as_deref()].into_iter().flatten() {
            values::validate_datetime(timestamp)?;
        }
        // Tags Remain In Authored Order
        let mut tags = Vec::new();
        tags.try_reserve_exact(value.tags.len()).map_err(|_| Error::Invalid("metadata tag allocation failed"))?;
        tags.extend(value.tags.iter().map(String::as_str));
        tags.sort_unstable();
        if tags.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(Error::Invalid("metadata tags must be unique"));
        }
        drop(tags);

        Ok(Self {
            identifier: Identifier::new(value.identifier)?,
            label: value.label,
            description: value.description,
            tags: value.tags,
            attributes: values::decode_owned(value.attributes)?,
            created_at: value.created_at.map(|at| Timestamp::new(at).to_datetime()).transpose()?,
            updated_at: value.updated_at.map(|at| Timestamp::new(at).to_datetime()).transpose()?,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
