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
use super::ConnectionError;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct KeyringStore;

/* === Enums === */

/* === Traits === */
pub(super) trait SecretStore: Send + Sync {
    fn read(&self, endpoint: &str) -> Result<Option<Zeroizing<String>>, ConnectionError>;
    fn write(&self, endpoint: &str, token: &str) -> Result<(), ConnectionError>;
}

/* === Implementations === */
impl SecretStore for KeyringStore {
    fn read(&self, endpoint: &str) -> Result<Option<Zeroizing<String>>, ConnectionError> {
        match entry(endpoint)?.get_password() {
            Ok(token) => Ok(Some(Zeroizing::new(token))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(ConnectionError::CredentialsUnavailable),
        }
    }

    fn write(&self, endpoint: &str, token: &str) -> Result<(), ConnectionError> {
        entry(endpoint)?
            .set_password(token)
            .map_err(|_| ConnectionError::CredentialsUnavailable)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn entry(endpoint: &str) -> Result<Entry, ConnectionError> {
    let account = Sha256::digest(endpoint.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Entry::new("com.stokker.dokime.backend", &account)
        .map_err(|_| ConnectionError::CredentialsUnavailable)
}

/* === Tests === */
