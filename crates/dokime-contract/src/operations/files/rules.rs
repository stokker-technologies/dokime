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
use super::{FileRule, FileRuleAction};

/* === Constants === */
// The most rules one watch may carry.
pub const MAX_RULES: usize = 64;

// The longest glob one rule may carry, in UTF-8 bytes.
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
    #[error("a watch needs at least one include rule and at most {MAX_RULES} rules")]
    Count,
    #[error("rule {index} must include or exclude")]
    Action { index: usize },
    #[error("rule {index} glob must contain 1–{MAX_GLOB_BYTES} bytes without a leading '/'")]
    Shape { index: usize },
    #[error("rule {index} glob is invalid: {reason}")]
    Glob { index: usize, reason: String },
}

/* === Traits === */

/* === Implementations === */
impl FileMatcher {
    pub fn compile(rules: &[FileRule]) -> Result<Self, InvalidRule> {
        if rules.len() > MAX_RULES
            || !rules
                .iter()
                .any(|rule| rule.action == FileRuleAction::Include)
        {
            return Err(InvalidRule::Count);
        }
        let mut compiled = Vec::with_capacity(rules.len());
        for (index, rule) in rules.iter().enumerate() {
            let include = match rule.action {
                FileRuleAction::Include => true,
                FileRuleAction::Exclude => false,
                FileRuleAction::Unspecified => return Err(InvalidRule::Action { index }),
            };
            if rule.glob.is_empty()
                || rule.glob.len() > MAX_GLOB_BYTES
                || rule.glob.starts_with('/')
                || rule.glob.contains('\\')
            {
                return Err(InvalidRule::Shape { index });
            }
            let glob = GlobBuilder::new(&rule.glob)
                .literal_separator(true)
                .backslash_escape(false)
                .build()
                .map_err(|source| InvalidRule::Glob {
                    index,
                    reason: source.kind().to_string(),
                })?;
            compiled.push((glob.compile_matcher(), include));
        }
        Ok(Self { rules: compiled })
    }

    // Whether a forward-slash path relative to the source root is tracked.
    pub fn matches(&self, relative: &str) -> bool {
        self.rules
            .iter()
            .rev()
            .find(|(glob, _)| glob.is_match(relative))
            .is_some_and(|(_, include)| *include)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
