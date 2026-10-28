/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde_json::Value;

pub(crate) const REDACTED: &str = "<redacted>";
const SECRET_FRAGMENTS: &[&str] =
    &["secret", "credential", "password", "passphrase", "token", "apikey", "privatekey", "accesskey", "authorization", "cookie"];

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
    dokime::plugin::redacted(value)
}

fn is_redacted_evidence(value: &Value) -> bool {
    if is_redacted_binding(value) {
        return true;
    }
    let Some(fields) = value.as_object() else { return false };
    fields.len() == 2
        && fields.get("bindings").and_then(Value::as_object).is_some_and(|bindings| bindings.values().all(is_redacted_binding))
        && fields.get("missing_required").and_then(Value::as_array).is_some_and(|names| names.iter().all(Value::is_string))
}

fn is_redacted_binding(value: &Value) -> bool {
    let Some(fields) = value.as_object() else { return false };
    fields.len() == 3
        && fields.get("secret_id").is_some_and(Value::is_string)
        && fields.get("backend").is_some_and(Value::is_string)
        && fields.get("configured").is_some_and(Value::is_boolean)
}
