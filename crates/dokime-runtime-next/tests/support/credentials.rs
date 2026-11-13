/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Isolated Operating System Credential Store for Host Stories

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{ffi::OsString, path::Path};

// Third Party Imports

// Local Imports

/* === Constants === */

#[cfg(target_os = "linux")]
const STARTUP_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct CredentialStore {
    pub environment: Vec<(&'static str, OsString)>,

    #[cfg(target_os = "linux")]
    children: Vec<std::process::Child>,

    #[cfg(target_os = "linux")]
    _home: tempfile::TempDir,
}

pub struct Root(Option<tempfile::TempDir>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Root {
    pub fn path(&self) -> &Path {
        self.0.as_ref().unwrap().path()
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        #[cfg(windows)]
        if let Err(error) = remove_credentials(self.path()) {
            let retained = self.0.take().unwrap().keep();
            if std::thread::panicking() {
                eprintln!("Remove Story Credentials: {error}; Retained Root: {}", retained.display());
            } else {
                panic!("Remove Story Credentials: {error}; Retained Root: {}", retained.display());
            }
        }
    }
}

impl CredentialStore {
    pub async fn start(root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = root;
            Ok(Self { environment: Vec::new() })
        }
        #[cfg(target_os = "linux")]
        {
            use std::{io::Write, process::Stdio};
            use tokio::process::Command;

            // Isolate Every Credential and Socket Under This Story's Root
            let home = tempfile::tempdir_in(root)?;
            let mut store = Self {
                environment: ["HOME", "XDG_DATA_HOME", "XDG_RUNTIME_DIR"]
                    .into_iter()
                    .map(|key| (key, home.path().as_os_str().to_owned()))
                    .collect(),
                children: Vec::new(),
                _home: home,
            };

            // Start a Private Bus Without Inheriting the Desktop Session
            let address_file = store._home.path().join("bus-address");
            let mut command = dokime_runtime_next::process::command("dbus-daemon");
            command
                .args(["--session", "--nofork", "--print-address=1"])
                .envs(store.environment.iter().cloned())
                .stdin(Stdio::null())
                .stdout(std::fs::File::create(&address_file)?)
                .stderr(Stdio::inherit());
            store.spawn(command, "dbus")?;
            let address = loop {
                store.check_running()?;
                let address = std::fs::read_to_string(&address_file)?;
                if address.ends_with('\n') {
                    break address.trim().to_owned();
                }
                tokio::time::sleep(STARTUP_POLL_INTERVAL).await;
            };
            store.environment.push(("DBUS_SESSION_BUS_ADDRESS", address.into()));

            // Unlock a Disposable Keyring Before Starting the Host
            let mut command = dokime_runtime_next::process::command("gnome-keyring-daemon");
            command
                .args(["--foreground", "--components=secrets", "--unlock", "--control-directory"])
                .arg(store._home.path())
                .envs(store.environment.iter().cloned())
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::inherit());
            store.spawn(command, "gnome-keyring")?;
            store.children.last_mut().unwrap().stdin.take().unwrap().write_all(b"host-story-keyring-password")?;
            loop {
                store.check_running()?;
                let result = Command::from(dokime_runtime_next::process::command("dbus-send"))
                    .args([
                        "--session",
                        "--print-reply",
                        "--dest=org.freedesktop.DBus",
                        "/org/freedesktop/DBus",
                        "org.freedesktop.DBus.NameHasOwner",
                        "string:org.freedesktop.secrets",
                    ])
                    .envs(store.environment.iter().cloned())
                    .kill_on_drop(true)
                    .output()
                    .await
                    .map_err(|error| format!("Start Story Credential Store: Install the dbus Package: {error}"))?;
                if result.status.success() && String::from_utf8_lossy(&result.stdout).contains("boolean true") {
                    break;
                }
                tokio::time::sleep(STARTUP_POLL_INTERVAL).await;
            }
            Ok(store)
        }
    }

    #[cfg(target_os = "linux")]
    fn spawn(&mut self, mut command: std::process::Command, package: &str) -> Result<(), Box<dyn std::error::Error>> {
        let child = command.spawn().map_err(|error| format!("Start Story Credential Store: Install the {package} Package: {error}"))?;
        self.children.push(child);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn check_running(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for child in &mut self.children {
            if let Some(status) = child.try_wait()? {
                return Err(format!("Story Credential Store Exited During Startup: {status}").into());
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Drop for CredentialStore {
    fn drop(&mut self) {
        // Reap Only Owned Daemons Before Removing Their Private Storage
        for child in self.children.iter_mut().rev() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/* === Public Functions === */

pub fn tempdir() -> std::io::Result<Root> {
    tempfile::tempdir().map(|root| Root(Some(root)))
}

/* === Private Functions === */

#[cfg(windows)]
fn remove_credentials(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if root.join("backend-id").is_file() {
        // A Dropped Child Has Been Killed; Await Release Before Discarding Its Root.
        let lease = std::fs::OpenOptions::new().read(true).write(true).open(root.join("local.lock"))?;
        lease.lock()?;
        drop(lease);
        dokime_runtime_next::services::remove_credentials(root)?;
        return Ok(());
    }
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            remove_credentials(&entry.path())?;
        }
    }
    Ok(())
}

/* === Tests === */
