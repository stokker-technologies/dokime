/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Coordinate Native Credential Access Across Host Processes

/* === Modules === */

#[cfg(target_os = "windows")]
mod windows;

#[cfg(unix)]
mod unix;

/* === Re-Exports === */

#[cfg(target_os = "windows")]
pub(self) use windows::call;

#[cfg(unix)]
pub(self) use unix::call;

/* === Imports === */

// Standard Library Imports
use std::io;

// Third Party Imports
use zeroize::Zeroizing;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The Prefix of Every Native Credential Service the Host Owns, Such as `<namespace>.access`.
///
/// One host configuration value, default [`CredentialNamespace::DEFAULT`]: a host only reads, writes and
/// removes credentials inside its own namespace, so hosts in different namespaces never share an entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CredentialNamespace(String);

/// A Native Slot Whose Calls Coordinate With Other Host Processes for This User.
///
/// Every native access, including diagnostic reads, must use this entry: Windows persistent credential
/// reads can race writes to other slots and resurrect deleted entries. Calls run on the calling thread.
pub struct Entry {
    inner: keyring::Entry,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl CredentialNamespace {
    /// The Namespace Every Released Host Has Used, So Existing Credentials Stay Readable.
    pub const DEFAULT: &str = "dokime";

    /// A Namespace of ASCII Letters, Digits, Dots, Dashes and Underscores, up to 64 Bytes.
    pub fn new(namespace: impl Into<String>) -> Result<Self, String> {
        let namespace = namespace.into();
        let valid = !namespace.is_empty()
            && namespace.len() <= 64
            && namespace.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'));
        if !valid {
            return Err(format!(
                "Configure the Credential Namespace: Use 1 to 64 ASCII Letters, Digits, Dots, Dashes or Underscores, Not {namespace:?}"
            ));
        }
        Ok(Self(namespace))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// The Native Service Name of One Credential Owner Inside This Namespace.
    pub fn service(&self, owner: &str) -> String {
        format!("{}.{owner}", self.0)
    }
}

impl Default for CredentialNamespace {
    fn default() -> Self {
        Self(Self::DEFAULT.to_owned())
    }
}

impl Entry {
    pub fn new(service: &str, account: &str) -> keyring::Result<Self> {
        let inner = keyring::Entry::new(service, account)?;
        Ok(Self { inner })
    }

    pub fn get_password(&self) -> keyring::Result<String> {
        call(&self.inner, |entry| entry.get_password())
    }

    pub fn get_secret(&self) -> keyring::Result<Vec<u8>> {
        call(&self.inner, |entry| entry.get_secret())
    }

    pub fn set_password(&self, value: &str) -> keyring::Result<()> {
        call(&self.inner, |entry| entry.set_password(value))
    }

    pub fn delete_credential(&self) -> keyring::Result<()> {
        call(&self.inner, |entry| {
            entry.delete_credential()?;
            match entry.get_secret().map(Zeroizing::new) {
                Err(keyring::Error::NoEntry) => Ok(()),
                Err(error) => Err(error),
                Ok(_) => Err(keyring::Error::PlatformFailure(Box::new(io::Error::other(
                    "Remove Native Credential: Stop Other Credential Writers and Retry Removing This Root",
                )))),
            }
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
