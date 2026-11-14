/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Diagnose Storage Without Changing Existing Roots

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::{Read, Write},
    path::PathBuf,
};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct StorageArgs {
    #[arg(long, alias = "data-root")]
    storage: Option<PathBuf>,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

#[derive(clap::Subcommand)]
pub(crate) enum Command {
    Storage(StorageArgs),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(command: Command, settings: crate::settings::Settings) -> crate::Result<()> {
    let Command::Storage(args) = command;
    let root = args.storage.or(settings.storage.clone());
    let scratch = tempfile::Builder::new().prefix("dokime-doctor-").tempdir()?;
    let mut report = serde_json::json!({"scratch": true, "existing_root": root, "checks": [], "compatible": true});
    let result: crate::Result<()> = async {
        // Exercise Publication and Readback Only in Disposable Storage
        let mut file = tempfile::NamedTempFile::new_in(scratch.path())?;
        let probe = b"dokime storage probe";
        file.write_all(probe)?;
        file.as_file().sync_all()?;
        let path = scratch.path().join("probe");
        file.persist_noclobber(&path)?;
        let bytes = std::fs::read(&path)?;
        if bytes != probe {
            return Err("Scratch Readback Differs From Written Bytes".into());
        }
        std::fs::remove_file(&path)?;
        report["checks"].as_array_mut().unwrap().push("write_sync_rename_read_remove".into());

        // Validate a Frozen Copy; Never Open an Existing Database for Writing
        let copy = scratch.path().join("storage");
        std::fs::create_dir(&copy)?;
        let mut lease = None;
        if let Some(root) = &root {
            if !root.is_dir() {
                return Err("Select an Existing Storage Directory".into());
            }
            let entries = std::fs::read_dir(root)?.collect::<std::io::Result<Vec<_>>>()?;
            if !entries.is_empty() {
                let lock = std::fs::OpenOptions::new().read(true).write(true).open(root.join("local.lock"))?;
                lock.try_lock().map_err(|error| format!("Stop the Host Before Diagnosing Its Root: {error}"))?;
                lease = Some(lock);
                if !root.join("execution.sqlite3").is_file() {
                    return Err("Incompatible Evidence Root".into());
                }
                let mut total = 0u64;
                let mut names = ["backend-id", "execution.sqlite3", "execution.sqlite3-wal", "execution.sqlite3-shm"]
                    .into_iter()
                    .map(std::path::PathBuf::from)
                    .collect::<Vec<_>>();
                if root.join("telemetry").is_dir() {
                    std::fs::create_dir(copy.join("telemetry"))?;
                    for entry in std::fs::read_dir(root.join("telemetry"))? {
                        let entry = entry?;
                        if !entry.file_type()?.is_file() {
                            return Err("Diagnose Storage: Preserve Only Regular Telemetry Files".into());
                        }
                        names.push(std::path::PathBuf::from("telemetry").join(entry.file_name()));
                    }
                }
                for name in names {
                    let path = root.join(&name);
                    if !path.exists() {
                        continue;
                    }
                    let input = std::fs::File::open(path)?;
                    total = total.saturating_add(input.metadata()?.len());
                    if total > settings.retained_bytes.get() as u64 {
                        return Err(crate::error::Error::Capacity("Raise retained_bytes to Diagnose This Database Copy".into()));
                    }
                    let mut output = std::fs::File::create(copy.join(&name))?;
                    let copied = std::io::copy(&mut input.take(settings.retained_bytes.get() as u64 + 1), &mut output)?;
                    if copied > settings.retained_bytes.get() as u64 {
                        return Err(crate::error::Error::Capacity("Database Copy Exceeds retained_bytes".into()));
                    }
                }
            }
        }
        let host = dokime_runtime_next::Host::open(&copy, settings.host(true))?;
        host.shutdown().await?;
        drop(host);
        drop(lease);
        report["checks"].as_array_mut().unwrap().push("open_close_scratch_host".into());
        Ok(())
    }
    .await;
    if let Err(error) = &result {
        report["compatible"] = false.into();
        report["error"] = error.to_string().into();
        report["recovery"] = "Back Up the Existing Root; Choose a Fresh Root or Reset It Explicitly After Backup".into();
    }
    if args.json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!("Storage Diagnostics\n{}", serde_json::to_string_pretty(&report)?);
    }
    result.map_err(|error| match error {
        crate::error::Error::Capacity(_) => error,
        error => format!("{error}; Back Up the Existing Root and Choose a Fresh Root or Reset It Explicitly After Backup").into(),
    })
}

/* === Private Functions === */

/* === Tests === */
