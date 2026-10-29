/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{
    fs::{File, OpenOptions, TryLockError},
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use super::{DesktopSessionError as Error, StorageResetReason as Reason};
use dokime::paths::ApplicationPaths;

const DESKTOP_STORAGE_BACKUP_PROGRESS_BYTES: u64 = 1_048_576;

// The same override the backend reads; the owned backend inherits it.
const DATA_ROOT_ENV: &str = "DOKIME_OUTPUT_ROOT";

// The lease every backend holds on its store while it runs.
const LEASE: &str = "local.lock";

// How many archive bytes pass between two progress reports.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageRoot {
    progress_step: u64,
    data: PathBuf,
    application: PathBuf,
}

struct Counted<W, F> {
    progress_step: u64,
    inner: W,
    position: u64,
    written: u64,
    reported: u64,
    report: F,
}

impl StorageRoot {
    pub fn new(data: PathBuf, application: PathBuf) -> Self {
        Self {
            data,
            application,
            progress_step: DESKTOP_STORAGE_BACKUP_PROGRESS_BYTES,
        }
    }

    pub fn resolve(application: PathBuf, progress_step: u64) -> Result<Self, String> {
        let configured = std::env::var_os(DATA_ROOT_ENV).filter(|value| !value.is_empty()).map(PathBuf::from);
        // Keep the root in the form it was configured, as the backend does;
        // canonicalizing would give Windows a verbatim prefix it never uses.
        let data = configured.unwrap_or_else(|| ApplicationPaths::new(&application).default_data());
        let data = std::path::absolute(data).map_err(|error| error.to_string())?;
        Ok(Self { data, application, progress_step })
    }

    pub fn path(&self) -> &Path {
        &self.data
    }

    pub fn verify(&self, confirmed: &Path) -> Result<(), Error> {
        if confirmed != self.data {
            return Err(refusal(
                Reason::OutsideRoot,
                format!("{} is not the data directory of this desktop's backend ({}).", confirmed.display(), self.data.display()),
            ));
        }
        if !self.data.is_absolute() || self.data.parent().is_none() || self.application.starts_with(&self.data) {
            return Err(refusal(
                Reason::OutsideRoot,
                format!("{} encloses more than this desktop's data; reset it by hand.", self.data.display()),
            ));
        }
        if std::fs::symlink_metadata(&self.data).is_ok_and(|meta| meta.file_type().is_symlink()) {
            return Err(refusal(Reason::OutsideRoot, format!("{} is a symbolic link; reset its target by hand.", self.data.display())));
        }
        Ok(())
    }

    pub fn backup_destination(&self, now: time::OffsetDateTime) -> PathBuf {
        let now = now.to_offset(time::UtcOffset::UTC);
        let name = self.data.file_name().map_or_else(|| "data".into(), |name| name.to_string_lossy());
        let stamp = format!(
            "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
            now.year(),
            u8::from(now.month()),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        let parent = self.data.parent().unwrap_or(&self.data);
        parent.join(format!("{name}-backup-{stamp}.zip"))
    }

    pub fn archive(&self, destination: &Path, progress: impl FnMut(u64)) -> Result<PathBuf, Error> {
        // Refuse A Destination The Reset Would Delete Or Overwrite
        let destination = std::path::absolute(destination).map_err(|error| archive_failure(destination, &error))?;
        if destination.starts_with(&self.data) {
            return Err(refusal(
                Reason::OutsideRoot,
                format!("{} is inside the data directory it would back up; choose another location.", destination.display()),
            ));
        }
        if std::fs::symlink_metadata(&destination).is_ok() {
            return Err(refusal(Reason::Failed, format!("{} already exists; choose another location.", destination.display())));
        }

        // Prove No Backend Holds The Store
        let _lease = self.lease("backing it up")?;

        // Stream Into A Partial Archive, Then Move It Into Place
        let mut partial = destination.clone().into_os_string();
        partial.push(".partial");
        let partial = PathBuf::from(partial);
        let file = OpenOptions::new().write(true).create_new(true).open(&partial).map_err(|error| archive_failure(&partial, &error))?;
        let written = self
            .write_archive(file, progress)
            .and_then(|()| std::fs::rename(&partial, &destination).map_err(|error| archive_failure(&destination, &error)));
        if let Err(error) = written {
            let _ = std::fs::remove_file(&partial);
            return Err(error);
        }
        Ok(destination)
    }

    pub fn clear(&self) -> Result<(), Error> {
        // Prove No Backend Holds The Store
        let lease = self.lease("resetting")?;

        // Delete The Contents
        let entries = match std::fs::read_dir(&self.data) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(failure(&self.data, &error)),
        };
        for entry in entries {
            let entry = entry.map_err(|error| failure(&self.data, &error))?;
            if entry.file_name() == LEASE {
                continue;
            }
            // The entry's own type: a link is removed, never followed.
            let kind = entry.file_type().map_err(|error| failure(&entry.path(), &error))?;
            let removed = if kind.is_dir() { std::fs::remove_dir_all(entry.path()) } else { std::fs::remove_file(entry.path()) };
            removed.map_err(|error| failure(&entry.path(), &error))?;
        }

        // Release And Remove The Lease Last
        // An open file cannot be deleted on Windows.
        if lease.is_some() {
            drop(lease);
            let path = self.data.join(LEASE);
            std::fs::remove_file(&path).map_err(|error| failure(&path, &error))?;
        }
        Ok(())
    }

