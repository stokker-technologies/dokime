/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File watch rules compose as ordered include and exclude globs.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_runtime::files::{FileMatcher, InvalidRule};
use dokime_domain::operations::files::{FileRule, FileRuleAction};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
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
    FileMatcher::compile(rules).expect("the rules compile")
}

/* === Tests === */
#[test]
fn last_rule_decides() {
    let rules = matcher(&[
        include("**/*.log"),
        exclude("**/debug/**"),
        include("**/debug/bms.log"),
    ]);

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
    assert_eq!(FileMatcher::compile(&[]).unwrap_err(), InvalidRule::Count);
    assert_eq!(
        FileMatcher::compile(&[exclude("**/*.log")]).unwrap_err(),
        InvalidRule::Count
    );
    assert_eq!(
        FileMatcher::compile(&[
            include("**"),
            FileRule {
                glob: "*.log".into(),
                action: FileRuleAction::Unspecified,
            },
        ])
        .unwrap_err(),
        InvalidRule::Action { index: 1 }
    );
    for shape in ["", "/var/log/*.log", "logs\\*.log"] {
        assert_eq!(
            FileMatcher::compile(&[include(shape)]).unwrap_err(),
            InvalidRule::Shape { index: 0 },
            "{shape:?}"
        );
    }
    assert!(matches!(
        FileMatcher::compile(&[include("logs/[a-.log")]).unwrap_err(),
        InvalidRule::Glob { index: 0, .. }
    ));
}
