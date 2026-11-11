/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[allow(dead_code)]
#[path = "../../src/follow/receipt.rs"]
mod implementation;
/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use dokime_client::DownloadError as Error;
use implementation::{FileResume, hash, open};
use sha2::{Digest, Sha256};
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

#[tokio::test]
async fn configured_prefix_reads_preserve_receipt_authority() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("recording.bin");
    tokio::fs::write(&path, b"prefixsuffix").await.unwrap();

    let mut digest = Sha256::new();
    digest.update(b"prefix");
    let resume = FileResume {
        path: path.clone(),
        identity: Some(Default::default()),
        offset: 6,
        sha256: hash(&digest),
    };

    let (file, verified) = open(&resume).await.unwrap();
    assert_eq!(hash(&verified), resume.sha256);
    assert_eq!(file.metadata().await.unwrap().len(), resume.offset);
    drop(file);

    tokio::fs::write(&path, b"brokensuffix").await.unwrap();
    assert!(open(&resume).await.is_err());
    assert_eq!(tokio::fs::read(&path).await.unwrap(), b"brokensuffix");
}
