/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Embed The Plugin Scaffold Assets

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    env, fs, io,
    path::{Path, PathBuf},
};

// Third Party Imports

// Local Imports


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Asset {
    source: PathBuf,
    text: bool,
}


/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn main() -> io::Result<()> {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest directory"));
    // Only This Directory Ships. Product Task Overrides And Export Registration Live Outside It.
    let templates = manifest.join("templates");
    let assets = inventory(&templates)?;
    let mut generated = String::from("const FILES: &[(&str, Template)] = &[\n");
    for (destination, asset) in assets {
        let source = asset.source.to_str().ok_or_else(|| invalid("template source path is not UTF-8"))?;
        let text = asset.text;
        generated.push_str(&format!(
            "    ({destination:?}, Template {{ bytes: include_bytes!({source:?}), text: {text} }}),\n"
        ));
    }
    generated.push_str("];\n");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo supplies the build output"));
    fs::write(output.join("template-assets.rs"), generated)
}

fn inventory(root: &Path) -> io::Result<BTreeMap<String, Asset>> {
    let mut directories = vec![root.to_owned()];
    let mut assets = BTreeMap::new();
    let mut destinations = BTreeMap::new();
    while let Some(directory) = directories.pop() {
        if fs::symlink_metadata(&directory)?.file_type().is_symlink() {
            return Err(invalid("template directories must not be symlinks"));
        }
        println!("cargo:rerun-if-changed={}", directory.display());
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                return Err(invalid("template entries must not be symlinks"));
            }
            if kind.is_dir() {
                directories.push(entry.path());
                continue;
            }
            if !kind.is_file() {
                return Err(invalid("template entries must be regular files or directories"));
            }
            let path = entry.path();
            let relative = path.strip_prefix(root).expect("the entry belongs to the template tree");
            let parts = relative
                .components()
                .map(|component| {
                    let part = component.as_os_str().to_str().ok_or_else(|| invalid("template names must be UTF-8"))?;
                    if part.contains('\\') {
                        return Err(invalid("template names must not contain backslashes"));
                    }
                    Ok(part)
                })
                .collect::<io::Result<Vec<_>>>()?;
            let relative = parts.join("/");
            let destination = relative.strip_suffix(".tmpl").unwrap_or(&relative).to_owned();
            if destination.is_empty() || destination.ends_with('/') {
                return Err(invalid("template destination must name a file"));
            }
            let key = destination.to_lowercase();
            if destinations.insert(key, destination.clone()).is_some() {
                return Err(invalid("template destinations collide after substitution suffix removal"));
            }
            let bytes = fs::read(&path)?;
            let text = std::str::from_utf8(&bytes).is_ok() && !bytes.contains(&0);
            assets.insert(destination, Asset { source: path, text });
        }
    }
    for destination in destinations.keys() {
        let mut parent = destination.as_str();
        while let Some((directory, _)) = parent.rsplit_once('/') {
            if destinations.contains_key(directory) {
                return Err(invalid("a template file also names an output directory"));
            }
            parent = directory;
        }
    }
    Ok(assets)
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}


/* === Tests === */
