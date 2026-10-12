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
use dokime_app::config::{ApplicationOptions, Settings};

// an explicit application root keeps settings, plugins and default data local.

pub(crate) fn explicit_root_local() {
    let root = tempfile::tempdir().unwrap();
    let config = ApplicationOptions {
        application_root: Some(root.path().to_owned()),
        ..Default::default()
    }
    .resolve()
    .unwrap();
    assert_eq!(config.output_root, root.path().join("data"));
    assert_eq!(config.application_root.as_deref(), Some(root.path()));
    assert_eq!(config.plugin_lookup, None);
    assert!(root.path().join("settings.json").is_file());
    assert!(
        !config.output_root.exists(),
        "resolution must not start the backend"
    );
}

// the saved data root applies unless this process overrides it.

pub(crate) fn saved_data_overridden() {
    let root = tempfile::tempdir().unwrap();
    let saved = root.path().join("saved-data");
    Settings::load(root.path())
        .unwrap()
        .edit(|document| {
            document.data_root = Some(saved.clone());
            Ok::<_, std::io::Error>(())
        })
        .unwrap();
    let options = ApplicationOptions {
        application_root: Some(root.path().to_owned()),
        ..Default::default()
    };
    assert_eq!(options.clone().resolve().unwrap().output_root, saved);
    let overridden = ApplicationOptions {
        storage_root: Some(root.path().join("override")),
        ..options
    };
    assert_eq!(
        overridden.resolve().unwrap().output_root,
        root.path().join("override")
    );
}
