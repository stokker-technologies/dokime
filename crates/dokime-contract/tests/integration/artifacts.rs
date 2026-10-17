/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Artifact names are validated relative paths shared with execution outputs.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_contract::artifacts::name::{self, InvalidName};

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
fn artifact_names_validate() {
    // Folders Are Implicit Path Segments
    for accepted in [
        "photo.jpg",
        "upload/notes.txt",
        "screenshots/2026-10-13_21-48-05-123.png",
        "a/b/c/d.wav",
    ] {
        assert_eq!(name::validate(accepted), Ok(()), "{accepted}");
    }

    // Escapes, Roots And Platform Separators Are Refused
    for refused in [
        "../escape.bin",
        "upload/../escape.bin",
        "./photo.jpg",
        "/absolute.png",
        "upload//photo.png",
        "upload/",
        "upload\\photo.png",
        "C:/photo.png",
        "line\nbreak.txt",
    ] {
        assert_eq!(name::validate(refused), Err(InvalidName::Path), "{refused}");
    }

    // The Name Bound Is Tighter Than The Execution Path Bound
    let long = format!("upload/{}", "x".repeat(255));
    assert_eq!(name::validate(""), Err(InvalidName::Length));
    assert_eq!(name::validate(&long), Err(InvalidName::Length));
    assert!(name::is_relative_path(&long));
    assert!(!name::is_relative_path(
        &"x".repeat(name::MAX_PATH_BYTES + 1)
    ));
}

#[test]
fn fresh_identities_unique() {
    let identifiers = std::thread::scope(|scope| {
        let workers = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    (0..10_000)
                        .map(|_| dokime_contract::core::ArtifactId::new())
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>()
    });
    let mut unique = std::collections::BTreeSet::new();
    for identifier in identifiers {
        assert_eq!(identifier.as_str().as_bytes()[14], b'7');
        assert!(unique.insert(identifier), "fresh identity was reused");
    }
}
