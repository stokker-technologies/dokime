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
use keyring::Entry;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct KeyringStore;

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The OS credential store is unavailable.")]
    Unavailable,
}

/* === Traits === */
pub trait SecretStore: Send + Sync {
    fn read(&self, endpoint: &str) -> Result<Option<Zeroizing<String>>, Error>;
    fn write(&self, endpoint: &str, token: &str) -> Result<(), Error>;
    fn remove(&self, endpoint: &str) -> Result<(), Error>;
}

/* === Implementations === */
impl SecretStore for KeyringStore {
    fn read(&self, endpoint: &str) -> Result<Option<Zeroizing<String>>, Error> {
        match entry(endpoint)?.get_password() {
            Ok(token) => Ok(Some(Zeroizing::new(token))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(Error::Unavailable),
        }
    }

    fn write(&self, endpoint: &str, token: &str) -> Result<(), Error> {
        entry(endpoint)?
            .set_password(token)
            .map_err(|_| Error::Unavailable)
    }

    fn remove(&self, endpoint: &str) -> Result<(), Error> {
        match entry(endpoint)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(Error::Unavailable),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn entry(endpoint: &str) -> Result<Entry, Error> {
    let account = Sha256::digest(endpoint.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Entry::new("com.stokker.dokime.backend", &account).map_err(|_| Error::Unavailable)
}

/* === Tests === */
