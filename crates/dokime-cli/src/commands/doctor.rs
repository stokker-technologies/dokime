/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Local Machine Diagnostics That Need No Backend

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::storage::{
    BackendFailure, ByteRange, CaptureConsistency, FileMetadata, Identifier, StorageBackend, StorageBackends, StorageFuture, Worker,
    WorkerOwner,
};

// Local Imports

/* === Constants === */

const PROBE_BYTES: &[u8] = b"dokime storage probe\n";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct StorageArgs {
    #[arg(long, help = "Data root to probe instead of the resolved Dokime data root.")]
    data_root: Option<PathBuf>,
    #[arg(long = "path", help = "Additional directory to probe; repeatable.")]
    paths: Vec<PathBuf>,
}

struct Report {
    failures: usize,
}

// Runs The Scratch Store's One Worker For The Diagnostic's Lifetime.
struct WorkerTask(tokio::sync::Mutex<Option<tokio::task::JoinHandle<dokime::harmos::storage::Result<()>>>>);

/* === Enums === */

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    #[command(about = "Probe Each Directory Dokime Writes With The Same File Operations Its Storage Uses.")]
    Storage(StorageArgs),
}

/* === Traits === */

/* === Implementations === */

impl Report {
    fn step<T, E: std::error::Error + 'static>(&mut self, location: &Path, step: &str, result: Result<T, E>) -> Option<T> {
        match result {
            Ok(value) => {
                println!("{}  {step}  OK", location.display());
                Some(value)
            }
            Err(error) => {
                self.failures += 1;
                println!("{}  {step}  FAILED: {}", location.display(), describe(&error));
                None
            }
        }
    }
}

impl WorkerOwner for WorkerTask {
    fn stop(&self) -> StorageFuture<'_, ()> {
        Box::pin(async move {
            let Some(task) = self.0.lock().await.take() else { return Ok(()) };
            let joined = task
                .await
                .map_err(|error| dokime::harmos::storage::Error::backend("doctor storage worker", BackendFailure::Permanent, error));
            joined?
        })
    }
}

/* === Public Functions === */

pub(super) async fn run(command: Command, configuration: dokime::config::Config) -> crate::Result<()> {
    match command {
        Command::Storage(args) => storage(args, configuration).await,
    }
}

/* === Private Functions === */

async fn storage(args: StorageArgs, configuration: dokime::config::Config) -> crate::Result<()> {
    let temp_root = std::env::temp_dir();
    let data_root = args.data_root.unwrap_or_else(|| configuration.output_root.clone());
    println!("Dokime Storage Diagnostics");
    println!("executable: {}", std::env::current_exe().map(|path| path.display().to_string()).unwrap_or_default());
    for name in ["TEMP", "TMP", "LOCALAPPDATA", "APPDATA", "USERPROFILE", "DOKIME_APPLICATION_ROOT", "DOKIME_OUTPUT_ROOT"] {
        println!(
            "{name}: {}",
            std::env::var_os(name).map(|value| value.to_string_lossy().into_owned()).unwrap_or_else(|| "(unset)".into())
        );
    }
    println!("temp root: {}", temp_root.display());
    println!("data root: {}", data_root.display());
    println!();

    let mut report = Report { failures: 0 };
    let mut locations = vec![temp_root.clone(), data_root.clone()];
    locations.extend(args.paths);
    for location in &locations {
        filesystem(&mut report, location);
    }
    // Storage Keeps Its Staging Beside The Data Root; Frozen Copies Always Use The Temp Root.
    let scratch_parent = data_root.parent().map(Path::to_path_buf).unwrap_or_else(|| temp_root.clone());
    harmos(&mut report, &scratch_parent, &configuration).await;

    println!();
    if report.failures == 0 {
        println!("All storage checks passed.");
        return Ok(());
    }
    Err(crate::Error::Setup(format!("{} storage checks failed", report.failures)))
}

// The Same Create, Write, Reopen, Rename And Delete Steps Dokime's Temporary And Published Files Take.
fn filesystem(report: &mut Report, location: &Path) {
    let metadata = std::fs::metadata(location);
    let Some(metadata) = report.step(location, "directory exists", metadata) else { return };
    if !metadata.is_dir() {
        println!("{}  directory exists  FAILED: not a directory", location.display());
        report.failures += 1;
        return;
    }
    let created = tempfile::Builder::new().prefix(".dokime-doctor-").tempfile_in(location);
    let Some(mut named) = report.step(location, "create file", created) else { return };
    let written = named.write_all(PROBE_BYTES).and_then(|()| named.as_file().sync_all());
    report.step(location, "write and flush", written);
    let reopened = named.reopen().and_then(read_all);
    report.step(location, "reopen open file", reopened.and_then(matches_probe));

    let target = location.join(format!(".dokime-doctor-{}", uuid::Uuid::now_v7()));
    let renamed = named.persist_noclobber(&target).map_err(|error| error.error);
    if report.step(location, "rename file", renamed).is_some() {
        report.step(location, "delete file", std::fs::remove_file(&target));
    }

    // Storage Copies Live In Files Whose Name Was Removed While Their Handle Stays Open.
    let created = tempfile::Builder::new().prefix(".dokime-doctor-").tempfile_in(location);
    let Some(mut unlinked) = report.step(location, "create second file", created) else { return };
    let written = unlinked.write_all(PROBE_BYTES).and_then(|()| unlinked.as_file().sync_all());
    report.step(location, "write second file", written);
    let removed = std::fs::remove_file(unlinked.path());
    if report.step(location, "delete open file", removed).is_some() {
        let reopened = unlinked.reopen().and_then(read_all);
        report.step(location, "reopen deleted open file", reopened.and_then(matches_probe));
    }
}

