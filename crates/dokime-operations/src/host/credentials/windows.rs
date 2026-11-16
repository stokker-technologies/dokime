/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Serialize Every Native Credential Call Across Processes for the Current User

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{env, fs, fs::OpenOptions, io, path::PathBuf};

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

pub(crate) fn call<T>(entry: &keyring::Entry, operation: impl FnOnce(&keyring::Entry) -> keyring::Result<T>) -> keyring::Result<T> {
    // Persistent Credential Reads Also Race Writes to Unrelated Slots and Namespaces.
    // One User-Owned File Coordinates All Sessions; a Namespace Lock Would Be Too Narrow.
    let directory = env::var_os("LOCALAPPDATA")
        .ok_or_else(|| unavailable(io::Error::other("Locate Local Application Data: Restore LOCALAPPDATA Before Opening the Host")))?;
    let directory = PathBuf::from(directory).join("dokime");
    fs::create_dir_all(&directory).map_err(unavailable)?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(directory.join("native-credentials.lock"))
        .map_err(unavailable)?;
    file.lock().map_err(unavailable)?;
    let result = operation(entry);
    // Closing a Locked Handle Releases Its Lock Only Eventually on Windows.
    file.unlock().map_err(unavailable)?;
    result
}

/* === Private Functions === */

fn unavailable(source: io::Error) -> keyring::Error {
    let error = io::Error::new(
        source.kind(),
        format!("Coordinate Native Credentials: Restore Access to Local Application Data and Retry: {source}"),
    );
    keyring::Error::PlatformFailure(Box::new(error))
}

/* === Tests === */
