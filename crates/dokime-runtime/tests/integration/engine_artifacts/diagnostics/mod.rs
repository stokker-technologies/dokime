/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use dokime_runtime::diagnostics::Diagnostic;

pub(crate) fn one_redacted_diagnostic() {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return;
    }

    assert!(!dokime_runtime::diagnostics::redacted(&format!("{home}/bench.toml")).contains(&home));
}

pub(crate) fn redacted_diagnostic_directory() {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return;
    }
    let diagnostic = Diagnostic {
        message: format!("{home}/secret"),
        ..Diagnostic::default()
    }
    .redacted();

    assert!(!diagnostic.message.contains(&home), "{}", diagnostic.message);
}
