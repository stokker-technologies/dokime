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

// Personal Data Is Opt-In Only for Support Archives, Never for Secret Values.
pub(super) fn include_personal_data() -> bool {
    std::env::var("DOKIME_SUPPORT_INCLUDE_PERSONAL_DATA").is_ok_and(|value| value == "1")
}

pub(super) fn profile_paths() -> Vec<String> {
    ["USERPROFILE", "HOME"].iter().filter_map(|key| std::env::var(key).ok()).collect()
}

pub(super) fn personal(value: &mut Value, profiles: &[String]) {
    match value {
        Value::Object(fields) => {
            let identity_setting =
                ["id", "identifier", "name"].iter().any(|key| fields.get(*key).and_then(Value::as_str).is_some_and(personal_name));
            for (name, field) in fields.iter_mut() {
                if personal_name(name) || (identity_setting && matches!(name.as_str(), "value" | "default")) {
                    *field = Value::String(REDACTED.to_owned());
                    continue;
                }
                personal(field, profiles);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|item| personal(item, profiles)),
        Value::String(value) if sensitive_text(value.as_bytes(), profiles) => *value = REDACTED.to_owned(),
        _ => {}
    }
}

// Preserve Non-UTF-8 Diagnostic Bytes; Omit the Whole Sensitive Line.
pub(super) fn personal_line(line: &[u8], include_personal: bool, profiles: &[String]) -> Vec<u8> {
    if !include_personal && sensitive_text(line, profiles) {
        let mut output = REDACTED.as_bytes().to_vec();
        if line.ends_with(b"\n") {
            output.push(b'\n');
        }
        return output;
    }
    line.to_vec()
}

/* === Private Functions === */

fn personal_name(name: &str) -> bool {
    let folded: String = name.chars().filter(|character| character.is_ascii_alphanumeric()).flat_map(char::to_lowercase).collect();
    matches!(
        folded.as_str(),
        "gitusername"
            | "gituseremail"
            | "username"
            | "useremail"
            | "gitauthorname"
            | "gitauthoremail"
            | "gitcommittername"
            | "gitcommitteremail"
    )
}

fn sensitive_text(value: &[u8], profiles: &[String]) -> bool {
    let lower = value.to_ascii_lowercase();
    // Backslash Normalization Also Covers JSON-Escaped Windows Paths.
    let normalized: Vec<u8> = lower.iter().copied().filter(|byte| *byte != b'\\').collect();
    let profile = [b"/home/".as_slice(), b"/users/", b"\\users\\", b"\\documents and settings\\"].iter().any(|pattern| {
        lower.windows(pattern.len()).any(|window| window == *pattern) || normalized.windows(pattern.len()).any(|window| window == *pattern)
    });
    let identity = [
        b"git_user_name".as_slice(),
        b"git_user_email",
        b"gitusername",
        b"gituseremail",
        b"git user name",
        b"git user email",
        b"user.name",
        b"user.email",
        b"git_author_name",
        b"git_committer_name",
    ]
    .iter()
    .any(|pattern| normalized.windows(pattern.len()).any(|window| window == *pattern));
    let current_profile = profiles.iter().filter(|path| path.len() > 1).any(|path| {
        let path = path.to_ascii_lowercase().replace('\\', "");
        !path.is_empty() && normalized.windows(path.len()).any(|window| window == path.as_bytes())
    });
    profile || identity || current_profile || value.contains(&b'@')
}

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
    fn support_personal_data() {
        let mut value = serde_json::json!({
            "git_user_name": "Fixture Person",
            "git_user_email": "fixture.person@example.test",
            "nested": [{"id": "git_user_name", "default": "Other Person"}],
            "path": "C:\\Users\\Fixture Person\\lab",
            "message": "contact fixture.person@example.test",
            "ordinary": "ready",
        });
        personal(&mut value, &[]);
        assert_eq!(value["git_user_name"], REDACTED);
        assert_eq!(value["git_user_email"], REDACTED);
        assert_eq!(value["nested"][0]["default"], REDACTED);
        assert_eq!(value["path"], REDACTED);
        assert_eq!(value["message"], REDACTED);
        assert_eq!(value["ordinary"], "ready");
        let line = b"git_user_name=Fixture Person fixture.person@example.test C:\\Users\\Fixture\n";
        assert_eq!(personal_line(line, false, &[]), b"<redacted>\n");
        assert_eq!(personal_line(line, true, &[]), line);
        assert_eq!(personal_line(b"workspace=/srv/operator/lab", false, &["/srv/operator".into()]), b"<redacted>");
    }

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
