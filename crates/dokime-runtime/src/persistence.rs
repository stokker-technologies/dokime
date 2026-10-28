/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Serialize;
use serde_json::{Value as Json, json};

pub(crate) fn to_value(value: &impl Serialize) -> serde_json::Result<Json> {
    Ok(json!({"dokime_encoding": 2, "payload": serde_json::to_value(value)?}))
}

pub(crate) fn to_vec(value: &impl Serialize) -> serde_json::Result<Vec<u8>> {
    serde_json::to_vec(&to_value(value)?)
}
