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
    &["secret", "credential", "password", "passphrase", "token", "apikey", "privatekey", "accesskey", "authorization", "cookie"];

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
    dokime::plugin::redacted(value)
}

/* === Private Functions === */

fn is_redacted_evidence(value: &Value) -> bool {
    let Some(entries) = value.as_array() else { return false };
    entries.iter().all(|entry| {
        let Some(fields) = entry.as_object() else { return false };
        fields.len() == 2
            && fields.get("identifier").is_some_and(Value::is_string)
            && fields.get("available").is_some_and(Value::is_boolean)
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_availability_redaction() {
        let availability = serde_json::json!([
            {"identifier": "fixture.present", "available": true},
            {"identifier": "fixture.missing", "available": false}
        ]);
        let mut value = serde_json::json!({"secrets": availability.clone(), "api_token": "fixture-secret"});
        scrub(&mut value);
        assert_eq!(value["secrets"], availability);
        assert_eq!(value["api_token"], REDACTED);

        for malformed in [
            serde_json::json!(["fixture-secret"]),
            serde_json::json!([{"identifier": "fixture.present", "available": true, "value": "fixture-secret"}]),
            serde_json::json!([{"identifier": "fixture.present", "available": "fixture-secret"}]),
            serde_json::json!([{"secret_id": "fixture.present", "backend": "fixture", "configured": true}]),
        ] {
            let mut value = serde_json::json!({"secrets": malformed});
            scrub(&mut value);
            assert_eq!(value["secrets"], REDACTED);
        }
    }
}
