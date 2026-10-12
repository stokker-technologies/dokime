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
use std::io::Read;
use std::path::{Path, PathBuf};
// Third Party Imports

// Local Imports
use crate::error::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Discovery<'a> {
    root: &'a Path,
    paths: Vec<(String, PathBuf)>,
    maximum: usize,
    visited: usize,
}

/* === Enums === */
enum Entry {
    Directory(Box<dyn Iterator<Item = std::io::Result<PathBuf>>>),
    File,
    Other,
}

/* === Traits === */

/* === Implementations === */
impl Discovery<'_> {
    fn visit(
        &mut self,
        path: &Path,
        depth: usize,
        inspect: &impl Fn(&Path) -> std::io::Result<Entry>,
    ) -> Result<(), Error> {
        self.visited += 1;
        if self.visited > self.maximum.saturating_mul(2).saturating_add(1) || depth > 32 {
            return Err(Error::Invalid(
                "asset tree exceeds entry or depth limit".into(),
            ));
        }

        match inspect(path)? {
            Entry::Directory(children) => {
                for child in children {
                    let child = child?;
                    self.visit(&child, depth + 1, inspect)?;
                }
            }
            Entry::File => {
                if self.paths.len() >= self.maximum {
                    return Err(Error::Invalid("too many assets".into()));
                }
                let relative = path
                    .strip_prefix(self.root)
                    .expect("visited child")
                    .to_str()
                    .ok_or_else(|| Error::Invalid("asset paths must be UTF-8".into()))?
                    .replace(std::path::MAIN_SEPARATOR, "/");
                self.paths
                    .push((format!("assets/{relative}"), path.to_owned()));
            }
            Entry::Other => {
                return Err(Error::Invalid(format!(
                    "{} is not a regular asset or directory (no links)",
                    path.display()
                )));
            }
        }
        Ok(())
    }
}

/* === Public Functions === */
pub fn read(path: &Path, limit: usize) -> Result<Vec<u8>, Error> {
    if !std::fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(Error::Invalid(format!(
            "{} must be a regular file (no links)",
            path.display()
        )));
    }
    bounded(
        std::fs::File::open(path)?,
        limit,
        &path.display().to_string(),
    )
}

pub fn assets(root: &Path, maximum: usize) -> Result<Vec<(String, PathBuf)>, Error> {
    match std::fs::symlink_metadata(root) {
        Ok(_) => discover(root, maximum, &inspect),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

/* === Private Functions === */
fn bounded(reader: impl Read, limit: usize, label: &str) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    reader.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(Error::Output(label.to_owned(), limit));
    }
    Ok(bytes)
}

fn inspect(path: &Path) -> std::io::Result<Entry> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.is_dir() {
        let children = std::fs::read_dir(path)?.map(|entry| entry.map(|entry| entry.path()));
        Ok(Entry::Directory(Box::new(children)))
    } else if metadata.is_file() {
        Ok(Entry::File)
    } else {
        Ok(Entry::Other)
    }
}

fn discover(
    root: &Path,
    maximum: usize,
    inspect: &impl Fn(&Path) -> std::io::Result<Entry>,
) -> Result<Vec<(String, PathBuf)>, Error> {
    let mut discovery = Discovery {
        root,
        paths: Vec::new(),
        maximum,
        visited: 0,
    };
    discovery.visit(root, 0, inspect)?;
    discovery.paths.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(discovery.paths)
}

/* === Tests === */
