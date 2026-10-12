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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::storage::*;
use sha2::{Digest, Sha256};

// Local Imports
use super::storage::{history, id, limits, position, record, schema, view, write_batch};
use dokime::storage::sqlite::*;

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

pub(crate) async fn finished_files_reopen() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    let bytes = b"charger evidence";
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    let metadata = FileMetadata {
        id: id("trace"),
        scope: Some(id("execution")),
        media_type: "text/plain".into(),
        expected_length: Some(bytes.len() as u64),
        expected_sha256: Some(digest),
    };
    let staging = backend.create_file(&metadata).await.unwrap();
    backend.write_file(&staging, 0, &bytes[..4]).await.unwrap();
    assert!(backend.write_file(&staging, 5, b"gap").await.is_err());
    assert!(backend.write_file(&staging, 0, b"oops").await.is_err());
    backend.write_file(&staging, 0, bytes).await.unwrap();
    assert!(
        backend
            .finish_file(&staging, &position(1), bytes.len() as u64, [0; 32])
            .await
            .is_err()
    );
    assert!(
        backend
            .finish_file(&staging, &position(0), bytes.len() as u64, digest)
            .await
            .is_err()
    );
    let file = backend
        .finish_file(&staging, &position(1), bytes.len() as u64, digest)
        .await
        .unwrap();
    assert_eq!(
        backend
            .finish_file(&staging, &position(1), bytes.len() as u64, digest)
            .await
            .unwrap(),
        file
    );
    backend.abort_file(&staging).await.unwrap();
    backend.write_file(&staging, 0, bytes).await.unwrap();
    assert!(backend.write_file(&staging, 0, b"different").await.is_err());
    let saved = position(0);
    let buffered = ReadView::new(
        id("buffered-file"),
        position(1),
        saved.clone(),
        backend.read_view(&saved).await.unwrap(),
    )
    .unwrap();
    assert_eq!(
        backend
            .read_file(&buffered, &file, 0, NonZeroUsize::new(100).unwrap())
            .await
            .unwrap(),
        bytes
    );
    assert!(
        backend
            .read_files(
                &buffered,
                &ReadSelection::new(ReadKind::Files),
                None,
                limits(1)
            )
            .await
            .unwrap()
            .entries
            .is_empty()
    );
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 0);
    backend
        .write(write_batch(&[StoragePart::Files(std::slice::from_ref(
            &file,
        ))]))
        .await
        .unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 1);
    assert_eq!(
        backend
            .read_file(&buffered, &file, 0, NonZeroUsize::new(100).unwrap())
            .await
            .unwrap(),
        bytes
    );
    let current = view(&backend, 1).await;
    let control = backend.control().await.unwrap();
    control
        .call(|connection| {
            connection.execute_batch(
                "CREATE TEMP TABLE fixture_saved_file_reference AS
            SELECT * FROM storage_data_entries WHERE position='0000000000000001';
         DELETE FROM storage_data_entries WHERE position='0000000000000001';",
            )
        })
        .await
        .unwrap();
    assert!(
        backend
            .read_file(&current, &file, 0, NonZeroUsize::new(100).unwrap())
            .await
            .is_err()
    );
    assert_eq!(
        backend
            .read_file(&buffered, &file, 0, NonZeroUsize::new(100).unwrap())
            .await
            .unwrap(),
        bytes
    );
    control
        .call(|connection| {
            connection.execute_batch(
                "INSERT INTO storage_data_entries SELECT * FROM fixture_saved_file_reference;
         DROP TABLE fixture_saved_file_reference;",
            )
        })
        .await
        .unwrap();
    assert_eq!(
        backend
            .read_files(
                &current,
                &ReadSelection::new(ReadKind::Files),
                None,
                limits(1)
            )
            .await
            .unwrap()
            .entries,
        vec![file.clone()]
    );
    assert_eq!(
        backend
            .read_file(&current, &file, 4, NonZeroUsize::new(4).unwrap())
            .await
            .unwrap(),
        bytes[4..8]
    );
    backend.close().await.unwrap();
    backend.open().await.unwrap();
    let current = view(&backend, 1).await;
    assert_eq!(
        backend
            .read_file(&current, &file, 0, NonZeroUsize::new(100).unwrap())
            .await
            .unwrap(),
        bytes
    );
    backend.close().await.unwrap();
}

pub(crate) async fn sealed_files_publication() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    let bytes = b"charger trace";
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    let metadata = FileMetadata {
        id: id("sealed-trace"),
        scope: None,
        media_type: "text/plain".into(),
        expected_length: Some(bytes.len() as u64),
        expected_sha256: Some(digest),
    };
    let staging = backend.create_file(&metadata).await.unwrap();
    backend.write_file(&staging, 0, bytes).await.unwrap();
    let file = backend
        .finish_file(&staging, &position(3), bytes.len() as u64, digest)
        .await
        .unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap(), position(0));
    assert!(
        backend
            .write(write_batch(&[StoragePart::Files(std::slice::from_ref(
                &file
            ))]))
            .await
            .is_err()
    );
    backend.abort_file(&staging).await.unwrap();
    backend.close().await.unwrap();

    let reopened = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    assert_eq!(reopened.open().await.unwrap(), position(0));
    assert_eq!(reopened.create_file(&metadata).await.unwrap(), staging);
    assert_eq!(
        reopened
            .finish_file(&staging, &position(3), bytes.len() as u64, digest)
            .await
            .unwrap(),
        file
    );
    assert!(
        reopened
            .finish_file(&staging, &position(2), bytes.len() as u64, digest)
            .await
            .is_err()
    );
    let saved = position(0);
    let direct = ReadView::new(
        id("sealed-view"),
        position(3),
        saved.clone(),
        reopened.read_view(&saved).await.unwrap(),
    )
    .unwrap();
    assert_eq!(
        reopened
            .read_file(&direct, &file, 0, 64.try_into().unwrap())
            .await
            .unwrap(),
        bytes
    );
    assert!(
        reopened
            .read_files(
                &direct,
                &ReadSelection::new(ReadKind::Files),
                None,
                limits(4)
            )
            .await
            .unwrap()
            .entries
            .is_empty()
    );
    let mut forged = file.clone();
    forged.sha256 = [0; 32];
    assert!(
        reopened
            .read_file(&direct, &forged, 0, 64.try_into().unwrap())
            .await
            .is_err()
    );
    assert!(
        reopened
            .write(write_batch(&[StoragePart::Files(&[forged])]))
            .await
            .is_err()
    );

    let schemas = [schema()];
    let records = [record(2)];
    let files = [file.clone()];
    let parts = [
        StoragePart::Schemas(&schemas),
        StoragePart::Records(&records),
        StoragePart::Files(&files),
    ];
    assert_eq!(
        reopened.write(write_batch(&parts)).await.unwrap(),
        position(3)
    );
    assert_eq!(
        reopened.write(write_batch(&parts)).await.unwrap(),
        position(3)
    );
    let current = view(&reopened, 3).await;
    assert_eq!(
        reopened
            .read_files(
                &current,
                &ReadSelection::new(ReadKind::Files),
                None,
                limits(4)
            )
            .await
            .unwrap()
            .entries,
        files
    );
    assert!(
        reopened
            .read_files(
                &direct,
                &ReadSelection::new(ReadKind::Files),
                None,
                limits(4)
            )
            .await
            .unwrap()
            .entries
            .is_empty()
    );
    reopened.release_view(direct.snapshot()).await.unwrap();
    reopened.release_view(current.snapshot()).await.unwrap();
    reopened.close().await.unwrap();
}