    fn lease(&self, action: &str) -> Result<Option<File>, Error> {
        let lease = match OpenOptions::new().write(true).open(self.data.join(LEASE)) {
            Ok(lease) => lease,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(failure(&self.data, &error)),
        };
        match lease.try_lock() {
            Ok(()) => Ok(Some(lease)),
            Err(TryLockError::WouldBlock) => Err(refusal(
                Reason::BackendRunning,
                format!("A {} backend still holds {}; stop it before {action}.", crate::PRODUCT, self.data.display()),
            )),
            Err(TryLockError::Error(error)) => Err(failure(&self.data, &error)),
        }
    }

    fn write_archive(&self, file: File, progress: impl FnMut(u64)) -> Result<(), Error> {
        let mut zip = ZipWriter::new(Counted {
            progress_step: self.progress_step,
            inner: std::io::BufWriter::new(file),
            position: 0,
            written: 0,
            reported: 0,
            report: progress,
        });
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        // Walk The Root Depth First Without Following Links
        let mut pending = vec![(self.data.clone(), String::new())];
        while let Some((directory, prefix)) = pending.pop() {
            let entries = match std::fs::read_dir(&directory) {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound && prefix.is_empty() => {
                    continue;
                }
                Err(error) => return Err(archive_failure(&directory, &error)),
            };
            for entry in entries {
                let entry = entry.map_err(|error| archive_failure(&directory, &error))?;
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().into_owned();
                let name = format!("{prefix}{file_name}");
                let kind = entry.file_type().map_err(|error| archive_failure(&path, &error))?;
                if kind.is_dir() {
                    zip.add_directory(format!("{name}/"), options).map_err(|error| zip_failure(&path, error))?;
                    pending.push((path, format!("{name}/")));
                    continue;
                }
                let lease = prefix.is_empty() && file_name == LEASE;
                if !kind.is_file() || lease {
                    continue;
                }

                // Stream One File
                let mut source = File::open(&path).map_err(|error| archive_failure(&path, &error))?;
                let length = source.metadata().map_err(|error| archive_failure(&path, &error))?.len();
                zip.start_file(name, options.large_file(length >= u64::from(u32::MAX))).map_err(|error| zip_failure(&path, error))?;
                std::io::copy(&mut source, &mut zip).map_err(|error| archive_failure(&path, &error))?;
            }
        }

        // Finish The Central Directory And Reach The Disk
        let mut counted = zip.finish().map_err(|error| zip_failure(&self.data, error))?;
        (counted.report)(counted.written);
        let file = counted.inner.into_inner().map_err(|error| archive_failure(&self.data, error.error()))?;
        file.sync_all().map_err(|error| archive_failure(&self.data, &error))
    }
}

impl<W: Write, F: FnMut(u64)> Write for Counted<W, F> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let written = self.inner.write(buffer)?;
        self.position += written as u64;
        self.written = self.written.max(self.position);
        if self.written - self.reported >= self.progress_step {
            self.reported = self.written;
            (self.report)(self.written);
        }
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<W: Seek, F> Seek for Counted<W, F> {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.position = self.inner.seek(position)?;
        Ok(self.position)
    }
}

fn refusal(reason: Reason, message: String) -> Error {
    Error::StorageReset { reason, message }
}

fn failure(path: &Path, error: &std::io::Error) -> Error {
    refusal(Reason::Failed, format!("{} could not be deleted: {error}.", path.display()))
}

fn archive_failure(path: &Path, error: &std::io::Error) -> Error {
    refusal(Reason::Failed, format!("{} could not be archived: {error}.", path.display()))
}

fn zip_failure(path: &Path, error: zip::result::ZipError) -> Error {
    refusal(Reason::Failed, format!("{} could not be archived: {error}.", path.display()))
}
