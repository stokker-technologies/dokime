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
use serde::{
    Serialize,
    de::{self, DeserializeOwned},
};
use serde_json::{Value as Json, json};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn to_value(value: &impl Serialize) -> serde_json::Result<Json> {
    Ok(json!({"dokime_encoding": 2, "payload": serde_json::to_value(value)?}))
}

pub(crate) fn from_value<T: DeserializeOwned>(mut value: Json) -> serde_json::Result<T> {
    if value.get("dokime_encoding") != Some(&json!(2)) {
        return Err(de::Error::custom("unsupported Dokime persistence encoding"));
    }

    let payload = value
        .as_object_mut()
        .and_then(|object| object.remove("payload"))
        .ok_or_else(|| <serde_json::Error as de::Error>::custom("missing persisted payload"))?;

    serde_json::from_value(payload)
}

pub(crate) fn from_slice<T: DeserializeOwned>(bytes: &[u8]) -> serde_json::Result<T> {
    from_value(serde_json::from_slice(bytes)?)
}

pub(crate) fn to_vec(value: &impl Serialize) -> serde_json::Result<Vec<u8>> {
    serde_json::to_vec(&to_value(value)?)
}

/* === Private Functions === */

/* === Tests === */
