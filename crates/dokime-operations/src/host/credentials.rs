/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Serialize Native Credentials on Their Owning Thread

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{OnceLock, mpsc};
use std::thread;

// Third Party Imports
use zeroize::Zeroizing;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Work = Box<dyn FnOnce() + Send>;

/* === Structs === */

/// The Prefix of Every Native Credential Service the Host Owns, Such as `<namespace>.access`.
///
/// One host configuration value, default [`CredentialNamespace::DEFAULT`]: a host only reads, writes and
/// removes credentials inside its own namespace, so hosts in different namespaces never share an entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CredentialNamespace(String);

/// A Native Slot Whose Calls Always Run on the Credential Owner Thread.
pub struct Entry {
    service: String,
    account: String,
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
        Ok(Self {
            service: service.into(),
            account: account.into(),
        })
    }

    pub fn get_password(&self) -> keyring::Result<String> {
        self.call(|entry| entry.get_password())
    }

    pub fn get_secret(&self) -> keyring::Result<Vec<u8>> {
        self.call(|entry| entry.get_secret())
    }

    pub fn set_password(&self, value: &str) -> keyring::Result<()> {
        let value = Zeroizing::new(value.to_owned());
        self.call(move |entry| entry.set_password(&value))
    }

    pub fn delete_credential(&self) -> keyring::Result<()> {
        self.call(|entry| {
            entry.delete_credential()?;
            match entry.get_secret().map(Zeroizing::new) {
                Err(keyring::Error::NoEntry) => Ok(()),
                Err(error) => Err(error),
                Ok(_) => Err(keyring::Error::PlatformFailure(Box::new(std::io::Error::other(
                    "Remove Native Credential: Credential Remains in the Native Store",
                )))),
            }
        })
    }

    fn call<T: Send + 'static>(&self, operation: impl FnOnce(keyring::Entry) -> keyring::Result<T> + Send + 'static) -> keyring::Result<T> {
        // One Process-Lifetime Owner Also Covers Root Reopen and Explicit Discard.
        static OWNER: OnceLock<mpsc::Sender<Work>> = OnceLock::new();
        let owner = OWNER.get_or_init(|| {
            let (send, receive) = mpsc::channel::<Work>();
            thread::Builder::new()
                .name("dokime-credentials".into())
                .spawn(move || {
                    for work in receive {
                        work();
                    }
                })
                .expect("Start Native Credential Owner");
            send
        });
        let service = self.service.clone();
        let account = self.account.clone();
        let (send, receive) = mpsc::sync_channel(1);
        owner
            .send(Box::new(move || {
                let result = keyring::Entry::new(&service, &account).and_then(operation);
                let _ = send.send(result);
            }))
            .map_err(|error| {
                keyring::Error::PlatformFailure(Box::new(std::io::Error::other(format!("Call Native Credential Owner: {error}"))))
            })?;
        receive.recv().map_err(|error| {
            keyring::Error::PlatformFailure(Box::new(std::io::Error::other(format!("Await Native Credential Owner: {error}"))))
        })?
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
