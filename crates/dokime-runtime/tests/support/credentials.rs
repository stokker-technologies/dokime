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
#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(target_os = "linux")]
use std::process::{Child, Command as ProcessCommand};
use std::{
    error::Error,
    ffi::OsString,
    io::Result as IoResult,
    path::Path,
    sync::{Arc, Mutex},
};
use tempfile::TempDir;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

tokio::task_local! {
    static ROOTS: Arc<Mutex<Vec<TempDir>>>;
}

/* === Type Aliases === */

/* === Structs === */

pub struct CredentialStore {
    pub environment: Vec<(&'static str, OsString)>,

    #[cfg(target_os = "linux")]
    children: Vec<(&'static str, Child)>,

    #[cfg(target_os = "linux")]
    _home: TempDir,
}

pub struct Root(Option<TempDir>);

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
        if let Err(error) = remove_credentials(self.path(), false) {
            let root = self.0.take().unwrap();
            let mut root = Some(root);
            if ROOTS.try_with(|roots| roots.lock().unwrap().push(root.take().unwrap())).is_ok() {
                return;
            }
            let retained = root.unwrap().keep();
            if std::thread::panicking() {
                eprintln!("Remove Story Credentials: {error}; Retained Root: {}", retained.display());
            } else {
                panic!("Remove Story Credentials: {error}; Retained Root: {}", retained.display());
            }
        }
    }
}

impl CredentialStore {
    pub async fn start(root: &Path) -> Result<Self, Box<dyn Error>> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = root;
            Ok(Self { environment: Vec::new() })
        }
        #[cfg(target_os = "linux")]
        {
            use std::{
                fs::Permissions,
                io::{BufRead, BufReader, Error as IoError, Write},
                os::unix::fs::PermissionsExt,
                process::Stdio,
            };
            use tokio::process::Command;

            // Isolate Every Credential and Socket Under This Story's Root; the Keyring Accepts Only a 0700 Control Directory
            let home = tempfile::Builder::new().permissions(Permissions::from_mode(0o700)).tempdir_in(root)?;
            let mut store = Self {
                environment: ["HOME", "XDG_DATA_HOME", "XDG_RUNTIME_DIR"]
                    .into_iter()
                    .map(|key| (key, home.path().as_os_str().to_owned()))
                    .collect(),
                children: Vec::new(),
                _home: home,
            };

            // Start a Private Bus Without Inheriting the Desktop Session
            let mut command = dokime_runtime::process::command("dbus-daemon");
            command
                .args(["--session", "--nofork", "--print-address=1"])
                .envs(store.environment.iter().cloned())
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit());
            store.spawn(command, "dbus")?;
            let stdout = store.children.last_mut().unwrap().1.stdout.take().ok_or("Start Story Bus: Missing Readiness Pipe")?;
            let address = tokio::task::spawn_blocking(move || {
                let mut address = String::new();
                BufReader::new(stdout).read_line(&mut address)?;
                Ok::<String, IoError>(address)
            })
            .await??;
            store.check_running()?;
            if !address.ends_with('\n') {
                return Err("Start Story Bus: Missing Readiness Event".into());
            }
            let address = address.trim().to_owned();
            store.environment.push(("DBUS_SESSION_BUS_ADDRESS", address.into()));

            // Unlock a Disposable Keyring Before Starting the Host
            let mut command = dokime_runtime::process::command("gnome-keyring-daemon");
            command
                .args(["--foreground", "--components=secrets", "--unlock", "--control-directory"])
                .arg(store._home.path())
                .envs(store.environment.iter().cloned())
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::inherit());
            store.spawn(command, "gnome-keyring")?;
            store.children.last_mut().unwrap().1.stdin.take().unwrap().write_all(b"host-story-keyring-password")?;
            loop {
                store.check_running()?;
                let result = Command::from(dokime_runtime::process::command("dbus-send"))
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
            }
            Ok(store)
        }
    }

    #[cfg(target_os = "linux")]
    fn spawn(&mut self, mut command: ProcessCommand, package: &'static str) -> Result<(), Box<dyn Error>> {
        let child = command.spawn().map_err(|error| format!("Start Story Credential Store: Install the {package} Package: {error}"))?;
        self.children.push((package, child));
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn check_running(&mut self) -> Result<(), Box<dyn Error>> {
        for (package, child) in &mut self.children {
            if let Some(status) = child.try_wait()? {
                return Err(format!(
                    "Start Story Credential Store: The {package} Daemon Exited During Startup ({status}); See Its Output Above"
                )
                .into());
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Drop for CredentialStore {
    fn drop(&mut self) {
        // Reap Only Owned Daemons Before Removing Their Private Storage
        for (_, child) in self.children.iter_mut().rev() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/* === Public Functions === */

pub fn run<T>(story: std::pin::Pin<Box<impl std::future::Future<Output = T>>>) -> T {
    let roots = Arc::new(Mutex::new(Vec::new()));
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.block_on(ROOTS.scope(roots.clone(), story))));
    drop(runtime);
    #[cfg(windows)]
    let mut failures = Vec::<String>::new();
    #[cfg(not(windows))]
    let failures = Vec::<String>::new();
    for root in std::mem::take(&mut *roots.lock().unwrap()) {
        #[cfg(windows)]
        if let Err(error) = remove_credentials(root.path(), true) {
            let retained = root.keep();
            failures.push(format!("{error}; Retained Root: {}", retained.display()));
        }
        #[cfg(not(windows))]
        drop(root);
    }
    if !failures.is_empty() {
        let message = format!("Remove Story Credentials After Runtime Teardown: {}", failures.join("; "));
        if result.is_err() {
            eprintln!("{message}");
        } else {
            panic!("{message}");
        }
    }
    match result {
        Ok(value) => value,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

pub fn tempdir() -> IoResult<Root> {
    tempfile::tempdir().map(|root| Root(Some(root)))
}

/* === Private Functions === */

#[cfg(windows)]
fn remove_credentials(root: &Path, runtime_closed: bool) -> Result<(), Box<dyn Error>> {
    if root.join("backend-id").is_file() {
        // Runtime-Owned Work Must Release the Root Before Credential Disposal.
        let lease = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.join("local.lock"))?;
        if runtime_closed {
            lease.lock()?;
        } else {
            lease.try_lock()?;
        }
        drop(lease);
        dokime_runtime::services::remove_credentials(root)?;
    }
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            remove_credentials(&entry.path(), runtime_closed)?;
        }
    }
    Ok(())
}

/* === Tests === */
