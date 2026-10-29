/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Ordered include and exclude globs: the last rule that matches a path
// decides, and a path no rule matches is excluded.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use globset::{GlobBuilder, GlobMatcher};

// Local Imports
use dokime_domain::model::files::wire::FileRule;
use dokime_domain::model::files::wire::FileRuleAction;

/* === Constants === */

pub const MAX_RULES: usize = 64;
pub const MAX_GLOB_BYTES: usize = 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct FileMatcher {
    rules: Vec<(GlobMatcher, bool)>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidRule {
    #[error("a watch needs at least one include rule within its configured rule limit")]
    Count,

    #[error("rule {index} must include or exclude")]
    Action { index: usize },

    #[error("rule {index} glob must be nonempty, within its configured byte limit and without a leading '/'")]
    Shape { index: usize },

    #[error("rule {index} glob is invalid: {reason}")]
    Glob { index: usize, reason: String },
}

/* === Traits === */

/* === Implementations === */

impl FileMatcher {
    pub fn compile(rules: &[FileRule]) -> Result<Self, InvalidRule> {
        if rules.len() > MAX_RULES || !rules.iter().any(|rule| rule.action == FileRuleAction::Include) {
            return Err(InvalidRule::Count);
        }
        let mut compiled = Vec::with_capacity(rules.len());
        for (index, rule) in rules.iter().enumerate() {
            let include = match rule.action {
                FileRuleAction::Include => true,
                FileRuleAction::Exclude => false,
                FileRuleAction::Unspecified => return Err(InvalidRule::Action { index }),
            };
            if rule.glob.is_empty() || rule.glob.len() > MAX_GLOB_BYTES || rule.glob.starts_with('/') || rule.glob.contains('\\') {
                return Err(InvalidRule::Shape { index });
            }
            let glob = GlobBuilder::new(&rule.glob)
                .literal_separator(true)
                .backslash_escape(false)
                .build()
                .map_err(|source| InvalidRule::Glob { index, reason: source.kind().to_string() })?;
            compiled.push((glob.compile_matcher(), include));
        }
        Ok(Self { rules: compiled })
    }

    // Whether a forward-slash path relative to the source root is tracked.
    pub fn matches(&self, relative: &str) -> bool {
        self.rules.iter().rev().find(|(glob, _)| glob.is_match(relative)).is_some_and(|(_, include)| *include)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{FileMatcher, InvalidRule};
    use dokime_domain::model::files::wire::{FileRule, FileRuleAction};

    fn include(glob: &str) -> FileRule {
        FileRule {
            glob: glob.into(),
            action: FileRuleAction::Include,
        }
    }

    fn exclude(glob: &str) -> FileRule {
        FileRule {
            glob: glob.into(),
            action: FileRuleAction::Exclude,
        }
    }

    fn matcher(rules: &[FileRule]) -> FileMatcher {
        compile(rules).expect("the rules compile")
    }

    fn compile(rules: &[FileRule]) -> Result<FileMatcher, InvalidRule> {
        FileMatcher::compile(rules)
    }

    #[test]
    fn last_rule_decides() {
        let rules = matcher(&[include("**/*.log"), exclude("**/debug/**"), include("**/debug/bms.log")]);

        // Included, Excluded, Then Re-Included By A Later Rule
        assert!(rules.matches("charger.log"));
        assert!(rules.matches("bench/a/charger.log"));
        assert!(!rules.matches("bench/debug/trace.log"));
        assert!(!rules.matches("debug/trace.log"));
        assert!(rules.matches("bench/debug/bms.log"));

        // No Rule Matching Means Excluded
        assert!(!rules.matches("charger.csv"));
        assert!(!rules.matches("bench/readme.txt"));
    }

    #[test]
    fn star_stays_segment() {
        let top = matcher(&[include("*.log")]);
        assert!(top.matches("charger.log"));
        assert!(!top.matches("bench/charger.log"));

        let nested = matcher(&[include("bench/*/charger.log")]);
        assert!(nested.matches("bench/a/charger.log"));
        assert!(!nested.matches("bench/a/b/charger.log"));
    }

    #[test]
    fn unreincluded_exclusion_wins() {
        let rules = matcher(&[include("**"), exclude("**/*.tmp")]);
        assert!(rules.matches("data/run.csv"));
        assert!(!rules.matches("data/run.csv.tmp"));

        // An Earlier Exclusion Is Overridden By A Later Inclusion Only Where It Matches
        let reordered = matcher(&[exclude("**/*.tmp"), include("**")]);
        assert!(reordered.matches("data/run.csv.tmp"));
    }

    #[test]
    fn malformed_rules_refused() {
        assert_eq!(compile(&[]).unwrap_err(), InvalidRule::Count);
        assert_eq!(compile(&[exclude("**/*.log")]).unwrap_err(), InvalidRule::Count);
        assert_eq!(
            compile(&[
                include("**"),
                FileRule {
                    glob: "*.log".into(),
                    action: FileRuleAction::Unspecified
                },
            ])
            .unwrap_err(),
            InvalidRule::Action { index: 1 }
        );
        for shape in ["", "/var/log/*.log", "logs\\*.log"] {
            assert_eq!(compile(&[include(shape)]).unwrap_err(), InvalidRule::Shape { index: 0 }, "{shape:?}");
        }
        assert!(matches!(compile(&[include("logs/[a-.log")]).unwrap_err(), InvalidRule::Glob { index: 0, .. }));
    }
}
