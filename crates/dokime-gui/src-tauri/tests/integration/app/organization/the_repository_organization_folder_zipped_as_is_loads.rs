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

use super::*;

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
fn the_repository_organization_folder_zipped_as_is_loads() {
    for prefix in ["", "organization/"] {
        let files = validate_archive(&zip_folder(&organization_folder(), prefix)).unwrap();
        assert!(files.contains_key(Path::new("organization.toml")));
        assert!(files.keys().all(|path| !path.starts_with("generated")));
        assert!(!files.contains_key(Path::new("fonts/OFL.txt")));
        let root = tempfile::tempdir().unwrap();
        for (relative, contents) in &files {
            let path = root.path().join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, contents).unwrap();
        }
        let pack = load(root.path()).unwrap();
        assert_eq!(pack["label"], "Dokime");
        assert_eq!(pack["logos"].as_object().unwrap().len(), 4);
        assert!(pack["css"].as_str().unwrap().contains("data:font/woff2;base64,"));
        assert!(pack["css"].as_str().unwrap().contains("--brand-radius-sm"));
        assert!(!pack["css"].as_str().unwrap().contains("url(\"fonts/"));
    }
}
