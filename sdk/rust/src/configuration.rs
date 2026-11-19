/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Working Directory Connection Configuration

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, env, fs, io::ErrorKind, path::Path};

// Third Party Imports

// Local Imports
use crate::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn load() -> Result<HashMap<String, String>> {
    let path = env::current_dir()?.join(".env");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(error) => return Err(Error::Invalid(format!("Read {}:1: Make the Configuration File Readable: {error}", path.display()))),
    };
    let content = str::from_utf8(&bytes).map_err(|error| {
        let number = bytes[..error.valid_up_to()].iter().filter(|byte| **byte == b'\n').count() + 1;
        Error::Invalid(format!("Read {}:{number}: Save the File as UTF-8", path.display()))
    })?;
    parse(content, &path)
}

pub(crate) fn resolve(key: &str, explicit: Option<&str>, values: &HashMap<String, String>) -> Result<String> {
    if explicit.is_some() {
        return select(key, explicit, None, values);
    }
    let process = env::var(key).map(Some).or_else(|error| match error {
        env::VarError::NotPresent => Ok(None),
        env::VarError::NotUnicode(_) => Err(Error::Invalid(format!("Configure Connection: Set {key} to UTF-8 Text"))),
    })?;
    select(key, explicit, process.as_deref(), values)
}

pub(crate) fn select(key: &str, explicit: Option<&str>, process: Option<&str>, values: &HashMap<String, String>) -> Result<String> {
    let default = if key == "DOKIME_PLUGIN_ADDRESS" { "127.0.0.1:0" } else { "" };
    let value = explicit.or(process).or_else(|| values.get(key).map(String::as_str)).unwrap_or(default);
    if value.is_empty() {
        return Err(Error::Invalid(format!("Configure Connection: Set {key} to a Nonempty Address")));
    }
    Ok(value.into())
}

pub(crate) fn parse(content: &str, path: &Path) -> Result<HashMap<String, String>> {
    let mut values = HashMap::new();
    let lines = content.strip_prefix('\u{feff}').unwrap_or(content).split('\n').collect::<Vec<_>>();
    for (index, raw) in lines.iter().enumerate() {
        let malformed = || {
            Error::Invalid(format!(
                "Read {}:{}: Use One Unique KEY=VALUE Assignment With Balanced Quotes and Supported Escapes",
                path.display(),
                index + 1
            ))
        };
        let raw = if index + 1 < lines.len() { raw.strip_suffix('\r').unwrap_or(raw) } else { raw };
        let line = raw.trim_matches([' ', '\t']);
        if line.contains(['\0', '\r']) {
            return Err(malformed());
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(malformed)?;
        let key = key.trim_end_matches([' ', '\t']);
        let mut characters = key.chars();
        if !characters.next().is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
            || !characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
            || values.contains_key(key)
        {
            return Err(malformed());
        }
        let value = value.trim_start_matches([' ', '\t']);
        let Some(quote @ ('\'' | '"')) = value.chars().next() else {
            let mut previous_space = true;
            let end = value
                .char_indices()
                .find_map(|(index, character)| {
                    let comment = character == '#' && previous_space;
                    previous_space = matches!(character, ' ' | '\t');
                    comment.then_some(index)
                })
                .unwrap_or(value.len());
            values.insert(key.into(), value[..end].trim_matches([' ', '\t']).into());
            continue;
        };
        let mut decoded = String::new();
        let mut characters = value[1..].char_indices();
        let mut closed = false;
        while let Some((index, character)) = characters.next() {
            if character == quote {
                let tail = value[index + 2..].trim_matches([' ', '\t']);
                if !tail.is_empty() && !tail.starts_with('#') {
                    return Err(malformed());
                }
                closed = true;
                break;
            }
            let character = if character == '\\' && quote == '"' {
                match characters.next().map(|(_, character)| character) {
                    Some('\\') => '\\',
                    Some('"') => '"',
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    _ => return Err(malformed()),
                }
            } else {
                character
            };
            decoded.push(character);
        }
        if !closed {
            return Err(malformed());
        }
        values.insert(key.into(), decoded);
    }
    Ok(values)
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{parse, select};
    use std::path::Path;

    #[test]
    fn dotenv_preserves_literals_and_decodes_only_supported_escapes() {
        let text = "\u{feff} # Addresses\r\nDOKIME_REMOTE = 'http://host:7#literal' # Comment\r\nDOKIME_PLUGIN_ADDRESS=127.0.0.1:0\nOTHER=\"a\\n\\t\\r\\\\\\\"#b\"\nEMPTY=\nHASH=a#b # Tail\n";
        let values = parse(text, Path::new(".env")).unwrap();
        assert_eq!(values["DOKIME_REMOTE"], "http://host:7#literal");
        assert_eq!(values["OTHER"], "a\n\t\r\\\"#b");
        assert_eq!(values["EMPTY"], "");
        assert_eq!(values["HASH"], "a#b");
        assert_eq!(select("DOKIME_REMOTE", Some("argument"), Some("process"), &values).unwrap(), "argument");
        assert_eq!(select("DOKIME_REMOTE", None, Some("process"), &values).unwrap(), "process");
        assert_eq!(select("DOKIME_REMOTE", None, None, &values).unwrap(), values["DOKIME_REMOTE"]);
        assert!(select("DOKIME_REMOTE", Some(""), Some("process"), &values).is_err());
        assert!(select("DOKIME_REMOTE", None, Some(""), &values).is_err());
    }

    #[test]
    fn dotenv_refuses_ambiguous_or_shell_grammar_with_a_location() {
        for text in ["A=1\nA=2", "export A=1", "A", "1A=2", "A='unclosed", "A=\"\\x\"", "A='x' tail", "A=x\0", "A=x\ry", "A=x\r"] {
            let error = parse(text, Path::new("project.env")).unwrap_err().to_string();
            assert!(error.contains("project.env:"), "{text:?}: {error}");
        }
    }
}
