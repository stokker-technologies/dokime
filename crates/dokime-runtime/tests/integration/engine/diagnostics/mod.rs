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

// Local Imports
use dokime_runtime::diagnostics::Diagnostic;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn one_redacted_diagnostic() {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return;
    }

    assert!(!dokime_runtime::diagnostics::redacted(&format!("{home}/bench.toml")).contains(&home));
}

#[test]
fn redacted_diagnostic_directory() {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return;
    }
    let diagnostic = Diagnostic {
        message: format!("{home}/secret"),
        ..Diagnostic::default()
    }
    .redacted();

    assert!(
        !diagnostic.message.contains(&home),
        "{}",
        diagnostic.message
    );
}
