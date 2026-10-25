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

const MAX_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

pub type Values = BTreeMap<String, BTreeMap<String, Value>>;

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Value {
    pub raw: String,

    #[serde(rename = "valueType")]
    pub value_type: String,

    pub unit: Option<String>,
}

/* === Enums === */

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
    file.take(MAX_BYTES as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid("remembered parameters exceed 1 MiB"));
    }
    let values = serde_json::from_slice(&bytes).map_err(|error| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid or unsupported remembered parameter format: {error}"))
    })?;
    validate(&values)?;
    Ok(Some(values))
}

pub fn save(path: &Path, values: &Values) -> std::io::Result<()> {
    validate(values)?;
    let bytes = serde_json::to_vec(values).map_err(std::io::Error::other)?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid("remembered parameters exceed 1 MiB"));
    }
    dokime::harmos::filesystem::publication::write(path, &bytes).map_err(Into::into)
}

/* === Private Functions === */

fn validate(values: &Values) -> std::io::Result<()> {
    let valid_id = |id: &str| !id.is_empty() && id.len() <= 256 && !id.chars().any(char::is_control);
    let count = values.values().map(BTreeMap::len).sum::<usize>();
    if values.len() > 4096
        || count > 4096
        || values.iter().any(|(template, parameters)| {
            !valid_id(template)
                || parameters.iter().any(|(id, value)| {
                    !valid_id(id)
                        || value.raw.len() > 8192
                        || !valid_id(&value.value_type)
                        || value.unit.as_ref().is_some_and(|unit| unit.len() > 256)
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