// The Exact Storage Calls That Seal Reports And Store Sidecar Artifacts, Over A Disposable Store.
async fn harmos(report: &mut Report, parent: &Path, configuration: &dokime::config::Config) {
    let scratch = tempfile::Builder::new().prefix(".dokime-doctor-").tempdir_in(parent);
    let Some(scratch) = report.step(parent, "create scratch store", scratch) else { return };
    let root = scratch.path().to_path_buf();
    let backend = dokime::storage::Backend::primary(root.clone()).await.map(Arc::new);
    let Some(backend) = report.step(&root, "open scratch store", backend) else { return };
    let state = Arc::new(dokime::harmos::storage::state::file::FileBackend::new(
        root.clone(),
        StorageBackend::history(backend.as_ref()).clone(),
        configuration.storage_state,
    ));
    let manager = dokime::storage::Manager::new(StorageBackends { primary: backend, state }, configuration.storage.clone());
    let Some(manager) = report.step(&root, "open scratch manager", manager) else { return };
    let started = manager.start_with(Arc::new(()), spawn_worker).await;
    if report.step(&root, "start scratch store", started).is_none() {
        return;
    }
    named_source(report, &manager, &root).await;
    unnamed_source(report, &manager, &root).await;
    report.step(&root, "close scratch store", manager.close().await);
    drop(manager);
    let removed = scratch.close();
    report.step(&root, "remove scratch store", removed);
}

// A Named Source, As Native Outputs Are Frozen And Their Copies Captured.
async fn named_source(report: &mut Report, manager: &dokime::storage::Manager, root: &Path) {
    let temp_root = std::env::temp_dir();
    let Some(named) = report.step(&temp_root, "storage: create named source", probe_file()) else { return };
    let Some(file) = report.step(&temp_root, "storage: reopen named source", named.reopen()) else { return };
    let frozen = manager.freeze_opened_file(file, ByteRange { offset: 0, limit: None }).await;
    let Some(frozen) = report.step(&temp_root, "storage: freeze named source", frozen) else { return };
    let captured = capture_frozen(manager, frozen, "doctor.frozen").await;
    report.step(root, "storage: capture frozen copy", captured);
}

// A Source Whose Name Is Already Removed, As Uploads And Interrupted Outputs Are Held.
async fn unnamed_source(report: &mut Report, manager: &dokime::storage::Manager, root: &Path) {
    let temp_root = std::env::temp_dir();
    let Some(named) = report.step(&temp_root, "storage: create unnamed source", probe_file()) else { return };
    let file = named.into_file();
    if let Some(duplicate) = report.step(&temp_root, "storage: duplicate unnamed source", file.try_clone()) {
        let frozen = manager.freeze_opened_file(duplicate, ByteRange { offset: 0, limit: None }).await;
        report.step(&temp_root, "storage: freeze unnamed source", frozen);
    }
    let captured = capture_opened(manager, file, "doctor.upload").await;
    report.step(root, "storage: capture unnamed upload", captured);
}

fn spawn_worker(mut worker: Worker) -> dokime::harmos::storage::Result<Box<dyn WorkerOwner>> {
    let task = tokio::spawn(async move {
        worker.run().await?;
        worker.cleanup().await
    });
    Ok(Box::new(WorkerTask(tokio::sync::Mutex::new(Some(task)))))
}

fn probe_file() -> std::io::Result<tempfile::NamedTempFile> {
    let mut file = tempfile::NamedTempFile::new()?;
    file.write_all(PROBE_BYTES)?;
    file.as_file().sync_all()?;
    Ok(file)
}

async fn capture_frozen(
    manager: &dokime::storage::Manager, frozen: dokime::harmos::storage::FrozenFile, id: &str,
) -> dokime::harmos::storage::Result<()> {
    let producer = manager.produce_file(metadata(id, frozen.length())?).await?;
    manager.capture_frozen_file(frozen, producer)?.finish().await.map(drop)
}

async fn capture_opened(manager: &dokime::storage::Manager, file: std::fs::File, id: &str) -> dokime::harmos::storage::Result<()> {
    let producer = manager.produce_file(metadata(id, PROBE_BYTES.len() as u64)?).await?;
    let range = ByteRange { offset: 0, limit: None };
    manager.capture_opened_file(file, producer, range, CaptureConsistency::Stable).finish().await.map(drop)
}

fn metadata(id: &str, length: u64) -> dokime::harmos::storage::Result<FileMetadata> {
    let id = Identifier::new(id).map_err(|_| dokime::harmos::storage::Error::InvalidInput {
        field: "doctor file",
        reason: "invalid probe identifier",
    })?;
    Ok(FileMetadata {
        id,
        scope: None,
        media_type: "application/octet-stream".to_owned(),
        expected_length: Some(length),
        expected_sha256: None,
    })
}

fn read_all(mut file: std::fs::File) -> std::io::Result<Vec<u8>> {
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn matches_probe(bytes: Vec<u8>) -> std::io::Result<()> {
    if bytes == PROBE_BYTES {
        return Ok(());
    }
    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "read back different bytes"))
}

// The Message Plus The Innermost OS Error, Which Names The Actual Windows Code.
fn describe(error: &(dyn std::error::Error + 'static)) -> String {
    let mut chain = std::iter::successors(Some(error), |error| error.source());
    let os = chain.find_map(|error| error.downcast_ref::<std::io::Error>()).and_then(std::io::Error::raw_os_error);
    match os {
        Some(code) if !error.to_string().contains("os error") => format!("{error} (os error {code})"),
        _ => error.to_string(),
    }
}

/* === Tests === */
