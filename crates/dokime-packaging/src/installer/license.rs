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

/// Dokime's Own License, Always Installed As `LICENSE`.
pub const DOKIME: &str = "Apache-2.0";

/// Dokime's Copyright Holder, Named In Package Metadata And The Debian Copyright File.
pub const DOKIME_HOLDER: &str = "Stokker Technologies";

/// Installed File Stem Of An Organization Pack's Additional Terms.
const TERMS_STEM: &str = "EULA";

/// Debian Machine-Readable Copyright Format Revision The Generated File Declares.
const DEP5_FORMAT: &str = "https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/";

/// Dokime's Upstream Name And Source Repository.
const UPSTREAM_NAME: &str = "Dokime";
const UPSTREAM_SOURCE: &str = env!("CARGO_PKG_REPOSITORY");

/// Debian's Shared Copy Of Dokime's License Text.
const COMMON_LICENSE: &str = "/usr/share/common-licenses/Apache-2.0";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Expression Grammar State: Open Groups And The Kind Of Token Expected Next.
struct Parser {
    depth: usize,
    operand: bool,
    simple: bool,
    exception: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Parser {
    fn accept(&mut self, token: &str) -> Result<(), String> {
        if self.operand {
            return self.accept_operand(token);
        }
        match token {
            ")" if self.depth > 0 => (self.depth, self.simple) = (self.depth - 1, false),
            "AND" | "OR" => self.operand = true,
            "WITH" if self.simple => (self.operand, self.exception) = (true, true),
            _ => return Err(format!("unexpected `{token}`; expected AND, OR, WITH or `)`")),
        }
        Ok(())
    }

    fn accept_operand(&mut self, token: &str) -> Result<(), String> {
        if token == "(" && !self.exception {
            self.depth += 1;
            return Ok(());
        }
        let valid = if self.exception { exception(token) } else { license(token) };
        if !valid {
            return Err(format!("`{token}` is not a license identifier, LicenseRef- or exception identifier"));
        }
        (self.operand, self.simple, self.exception) = (false, !self.exception, false);
        Ok(())
    }
}

/* === Public Functions === */

/// Checks An SPDX License Expression Against The SPDX Expression Grammar.
///
/// Identifiers Are Syntax-Checked Only, Not Matched Against The SPDX License List.
pub fn validate(expression: &str) -> Result<(), String> {
    let spaced = expression.replace('(', " ( ").replace(')', " ) ");
    let mut parser = Parser {
        depth: 0,
        operand: true,
        simple: false,
        exception: false,
    };
    for token in spaced.split_whitespace() {
        parser.accept(token)?;
    }
    if parser.operand || parser.depth > 0 {
        return Err("expression is incomplete".into());
    }
    Ok(())
}

/// Returns The Native Package License: The Pack's Terms Added To Dokime's Apache-2.0.
pub fn combined(expression: Option<&str>) -> String {
    let Some(expression) = expression.map(str::trim) else {
        return DOKIME.into();
    };
    // AND Binds Tighter Than OR, So A Compound Expression Keeps Its Own Grouping.
    let compound = expression.contains(char::is_whitespace) || expression.contains('(');
    if compound { format!("({expression}) AND {DOKIME}") } else { format!("{expression} and {DOKIME}") }
}

/// Returns The Installed Name Of The Pack's Terms: `EULA` Plus The Source File's Extension.
pub fn installed_name(file: &Path) -> String {
    let extension = file.extension().and_then(OsStr::to_str);
    match extension {
        Some(extension) => format!("{TERMS_STEM}.{extension}"),
        None => TERMS_STEM.into(),
    }
}

/// Renders A DEP-5 Debian Copyright File For Dokime And Any Pack Terms.
///
/// `terms` Holds The Pack's SPDX Expression And Its Text; Holders Are Listed In Order.
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

/// Collapses A Field Value Onto One Line So It Cannot Open Another Field.
fn single_line(value: &str) -> String {
    let words: Vec<&str> = value.split_whitespace().collect();
    words.join(" ")
}

/// Formats License Text As DEP-5 Continuation Lines Without Surrounding Blank Lines.
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

/// Returns One Continuation Line Indented One Space; A Blank Line Becomes ` .`.
fn continuation(line: &str) -> String {
    let line = line.trim_end();
    if line.is_empty() {
        return " .\n".into();
    }
    format!(" {line}\n")
}

fn license(token: &str) -> bool {
    if let Some((document, reference)) = token.split_once(':') {
        let document = document.strip_prefix("DocumentRef-");
        let reference = reference.strip_prefix("LicenseRef-");
        return document.is_some_and(idstring) && reference.is_some_and(idstring);
    }
    if let Some(name) = token.strip_prefix("LicenseRef-") {
        return idstring(name);
    }
    let name = token.strip_suffix('+').unwrap_or(token);
    idstring(name)
}

fn exception(token: &str) -> bool {
    idstring(token) && !token.starts_with("LicenseRef-")
}

fn idstring(token: &str) -> bool {
    let reserved = ["AND", "OR", "WITH"].contains(&token);
    !token.is_empty() && !reserved && token.bytes().all(idchar)
}

fn idchar(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.'
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expressions() {
        for valid in [
            "LicenseRef-Acme-Proprietary",
            "DocumentRef-acme:LicenseRef-Terms",
            "MIT",
            "GPL-2.0+",
            "GPL-2.0-only WITH Classpath-exception-2.0",
            "MIT OR (LicenseRef-Acme AND BSD-3-Clause)",
            "((MIT))",
        ] {
            assert!(validate(valid).is_ok(), "{valid}");
        }
        for invalid in [
            "",
            "   ",
            "LicenseRef-",
            "LicenseRef-Acme+",
            "Acme Proprietary",
            "MIT and BSD-3-Clause",
            "MIT AND",
            "AND MIT",
            "(MIT",
            "MIT)",
            "()",
            "MIT WITH LicenseRef-Acme",
            "(MIT) WITH Classpath-exception-2.0",
            "MIT WITH",
            "Acme/Terms",
            "DocumentRef-acme:MIT",
            "MIT WITH (Classpath-exception-2.0)",
        ] {
            assert!(validate(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn combined_expressions() {
        assert_eq!(combined(None), "Apache-2.0");
        assert_eq!(combined(Some("LicenseRef-Acme-Proprietary")), "LicenseRef-Acme-Proprietary AND Apache-2.0");
        assert_eq!(combined(Some("MIT OR LicenseRef-Acme")), "(MIT OR LicenseRef-Acme) AND Apache-2.0");
    }

    /// Splits A Rendered Copyright File Into Its Blank-Line-Separated Paragraphs.
    fn paragraphs(file: &str) -> Vec<Vec<&str>> {
        let paragraphs = file.split("\n\n");
        paragraphs.map(|paragraph| paragraph.lines().collect()).collect()
    }

    /// Asserts Every Line After A Paragraph's First Field Is A Nonblank DEP-5 Continuation Line.
    fn assert_continuations(paragraph: &[&str]) {
        for line in &paragraph[1..] {
            assert!(line.starts_with(' ') && !line.trim().is_empty(), "{line:?}");
        }
    }

    #[test]
    fn debian_copyright_without_terms() {
        let file = debian_copyright(&[DOKIME_HOLDER], "\n  Apache License\n\n  Terms   \n\n", None);
        let paragraphs = paragraphs(&file);
        assert_eq!(paragraphs.len(), 3);
        let header = [format!("Format: {DEP5_FORMAT}"), format!("Upstream-Name: {UPSTREAM_NAME}"), format!("Source: {UPSTREAM_SOURCE}")];
        assert_eq!(paragraphs[0], header);
        let files = ["Files: *".to_owned(), format!("Copyright: {DOKIME_HOLDER}"), "License: Apache-2.0".to_owned()];
        assert_eq!(paragraphs[1], files);

        // Apache-2.0 Names Debian's Common Copy, Then Carries Its Own Text.
        let apache = &paragraphs[2];
        assert_eq!(apache[0], "License: Apache-2.0");
        assert_continuations(apache);
        assert!(apache[1].contains(COMMON_LICENSE));
        assert_eq!(apache[2..], [" .", "   Apache License", " .", "   Terms"]);
        assert!(file.ends_with("   Terms\n"));
    }

    #[test]
    fn debian_copyright_with_terms() {
        let holders = [DOKIME_HOLDER, "Acme\nCorporation"];
        let terms = ("MIT OR\nLicenseRef-Acme", "\u{feff}Acme terms\r\n\r\nSection 2\r\n");
        let file = debian_copyright(&holders, "Apache text", Some(terms));
        let paragraphs = paragraphs(&file);
        assert_eq!(paragraphs.len(), 4);
        let copyright = format!("Copyright: {DOKIME_HOLDER}");
        let expected = ["Files: *", &copyright, " Acme Corporation", "License: (MIT OR LicenseRef-Acme) AND Apache-2.0"];
        assert_eq!(paragraphs[1], expected);

        // The Pack's Terms Precede Dokime's Apache-2.0 Paragraph.
        assert_eq!(paragraphs[2], ["License: MIT OR LicenseRef-Acme", " Acme terms", " .", " Section 2"]);
        let apache = &paragraphs[3];
        assert_eq!(apache[0], "License: Apache-2.0");
        assert_continuations(apache);
        assert_eq!(apache.last(), Some(&" Apache text"));
    }

    #[test]
    fn installed_names() {
        assert_eq!(installed_name(Path::new("terms/EULA.txt")), "EULA.txt");
        assert_eq!(installed_name(Path::new("Acme Terms.rtf")), "EULA.rtf");
        assert_eq!(installed_name(Path::new("TERMS")), "EULA");
    }
}
