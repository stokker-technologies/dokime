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
use std::{ffi::OsStr, path::Path};

// Third Party Imports

// Local Imports

/* === Constants === */

// Dokime's Own License, Always Installed As `LICENSE`.
pub const DOKIME: &str = "Apache-2.0";

// Dokime's Copyright Holder, Named In Package Metadata And The Debian Copyright File.
pub const DOKIME_HOLDER: &str = "Stokker Technologies";

// Installed File Stem Of An Organization Pack's Additional Terms.
const TERMS_STEM: &str = "EULA";

// Debian Machine-Readable Copyright Format Revision The Generated File Declares.
const DEP5_FORMAT: &str = "https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/";

// Dokime's Upstream Name And Source Repository.
const UPSTREAM_NAME: &str = "Dokime";
const UPSTREAM_SOURCE: &str = env!("CARGO_PKG_REPOSITORY");

// Debian's Shared Copy Of Dokime's License Text.
const COMMON_LICENSE: &str = "/usr/share/common-licenses/Apache-2.0";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Checks An SPDX License Expression Against The SPDX Expression Grammar.
//
// Identifiers Are Syntax-Checked Only, Not Matched Against The SPDX License List.
pub fn validate(expression: &str) -> Result<(), String> {
    spdx::Expression::parse(expression).map(|_| ()).map_err(|error| error.to_string())
}

// Returns The Native Package License: The Pack's Terms Added To Dokime's Apache-2.0.
pub fn combined(expression: Option<&str>) -> String {
    let Some(expression) = expression.map(str::trim) else {
        return DOKIME.into();
    };
    // AND Binds Tighter Than OR, So A Compound Expression Keeps Its Own Grouping.
    let compound = expression.contains(char::is_whitespace) || expression.contains('(');
    if compound { format!("({expression}) AND {DOKIME}") } else { format!("{expression} AND {DOKIME}") }
}

// Returns The Installed Name Of The Pack's Terms: `EULA` Plus The Source File's Extension.
pub fn installed_name(file: &Path) -> String {
    let extension = file.extension().and_then(OsStr::to_str);
    match extension {
        Some(extension) => format!("{TERMS_STEM}.{extension}"),
        None => TERMS_STEM.into(),
    }
}

// Renders A DEP-5 Debian Copyright File For Dokime And Any Pack Terms.
//
// `terms` Holds The Pack's SPDX Expression And Its Text; Holders Are Listed In Order.
pub fn debian_copyright(holders: &[&str], apache: &str, terms: Option<(&str, &str)>) -> String {
    let header = format!("Format: {DEP5_FORMAT}\nUpstream-Name: {UPSTREAM_NAME}\nSource: {UPSTREAM_SOURCE}\n");
    let mut files = String::from("Files: *\nCopyright:");
    for holder in holders {
        let holder = single_line(holder);
        files.push_str(&format!(" {holder}\n"));
    }

    // One Standalone License Paragraph Per Term: The Pack's Terms, Then Dokime's Apache-2.0.
    let mut paragraphs = vec![header];
    let mut pack = None;
    if let Some((expression, text)) = terms {
        let expression = single_line(expression);
        paragraphs.push(format!("License: {expression}\n{}", license_text(text)));
        pack = Some(expression);
    }
    let reference = format!(" On Debian systems, the complete text of the Apache License 2.0 can be found in {COMMON_LICENSE}.\n .\n");
    paragraphs.push(format!("License: {DOKIME}\n{reference}{}", license_text(apache)));

    // The Files Paragraph Declares The Effective Expression Of Every Term.
    let expression = combined(pack.as_deref());
    files.push_str(&format!("License: {expression}\n"));
    paragraphs.insert(1, files);
    paragraphs.join("\n")
}

/* === Private Functions === */

// Collapses A Field Value Onto One Line So It Cannot Open Another Field.
fn single_line(value: &str) -> String {
    let words: Vec<&str> = value.split_whitespace().collect();
    words.join(" ")
}

// Formats License Text As DEP-5 Continuation Lines Without Surrounding Blank Lines.
fn license_text(text: &str) -> String {
    // Drop Surrounding Blank Lines But Keep The First Line's Indentation.
    let text = text.trim_start_matches('\u{feff}');
    let text = text.trim_end();
    let first = text.len() - text.trim_start().len();
    let start = text[..first].rfind('\n').map_or(0, |newline| newline + 1);

    let mut lines = String::new();
    for line in text[start..].lines() {
        lines.push_str(&continuation(line));
    }
    lines
}

// Returns One Continuation Line Indented One Space; A Blank Line Becomes ` .`.
fn continuation(line: &str) -> String {
    let line = line.trim_end();
    if line.is_empty() {
        return " .\n".into();
    }
    format!(" {line}\n")
}

/* === Tests === */
