/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The sources a watch polls. Each lists its matched files with their size,
// modification time and, where the platform has one, file identity, and copies
// byte ranges of one file into a spool.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, time::SystemTime};

// Third Party Imports

// Local Imports
use super::{ftp::FtpSource, local::LocalSource};
use crate::{Error, Result};
use dokime_contract::operations::files::FileMatcher;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
// Matched files by forward-slash path relative to the source root.
pub(super) type Listing = BTreeMap<String, Entry>;

/* === Structs === */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Entry {
    pub(super) size: u64,
    pub(super) modified: Option<SystemTime>,
    // Device and inode where the platform exposes them; a change means the
    // path now names a different file.
    pub(super) identity: Option<(u64, u64)>,
}

/* === Enums === */
pub(super) enum Source {
    Local(LocalSource),
    Ftp(Box<FtpSource>),
}

pub(super) enum ReadFailure {
    // The source cannot start a retrieval at a nonzero offset.
    ResumeUnsupported,
    Failed(Error),
}

/* === Traits === */

/* === Implementations === */
impl Source {
    pub(super) async fn local(root: &str) -> Result<Self> {
        Ok(Self::Local(LocalSource::open(root).await?))
    }

    /// The source watching one named file reads only its parent's own entries.
    pub(super) fn single_file(self) -> Self {
        match self {
            Self::Local(local) => Self::Local(local.top_level()),
            ftp @ Self::Ftp(_) => ftp,
        }
    }

    pub(super) async fn ftp(url: &str, password: Option<String>, trusted: bool) -> Result<Self> {
        Ok(Self::Ftp(Box::new(
            FtpSource::open(url, password, trusted).await?,
        )))
    }

    pub(super) async fn scan(&mut self, matcher: &FileMatcher) -> Result<Listing> {
        match self {
            Self::Local(source) => source.scan(matcher).await,
            Self::Ftp(source) => source.scan(matcher).await,
        }
    }

    // Appends `length` bytes of the file from `offset` to the sink; fewer when
    // the file ends first.
    pub(super) async fn read_range(
        &mut self,
        relative: &str,
        offset: u64,
        length: u64,
        sink: &mut tokio::fs::File,
    ) -> std::result::Result<u64, ReadFailure> {
        match self {
            Self::Local(source) => source
                .read_range(relative, offset, length, sink)
                .await
                .map_err(ReadFailure::Failed),
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
