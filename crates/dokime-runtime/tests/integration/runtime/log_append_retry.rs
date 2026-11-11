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
use std::io::{Read, Seek, Write};

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
fn failed_append_retries_only_unacknowledged_bytes() {
    let mut file = tempfile::tempfile().unwrap();
    file.write_all(b"saved.partial").unwrap();
    dokime_runtime::RuntimeHandle::retry_log_append(file.try_clone().unwrap(), 6, b"complete".to_vec()).unwrap();
    dokime_runtime::RuntimeHandle::retry_log_append(file.try_clone().unwrap(), 6, b"complete".to_vec()).unwrap();
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"saved.complete");
    assert!(dokime_runtime::RuntimeHandle::retry_log_append(file, 1024, b"complete".to_vec()).is_err());
}
