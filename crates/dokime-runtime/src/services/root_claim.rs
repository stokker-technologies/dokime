/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bind a Root's Credential Identity to the One Folder That Owns It

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::{Path, PathBuf};

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, Result};
use dokime_operations::host::credentials::Entry;
use dokime_storage::Store;

/* === Constants === */

// One native entry per backend identity names the folder that owns its credential accounts.
const SERVICE: &str = "dokime.root";
const CREDENTIAL_RECORDS: [&str; 4] = ["secrets", "secret-key-slots", "access", "access-key-slots"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Any administration record decodes as this; only its presence matters.
#[derive(Clone, PartialEq, Message)]
struct Present {}

/* === Enums === */

enum Owner {
    This,
    Copied(PathBuf),
    Unclaimed,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Claims the backend identity for this folder at open, refusing a copied folder that holds credential records.
///
/// A folder copy carries `backend-id` and the credential catalogs, but not the native owner entry, so the
/// original folder stays the only one that may read, rotate or remove those accounts. A moved root reclaims.
pub(crate) fn claim(root: &Path, store: &Store) -> Result<()> {
    match owner(root, store.identity())? {
        Owner::This => Ok(()),
        Owner::Copied(original) if holds_credentials(store)? => Err(copied("Open Evidence Root", root, &original)),
        Owner::Copied(_) => Ok(()),
        Owner::Unclaimed => entry(store.identity())?.set_password(&canonical(root)?).map_err(unavailable),
    }
}

/// Checks that this folder owns the credentials it is about to remove.
pub(crate) fn require_owner(root: &Path, store: &Store) -> Result<()> {
    if !holds_credentials(store)? {
        return Ok(());
    }
    match owner(root, store.identity())? {
        Owner::This => Ok(()),
        Owner::Copied(original) => Err(copied("Remove Credentials", root, &original)),
        Owner::Unclaimed => Err(Error::Invalid(format!(
            "Remove Credentials: Open {} With the Host Once So It Claims Its Credentials, Then Retry",
            root.display()
        ))),
    }
}

/// Releases the claim after this folder's credentials are removed.
pub(crate) fn release(root: &Path, host_id: &str) -> Result<()> {
    if let Owner::This = owner(root, host_id)? {
        entry(host_id)?.delete_credential().map_err(unavailable)?;
    }
    Ok(())
}

/// Forgets credential records in a diagnostic copy without touching any native account.
pub fn detach_credentials(copy: &Path) -> Result<()> {
    let store = Store::open(copy)?;
    for record in CREDENTIAL_RECORDS {
        store.delete_administration(record)?;
    }
    Ok(())
}

/* === Private Functions === */

fn owner(root: &Path, host_id: &str) -> Result<Owner> {
    let path = canonical(root)?;
    let claimed = match entry(host_id)?.get_password() {
        Ok(claimed) => claimed,
        Err(keyring::Error::NoEntry) => return Ok(Owner::Unclaimed),
        Err(error) => return Err(unavailable(error)),
    };
    if claimed == path {
        return Ok(Owner::This);
    }
    // The Claimed Folder Still Holds the Same Identity: This Folder Is Its Copy.
    let original = PathBuf::from(&claimed);
    if std::fs::read_to_string(original.join("backend-id")).is_ok_and(|identity| identity == host_id) {
        return Ok(Owner::Copied(original));
    }
    Ok(Owner::Unclaimed)
}

fn holds_credentials(store: &Store) -> Result<bool> {
    for record in CREDENTIAL_RECORDS {
        if store.administration::<Present>(record)?.is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn canonical(root: &Path) -> Result<String> {
    let path = std::fs::canonicalize(root).map_err(|source| Error::RootIo { path: root.to_owned(), source })?;
    Ok(path.to_string_lossy().into_owned())
}

fn entry(host_id: &str) -> Result<Entry> {
    Entry::new(SERVICE, host_id).map_err(unavailable)
}

fn copied(action: &str, root: &Path, original: &Path) -> Error {
    Error::Invalid(format!(
        "{action}: {} Is a Copy of the Root at {}; Use That Root, or Copy Evidence Into a Fresh Root",
        root.display(),
        original.display()
    ))
}

fn unavailable(error: keyring::Error) -> Error {
    Error::Invalid(format!("Claim Root Credentials: Unlock the Credential Store and Retry: {error}"))
}

/* === Tests === */
