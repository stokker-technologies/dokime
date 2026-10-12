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
    io::{ErrorKind, Read, Seek},
};
// Third Party Imports

// Local Imports
use dokime_filesystem::publication as filesystem;

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

pub(crate) fn confirmation_refusal_retry() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("immutable");
    let bytes = vec![42; 512 * 1024];
    fs::write(&path, &bytes).unwrap();
    let mut total = 0;
    filesystem::confirm_with_checkpoint(&path, || {
        total += 1;
        Ok::<(), usize>(())
    })
    .unwrap();
    for cutoff in 1..=total {
        fs::write(&path, &bytes).unwrap();
        let mut calls = 0;
        let result = filesystem::confirm_with_checkpoint(&path, || {
            calls += 1;
            if calls == cutoff { Err(cutoff) } else { Ok(()) }
        });
        if calls == cutoff {
            assert!(
                matches!(result, Err(filesystem::ConfirmError::Checkpoint(value)) if value == cutoff)
            );
        } else {
            result.unwrap();
        }
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
        filesystem::confirm(&path).unwrap();
    }
}

pub(crate) fn confirmation_filesystem_error() {
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join("missing");
    let result = filesystem::confirm_with_checkpoint(&missing, || Ok::<(), u8>(()));
    match result {
        Err(filesystem::ConfirmError::Filesystem(error)) => {
            assert_eq!(error.kind(), ErrorKind::NotFound)
        }
        other => panic!("unexpected confirmation result: {other:?}"),
    }
}

pub(crate) fn install_preserves_existing() {
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().join("nested").join("evidence");
    filesystem::create_dir_all(&parent).unwrap();
    filesystem::create_dir_all(&parent).unwrap();
    let first = parent.join("first");
    let next = parent.join("next");
    let published = parent.join("published");
    fs::write(&first, b"first").unwrap();
    fs::write(&next, b"next").unwrap();
    filesystem::install(&first, &published).unwrap();
    assert_eq!(
        filesystem::install(&next, &published).unwrap_err().kind(),
        ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read(&published).unwrap(), b"first");
    let mut pinned = fs::File::open(&published).unwrap();
    filesystem::confirm(&published).unwrap();
    assert_eq!(fs::read(&published).unwrap(), b"first");
    filesystem::replace(&next, &published).unwrap();
    assert_eq!(fs::read(&published).unwrap(), b"next");
    let mut original = Vec::new();
    pinned.read_to_end(&mut original).unwrap();
    assert_eq!(original, b"first");
    #[cfg(unix)]
    {
        let link = root.path().join("substituted");
        std::os::unix::fs::symlink(&published, &link).unwrap();
        assert_eq!(
            filesystem::remove(&link).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(fs::read(&published).unwrap(), b"next");
        fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink(&parent, &link).unwrap();
        assert_eq!(
            filesystem::remove(&link.join("published"))
                .unwrap_err()
                .kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(fs::read(&published).unwrap(), b"next");
        fs::remove_file(&link).unwrap();
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        let mut blocked = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&published)
            .unwrap();
        assert!(filesystem::remove(&published).is_err());
        let mut preserved = Vec::new();
        blocked.read_to_end(&mut preserved).unwrap();
        assert_eq!(preserved, b"next");
        drop(blocked);
    }
    let mut removed = fs::File::open(&published).unwrap();
    let entries = fs::read_dir(&parent).unwrap().count();
    filesystem::remove(&published).unwrap();
    assert!(!published.exists());
    let mut retained = Vec::new();
    removed.read_to_end(&mut retained).unwrap();
    assert_eq!(retained, b"next");
    filesystem::remove(&published).unwrap();
    assert_eq!(fs::read_dir(&parent).unwrap().count(), entries - 1);
    filesystem::remove(&root.path().join("absent").join("file")).unwrap();
    original.clear();
    pinned.rewind().unwrap();
    pinned.read_to_end(&mut original).unwrap();
    assert_eq!(original, b"first");
    assert_eq!(
        filesystem::remove(&parent).unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert!(parent.is_dir());
}

pub(crate) fn settings_replace_document() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("new").join("settings.json");
    filesystem::write(&path, br#"{"version":1}"#).unwrap();
    filesystem::write(&path, br#"{"version":2,"folder":"new"}"#).unwrap();
    assert_eq!(fs::read(&path).unwrap(), br#"{"version":2,"folder":"new"}"#);
    assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
}

#[cfg(windows)]
pub(crate) fn long_unicode_paths() {
    let root = tempfile::tempdir().unwrap();
    let mut parent = root.path().join("測定 данные");
    for _ in 0..6 {
        parent = parent.join("long-directory-name-exceeding-legacy-path-limit");
    }
    let path = parent.join("settings.json");
    filesystem::write(&path, b"initial").unwrap();
    filesystem::write(&path, b"updated").unwrap();
    filesystem::confirm(&path).unwrap();
    assert_eq!(fs::read(path).unwrap(), b"updated");
}

pub(crate) fn failed_publication_cause() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("missing");
    let destination = root.path().join("published");
    let error = filesystem::install(&source, &destination).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::NotFound);
    let dokime_filesystem::Error::Publication {
        operation,
        path,
        source: cause,
    } = &error;
    assert_eq!(*operation, "flush staged file");
    assert_eq!(path, &source);
    assert_eq!(cause.kind(), ErrorKind::NotFound);
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .is_some()
    );
    let io: std::io::Error = error.into();
    assert_eq!(io.kind(), ErrorKind::NotFound);
    assert!(!destination.exists());
}

pub(crate) fn directory_migration_conflicts() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("old");
    let destination = root.path().join("new");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("owned"), b"evidence").unwrap();
    filesystem::move_directory(&source, &destination).unwrap();
    assert!(!source.exists());
    assert_eq!(fs::read(destination.join("owned")).unwrap(), b"evidence");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("different"), b"preserved").unwrap();
    assert_eq!(
        filesystem::move_directory(&source, &destination)
            .unwrap_err()
            .kind(),
        ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read(source.join("different")).unwrap(), b"preserved");
    assert_eq!(fs::read(destination.join("owned")).unwrap(), b"evidence");
}

#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: confirmation_refusal_retry");
        confirmation_refusal_retry();
    }
    {
        eprintln!("Scenario: confirmation_filesystem_error");
        confirmation_filesystem_error();
    }
    {
        eprintln!("Scenario: install_preserves_existing");
        install_preserves_existing();
    }
    {
        eprintln!("Scenario: settings_replace_document");
        settings_replace_document();
    }
    #[cfg(windows)]
    {
        eprintln!("Scenario: long_unicode_paths");
        long_unicode_paths();
    }
    {
        eprintln!("Scenario: failed_publication_cause");
        failed_publication_cause();
    }
    {
        eprintln!("Scenario: directory_migration_conflicts");
        directory_migration_conflicts();
    }
}
