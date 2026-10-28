/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Match Bounded Identifier Globs For Evidence Observation

use super::Identifier;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct IdentifierPattern(String);

impl IdentifierPattern {
    pub fn new(value: impl Into<String>) -> crate::Result<Self> {
        let value = value.into();
        let literal = value.replace(['*', '?'], "a");
        Identifier::new(literal)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn matches(&self, identifier: &str) -> bool {
        let pattern = self.0.as_bytes();
        let identifier = identifier.as_bytes();
        let (mut at, mut next) = (0, 0);
        let mut wildcard = None;
        let mut retry = 0;
        while next < identifier.len() {
            if at < pattern.len() && (pattern[at] == b'?' || pattern[at] == identifier[next]) {
                at += 1;
                next += 1;
            } else if at < pattern.len() && pattern[at] == b'*' {
                wildcard = Some(at);
                at += 1;
                retry = next;
            } else if let Some(star) = wildcard {
                retry += 1;
                next = retry;
                at = star + 1;
            } else {
                return false;
            }
        }
        pattern[at..].iter().all(|byte| *byte == b'*')
    }
}

impl TryFrom<String> for IdentifierPattern {
    type Error = crate::Error;

    fn try_from(value: String) -> crate::Result<Self> {
        Self::new(value)
    }
}

impl From<Identifier> for IdentifierPattern {
    fn from(value: Identifier) -> Self {
        Self(value.as_str().to_owned())
    }
}

impl From<IdentifierPattern> for String {
    fn from(value: IdentifierPattern) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::IdentifierPattern;

    #[test]
    fn whole_identifier_globs() {
        for (pattern, identifier, expected) in [
            ("obc.run.*", "obc.run.started", true),
            ("obc.*", "obc.run.completed", true),
            ("obc.run.?", "obc.run.x", true),
            ("obc.run.?", "obc.run.xx", false),
            ("obc.run.*", "other.obc.run.started", false),
            ("obc.run.*", "OBC.run.started", false),
            ("obc.run.started", "obc.run.started.extra", false),
            ("obc.run.started", "obc.run.started", true),
            ("*completed", "obc.run.completed", true),
            ("*a*a*b", "aaaab", true),
            ("*a*a*b", "aaaa", false),
            ("obc.**.done*", "obc.pulse.done", true),
            ("*", "obc.run.started", true),
        ] {
            let selected = IdentifierPattern::new(pattern).unwrap();
            assert_eq!(selected.matches(identifier), expected, "{pattern:?} against {identifier:?}");
        }
    }

    #[test]
    fn pattern_roundtrip_refusals() {
        let pattern = IdentifierPattern::new("obc.*.?").unwrap();
        let json = serde_json::to_string(&pattern).unwrap();
        assert_eq!(serde_json::from_str::<IdentifierPattern>(&json).unwrap(), pattern);

        for invalid in ["", " obc.*", "obc.[ab]", "obc.{run,pulse}", "^obc.*$", "obc/.*", "obc.é", "obc.\\*"] {
            assert!(IdentifierPattern::new(invalid).is_err(), "accepted {invalid:?}");
            let json = serde_json::to_string(invalid).unwrap();
            assert!(serde_json::from_str::<IdentifierPattern>(&json).is_err(), "decoded {invalid:?}");
        }
    }
}
