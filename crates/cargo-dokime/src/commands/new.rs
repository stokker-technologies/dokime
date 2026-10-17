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
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use crate::error::Error;

/* === Constants === */
const REPOSITORY: &str = "https://github.com/stokker-technologies/dokime";

include!(concat!(env!("OUT_DIR"), "/template-assets.rs"));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(clap::Args)]
#[command(about = "Create a native Dokime plugin (offline; no compilation)")]
pub struct Args {
    #[arg(help = "New directory to create. Its parent must already exist.")]
    pub path: PathBuf,
    #[arg(long, help = "Cargo package name (defaults to the directory name).")]
    pub name: Option<String>,
}

struct Template {
    bytes: &'static [u8],
    text: bool,
}

struct Workspace {
    manifest: PathBuf,
    original: String,
    updated: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn run(args: Args) -> Result<(), Error> {
    let name = args
        .name
        .as_deref()
        .or_else(|| args.path.file_name()?.to_str())
        .ok_or_else(|| Error::Invalid("provide a package name with --name".into()))?;
    validate_name(name)?;
    let leaf = args.path.file_name().ok_or_else(|| Error::Invalid("provide a new directory path".into()))?;
    let parent = args.path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let target = parent.canonicalize()?.join(leaf);
    if target.symlink_metadata().is_ok() {
        return Err(Error::Invalid(format!("refusing to overwrite {}", target.display())));
    }
    let (workspace, detached) = workspace(&target)?;
    let selection = if workspace.is_some() { format!(" -p {name}") } else { String::new() };
    let crate_name = name.replace('-', "_");
    let owner = name.replace('_', "-");
    // Reserve the target exclusively. On failure only this newly created tree
    // is removed; the workspace manifest is committed last by atomic rename.
    fs::create_dir(&target)?;
    let result = (|| -> Result<(), Error> {
        for (path, template) in FILES {
            let path = target.join(path);
            fs::create_dir_all(path.parent().expect("template has a parent"))?;
            if template.text {
                let contents = std::str::from_utf8(template.bytes).expect("the asset inventory identifies UTF-8 text");
                let mut contents = contents
                    .replace("{{package}}", name)
                    .replace("{{crate}}", &crate_name)
                    .replace("{{owner}}", &owner)
                    .replace("{{repository}}", REPOSITORY)
                    .replace("{{selection}}", &selection);
                if detached && path.file_name().is_some_and(|name| name == "Cargo.toml") {
                    contents.push_str("\n# This package is excluded from the enclosing workspace.\n[workspace]\n");
                }
                fs::write(path, finalize_managed(contents))?;
            } else {
                fs::write(path, template.bytes)?;
            }
        }
        if let Some(workspace) = workspace
            && workspace.original != workspace.updated
        {
            if fs::read_to_string(&workspace.manifest)? != workspace.original {
                return Err(Error::Invalid("workspace manifest changed during generation; retry".into()));
            }
            let mut temporary = tempfile::NamedTempFile::new_in(workspace.manifest.parent().unwrap())?;
            temporary.as_file().set_permissions(fs::metadata(&workspace.manifest)?.permissions())?;
            temporary.write_all(workspace.updated.as_bytes())?;
            temporary.flush()?;
            temporary.persist(&workspace.manifest).map_err(|error| Error::Io(error.error))?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        if let Err(rollback) = fs::remove_dir_all(&target) {
            return Err(Error::Invalid(format!("{error}; cleanup of {} failed: {rollback}", target.display())));
        }
        return Err(error);
    }
    let terminal = dokime::terminal::Terminal::default();
    terminal.success(format!("Created {name}"));
    terminal.hint(format!("cd {}", args.path.display()));
    terminal.hint("mise install, then commit mise.lock: CI installs the locked tools");
    terminal.hint(format!("cargo dokime dev{selection}"));
    Ok(())
}

/* === Private Functions === */

fn finalize_managed(contents: String) -> String {
    let normalized = contents.replace("\r\n", "\n");
    let at = if normalized.starts_with("#!") {
        normalized.find('\n').map_or(normalized.len(), |index| index + 1)
    } else if let Some(rest) = normalized.strip_prefix("---\n")
        && let Some(index) = rest.find("\n---\n")
    {
        4 + index + 5
    } else {
        0
    };
    let end = normalized[at..].find('\n').map_or(normalized.len(), |index| at + index + 1);
    let line = normalized[at..end].trim_end();
    if !line.contains("Rendered by stkr project update from ") {
        return contents;
    }
    let Some((prefix, _)) = line.rsplit_once("; sha256:") else {
        return contents;
    };
    let close = if line.ends_with(" -->") {
        " -->"
    } else if line.ends_with(" */") {
        " */"
    } else {
        ""
    };
    let body = format!("{}{}", &normalized[..at], &normalized[end..]);
    let digest = Sha256::digest(body.as_bytes());
    format!("{}{prefix}; sha256:{digest:x}{close}\n{}", &normalized[..at], &normalized[end..])
}

fn validate_name(name: &str) -> Result<(), Error> {
    const RESERVED: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl",
        "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "gen", "macro", "override", "priv",
        "try", "typeof", "unsized", "virtual", "yield", "test",
    ];
    let crate_name = name.replace('-', "_");
    let windows_name = name.to_ascii_lowercase();
    if name.is_empty()
        || !name.as_bytes()[0].is_ascii_lowercase()
        || !name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
        || RESERVED.contains(&crate_name.as_str())
        || ["con", "prn", "aux", "nul"].contains(&windows_name.as_str())
        || (1..=9).any(|n| windows_name == format!("com{n}") || windows_name == format!("lpt{n}"))
    {
        return Err(Error::Invalid(format!(
            "invalid package name {name:?}: use a lowercase letter followed by lowercase letters, digits, '-' or '_', excluding Rust keywords and reserved names"
        )));
    }
    Ok(())
}

fn workspace(target: &Path) -> Result<(Option<Workspace>, bool), Error> {
    for root in target.parent().unwrap().ancestors() {
        let manifest = root.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let original = fs::read_to_string(&manifest)?;
        let mut document =
            original.parse::<toml_edit::DocumentMut>().map_err(|error| Error::Invalid(format!("{}: {error}", manifest.display())))?;
        let Some(table) = document.get_mut("workspace") else {
            continue;
        };
        if !table.is_table() && !table.is_inline_table() {
            return Err(Error::Invalid("workspace must be a TOML table".into()));
        }
        let relative = target
            .strip_prefix(root)
            .unwrap()
            .to_str()
            .ok_or_else(|| Error::Invalid("workspace member path must be UTF-8".into()))?
            .replace('\\', "/");
        if matches(table.get("exclude"), &relative)? {
            return Ok((None, true));
        }
        if !matches(table.get("members"), &relative)? {
            if table.get("members").is_none() {
                table["members"] = toml_edit::value(toml_edit::Array::new());
            }
            table["members"].as_array_mut().ok_or_else(|| Error::Invalid("workspace.members must be an array".into()))?.push(&relative);
        }
        return Ok((Some(Workspace { manifest, updated: document.to_string(), original }), false));
    }
    Ok((None, false))
}

fn matches(item: Option<&toml_edit::Item>, relative: &str) -> Result<bool, Error> {
    let Some(item) = item else {
        return Ok(false);
    };
    let entries = item.as_array().ok_or_else(|| Error::Invalid("workspace members/exclude must be arrays".into()))?;
    for entry in entries {
        let pattern = entry.as_str().ok_or_else(|| Error::Invalid("workspace member must be a string".into()))?;
        let pattern = glob::Pattern::new(pattern.trim_start_matches("./").trim_end_matches('/'))
            .map_err(|error| Error::Invalid(error.to_string()))?;
        if pattern.matches_with(relative, glob::MatchOptions { require_literal_separator: true, ..Default::default() }) {
            return Ok(true);
        }
    }
    Ok(false)
}

/* === Tests === */
