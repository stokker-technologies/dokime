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
use serde_json::Value;

// Local Imports

/* === Constants === */

pub(crate) const REDACTED: &str = "<redacted>";
const SECRET_FRAGMENTS: &[&str] =
    &["secret", "password", "passphrase", "token", "apikey", "privatekey", "accesskey", "authorization", "cookie"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn is_secret_name(name: &str) -> bool {
    let folded: String = name.chars().filter(|character| character.is_alphanumeric()).flat_map(char::to_lowercase).collect();
    SECRET_FRAGMENTS.iter().any(|fragment| folded.contains(fragment))
}

pub(super) fn scrub(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            for (name, field) in fields.iter_mut() {
                if is_secret_name(name) && !is_redacted_evidence(field) {
                    *field = Value::String(REDACTED.to_owned());
                    continue;
                }
                scrub(field);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(scrub),
        _ => {}
    }
}

pub(crate) fn text(value: &str) -> String {
    dokime_runtime::plugin::redacted(value)
}

/* === Private Functions === */

fn is_redacted_evidence(value: &Value) -> bool {
    value.as_object().is_some_and(|fields| fields.contains_key("credential_id") && fields.contains_key("configured"))
}

/* === Tests === */
