/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Own Disposable Desktop Story Roots and Credential Cleanup

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

// Third Party Imports
use futures_util::FutureExt;

// Local Imports
use dokime_runtime::CredentialNamespace;

/* === Constants === */

/* === Macros === */

tokio::task_local! {
    static ROOTS: Arc<Mutex<Vec<tempfile::TempDir>>>;
    static OWNERS: Arc<Mutex<Vec<crate::native_session::DesktopSession>>>;
}

/* === Type Aliases === */

/* === Structs === */

pub struct Root(PathBuf);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Root {
    pub fn path(&self) -> &Path {
        &self.0
    }
}

/* === Public Functions === */

pub fn fixture(name: &str) -> PathBuf {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let profile = std::env::var("TEST_PROFILE").unwrap_or_else(|_| "dev".into());
    let target = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    repository.join("target/build").join(profile).join(target).join("fixtures/host").join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

/// Retain Disposable Roots Until Every Runtime Task Has Ended, Including on Panic.
pub fn run<T>(story: impl Future<Output = T>) -> T {
    let roots = Arc::new(Mutex::new(Vec::new()));
    let owners = Arc::new(Mutex::new(Vec::new()));
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let result = runtime.block_on(ROOTS.scope(
        roots.clone(),
        OWNERS.scope(owners.clone(), async {
            let result = std::panic::AssertUnwindSafe(story).catch_unwind().await;
            let retained = std::mem::take(&mut *owners.lock().unwrap());
            for owner in retained {
                if let Err(error) = owner.shutdown().await {
                    eprintln!("Stop Desktop Story Host: {error:?}");
                }
            }
            result
        }),
    ));
    drop(runtime);

    // The Host Owns Credential Deletion; Only Then May Temporary Storage Disappear
    let mut failures = Vec::new();
    for root in std::mem::take(&mut *roots.lock().unwrap()) {
        if let Err(error) = remove_credentials(root.path()) {
            let retained = root.keep();
            failures.push(format!("{error}; Retained Root: {}", retained.display()));
        }
    }
    if !failures.is_empty() {
        let message = format!("Remove Desktop Story Credentials: {}", failures.join("; "));
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

/// The Story Process's Own Credential Namespace, So Stories Never Touch the Operator's Credentials.
///
/// A harness that removes credentials from another process names the namespace in
/// `DOKIME_CREDENTIAL_NAMESPACE`; otherwise each story process uses a unique one.
pub fn credentials() -> CredentialNamespace {
    static NAMESPACE: OnceLock<CredentialNamespace> = OnceLock::new();
    NAMESPACE
        .get_or_init(|| {
            let namespace = std::env::var("DOKIME_CREDENTIAL_NAMESPACE").unwrap_or_else(|_| {
                let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
                format!("dokime-story-{}-{unique}", std::process::id())
            });
            CredentialNamespace::new(namespace).expect("Name the Story Credential Namespace")
        })
        .clone()
}

pub fn tempdir() -> std::io::Result<Root> {
    ROOTS
        .try_with(|roots| {
            let directory = tempfile::Builder::new().prefix("dokime-desktop-story-").tempdir()?;
            let root = Root(directory.path().to_owned());
            roots.lock().unwrap().push(directory);
            Ok(root)
        })
        .map_err(|_| std::io::Error::other("Create Desktop Story Root: Run the Story Through test_support::run"))?
}

pub fn register(owner: crate::native_session::DesktopSession) {
    let _ = OWNERS.try_with(|owners| owners.lock().unwrap().push(owner));
}

/* === Private Functions === */

fn remove_credentials(root: &Path) -> std::io::Result<()> {
    let mut pending = vec![root.to_owned()];
    while let Some(directory) = pending.pop() {
        if directory.join("backend-id").is_file() {
            dokime_runtime::services::remove_credentials(&directory, &credentials())
                .map_err(|error| std::io::Error::other(format!("Remove Credentials at {}: {error}", directory.display())))?;
        }
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                pending.push(entry.path());
            }
        }
    }
    Ok(())
}

/* === Tests === */
