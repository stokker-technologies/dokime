/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The sources a capture polls. Each lists its matched files with their size,
// modification time and, where the platform has one, file identity, and copies
// byte ranges of one file into a spool.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, time::SystemTime};

// Third Party Imports

// Local Imports
#[cfg(feature = "ftp")]
use super::ftp::FtpSource;
use super::local::LocalSource;
use crate::files::FileMatcher;
use crate::{runtime::Error, runtime::Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

// Matched files by forward-slash path relative to the source root.
pub(crate) type Listing = BTreeMap<String, Entry>;

/* === Structs === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) size: u64,
    pub(crate) modified: Option<SystemTime>,
    // Device and inode where the platform exposes them; a change means the
    // path now names a different file.
    pub(crate) identity: Option<(u64, u64)>,
}

/* === Enums === */

pub(super) enum Source {
    Local(LocalSource),

    #[cfg(feature = "ftp")]
    Ftp(Box<FtpSource>),
}

pub(super) enum ReadFailure {
    // The source cannot start a retrieval at a nonzero offset.
    #[cfg(feature = "ftp")]
    ResumeUnsupported,

    Failed(Error),
}

/* === Traits === */

/* === Implementations === */

impl Source {
    pub(super) async fn local(root: &str) -> Result<Self> {
        Ok(Self::Local(LocalSource::open(root).await?))
    }

    pub(super) fn single_file(self) -> Self {
        match self {
            Self::Local(local) => Self::Local(local.top_level()),
            #[cfg(feature = "ftp")]
            Self::Ftp(ftp) => Self::Ftp(Box::new(ftp.top_level())),
        }
    }

    #[cfg(feature = "ftp")]
    pub(super) async fn ftp(url: &str, password: Option<String>, trusted: bool) -> Result<Self> {
        Ok(Self::Ftp(Box::new(FtpSource::open(url, password, trusted).await?)))
    }

    pub(super) async fn changes(&self, storage: &crate::storage::Manager) -> Option<harmos::storage::DirectoryWatcher> {
        match self {
            Self::Local(source) => source.changes(storage).await,
            #[cfg(feature = "ftp")]
            Self::Ftp(_) => None,
        }
    }

    pub(super) async fn scan(&mut self, matcher: &FileMatcher) -> Result<Listing> {
        match self {
            Self::Local(source) => source.scan(matcher).await,
            #[cfg(feature = "ftp")]
            Self::Ftp(source) => source.scan(matcher).await,
        }
    }

    // Appends `length` bytes of the file from `offset` to the sink; fewer when
    // the file ends first.
    pub(super) async fn read_range(
        &mut self, relative: &str, offset: u64, length: u64, sink: &mut tokio::fs::File,
    ) -> std::result::Result<u64, ReadFailure> {
        match self {
            Self::Local(source) => source.read_range(relative, offset, length, sink).await.map_err(ReadFailure::Failed),
            #[cfg(feature = "ftp")]
            Self::Ftp(source) => source.read_range(relative, offset, length, sink).await,
        }
    }
}

impl From<Error> for ReadFailure {
    fn from(error: Error) -> Self {
        Self::Failed(error)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
