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
use std::path::{Path, PathBuf};
// Third Party Imports

// Local Imports
use dokime_app::plugins::{PluginBundle, PluginStore};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn fixture_bundle_bytes(fixture: &str) -> Vec<u8> {
    let prepared = prepared_bundle(fixture);
    std::fs::read(&prepared).unwrap_or_else(|error| {
        panic!(
            "run mise run build:plugins:dev to prepare {}: {error}",
            prepared.display()
        )
    })
}

pub fn fixture_bundle(root: &Path, fixture: &str) -> PluginBundle {
    let path = root.join(format!("{fixture}.dokime"));
    std::fs::write(&path, fixture_bundle_bytes(fixture)).unwrap();
    PluginBundle::new(path)
}

pub fn revised_bundle(root: &Path, fixture: &str, name: &str, revision: &str) -> PluginBundle {
    let original =
        harmos::bundle::Bundle::inspect(&fixture_bundle_bytes(fixture), Default::default())
            .unwrap();
    let mut files: Vec<_> = original
        .manifest()
        .files
        .iter()
        .map(|file| harmos::bundle::Source {
            path: file.path.as_str(),
            bytes: original.file(&file.path).unwrap(),
            executable: file.executable.clone(),
        })
        .collect();
    files.push(harmos::bundle::Source {
        path: "revision.txt",
        bytes: revision.as_bytes(),
        executable: None,
    });
    let candidate = harmos::bundle::Bundle::build(
        harmos::bundle::Identity {
            name: name.into(),
            ..original.manifest().identity.clone()
        },
        &files,
        Default::default(),
    )
    .unwrap();
    let path = root.join(format!("{name}-{revision}.dokime"));
    std::fs::write(&path, candidate.bytes()).unwrap();
    PluginBundle::new(path)
}

/* === Private Functions === */
pub fn plugin_store(root: &Path) -> PluginStore {
    PluginStore::new(dokime_app::config::Settings::load(root.join("application")).unwrap())
}

pub fn prepared_bundle(fixture: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/fixtures")
        .join(fixture)
        .join(format!("{fixture}.dokime"))
}

/* === Tests === */
