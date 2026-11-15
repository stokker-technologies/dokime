/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Explicit Host Environment Files

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::path::Path;

// Third Party Imports

// Local Imports
use crate::error::Error;

/* === Constants === */

/// Variables the Host Reads; Any Other Name in an Environment File Is Refused.
pub(crate) const VARIABLES: &[&str] = &["RUST_LOG"];
const MAXIMUM_BYTES: u64 = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Process Variables Take Precedence Over an Explicitly Selected File; Nothing Is Searched Implicitly.
#[derive(Debug, Default)]
pub(crate) struct Environment(BTreeMap<String, String>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Environment {
    pub(crate) fn read(path: Option<&Path>) -> crate::Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        let invalid = |message: String| Error::Invalid(format!("Read Environment File {}: {message}", path.display()));
        let metadata = std::fs::metadata(path).map_err(|error| invalid(error.to_string()))?;
        if metadata.len() > MAXIMUM_BYTES {
            return Err(invalid("Select a File of at Most 1 MiB".into()));
        }
        let text = std::fs::read_to_string(path).map_err(|error| invalid(error.to_string()))?;
        let mut values = BTreeMap::new();
        for (index, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (name, value) = line.split_once('=').ok_or_else(|| invalid(format!("Write Line {} as NAME=VALUE", index + 1)))?;
            let name = name.trim();
            if !VARIABLES.contains(&name) {
                return Err(invalid(format!("Remove Unknown Variable {name} on Line {}", index + 1)));
            }
            let value = value.trim();
            let value = ['"', '\'']
                .iter()
                .find_map(|quote| value.strip_prefix(*quote).and_then(|value| value.strip_suffix(*quote)))
                .unwrap_or(value);
            if values.insert(name.to_owned(), value.to_owned()).is_some() {
                return Err(invalid(format!("Remove the Repeated Variable {name} on Line {}", index + 1)));
            }
        }
        Ok(Self(values))
    }

    pub(crate) fn get(&self, name: &str) -> Option<String> {
        std::env::var(name).ok().or_else(|| self.0.get(name).cloned())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
