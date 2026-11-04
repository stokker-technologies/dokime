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

/// Installed File Stem Of An Organization Pack's Additional Terms.
const TERMS_STEM: &str = "EULA";

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
    if compound { format!("({expression}) AND {DOKIME}") } else { format!("{expression} AND {DOKIME}") }
}

/// Returns The Installed Name Of The Pack's Terms: `EULA` Plus The Source File's Extension.
pub fn installed_name(file: &Path) -> String {
    let extension = file.extension().and_then(OsStr::to_str);
    match extension {
        Some(extension) => format!("{TERMS_STEM}.{extension}"),
        None => TERMS_STEM.into(),
    }
}

/* === Private Functions === */

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

    #[test]
    fn installed_names() {
        assert_eq!(installed_name(Path::new("terms/EULA.txt")), "EULA.txt");
        assert_eq!(installed_name(Path::new("Acme Terms.rtf")), "EULA.rtf");
        assert_eq!(installed_name(Path::new("TERMS")), "EULA");
    }
}
