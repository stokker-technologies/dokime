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
use std::{collections::BTreeMap, io::Read, path::Path};

// Third Party Imports

// Local Imports

/* === Constants === */

const DESKTOP_PARAMETER_BYTES: u64 = 1_048_576;
const DESKTOP_PARAMETER_IDENTIFIER_BYTES: u64 = 256;
const DESKTOP_PARAMETER_ITEMS: u64 = 4_096;
const DESKTOP_PARAMETER_VALUE_BYTES: u64 = 8_192;

/* === Macros === */

/* === Type Aliases === */

pub type Values = BTreeMap<String, BTreeMap<String, Value>>;

/* === Structs === */

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Value {
    Legacy(String),
    Defined {
        raw: String,

        #[serde(rename = "valueType")]
        value_type: String,

        unit: Option<String>,
    },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn load(path: &Path) -> std::io::Result<Option<Values>> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(DESKTOP_PARAMETER_BYTES.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > DESKTOP_PARAMETER_BYTES {
        return Err(invalid("remembered parameters exceed the configured byte budget"));
    }
    let values = serde_json::from_slice(&bytes).map_err(std::io::Error::other)?;
    validate(&values)?;
    Ok(Some(values))
}

pub fn save(path: &Path, values: &Values) -> std::io::Result<()> {
    validate(values)?;
    let bytes = serde_json::to_vec(values).map_err(std::io::Error::other)?;
    if bytes.len() as u64 > DESKTOP_PARAMETER_BYTES {
        return Err(invalid("remembered parameters exceed the configured byte budget"));
    }
    dokime::harmos::filesystem::publication::write(path, &bytes).map_err(Into::into)
}

/* === Private Functions === */

fn validate(values: &Values) -> std::io::Result<()> {
    let valid_id = |id: &str| !id.is_empty() && id.len() as u64 <= DESKTOP_PARAMETER_IDENTIFIER_BYTES && !id.chars().any(char::is_control);
    let count = values.values().map(BTreeMap::len).sum::<usize>();
    if values.len() as u64 > DESKTOP_PARAMETER_ITEMS
        || count as u64 > DESKTOP_PARAMETER_ITEMS
        || values.iter().any(|(template, parameters)| {
            !valid_id(template)
                || parameters.iter().any(|(id, value)| {
                    let (raw, metadata_valid) = match value {
                        Value::Legacy(raw) => (raw, true),
                        Value::Defined { raw, value_type, unit } => (
                            raw,
                            valid_id(value_type)
                                && unit.as_ref().is_none_or(|unit| unit.len() as u64 <= DESKTOP_PARAMETER_IDENTIFIER_BYTES),
                        ),
                    };
                    !valid_id(id) || raw.len() as u64 > DESKTOP_PARAMETER_VALUE_BYTES || !metadata_valid
                })
        })
    {
        return Err(invalid("invalid remembered parameter values"));
    }
    Ok(())
}

fn invalid(message: &'static str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}

/* === Tests === */
