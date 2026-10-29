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

#[test]
fn resources() {
    let root = std::env::current_dir().unwrap();
    let input = tempfile::tempdir().unwrap();
    let staging = tempfile::Builder::new().prefix("dokime-installer-test-").tempdir_in(&root).unwrap();
    let files = [
        ("organization/fonts/font.woff2", b"font payload".as_slice()),
        ("plugins/host.dokime", b"plugin bundle".as_slice()),
        ("LICENSE", b"license payload".as_slice()),
        ("distribution.json", b"distribution payload".as_slice()),
    ];
    let mut resources = std::collections::HashMap::new();
    for (target, bytes) in files {
        let original = input.path().join(target);
        std::fs::create_dir_all(original.parent().unwrap()).unwrap();
        std::fs::write(&original, bytes).unwrap();
        let staged = staging.path().join(target);
        std::fs::create_dir_all(staged.parent().unwrap()).unwrap();
        std::fs::copy(original, &staged).unwrap();
        let relative = staged.strip_prefix(&root).unwrap().to_str().unwrap().to_owned();
        resources.insert(relative, target.to_owned());
    }
    let actual = tauri_utils::resources::ResourcePaths::from_map(&resources, true)
        .iter()
        .map(|resource| {
            let resource = resource.unwrap();
            assert!(resource.path().is_relative());
            assert_eq!(root.join(resource.path()), staging.path().join(resource.target()));
            (resource.target().to_owned(), std::fs::read(resource.path()).unwrap())
        })
        .collect::<std::collections::HashMap<_, _>>();
    for (target, bytes) in files {
        assert_eq!(actual[std::path::Path::new(target)], bytes);
    }
    let icon = staging.path().join("icon.ico");
    std::fs::write(&icon, b"icon payload").unwrap();
    let icons = vec![icon.strip_prefix(&root).unwrap().to_str().unwrap().to_owned()];
    let selected = tauri_utils::resources::ResourcePaths::new(&icons, false).next().unwrap().unwrap();
    assert!(selected.is_relative());
    assert_eq!(root.join(&selected), icon);
    assert_eq!(std::fs::read(selected).unwrap(), b"icon payload");
    let staged_path = staging.path().to_owned();
    staging.close().unwrap();
    assert!(!staged_path.exists());
}
