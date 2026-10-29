/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

pub const PAGE_ITEMS: u64 = 50;
pub const MAXIMUM_PAGE_ITEMS: u64 = 500;
pub const AFTER_BYTES: u64 = 4_096;

#[harmos::message]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cursor {
    pub epoch: String,
    pub stream: String,
    pub position: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("cursor epoch/stream mismatch; resynchronize without cursor")]
    Selection,

    #[error("cursor is ahead of runtime")]
    Ahead,

    #[error("runtime changed; restart pagination")]
    Changed,

    #[error("cursor text exceeds the configured byte limit")]
    AfterTooLarge,

    #[error("page limit exceeds the configured item limit")]
    Limit,

    #[error("subsequent page requires cursor")]
    Missing,
}

impl Cursor {
    pub fn validate(&self, given: Option<&Self>, exact: bool) -> Result<(), Error> {
        let Some(given) = given else { return Ok(()) };
        if given.epoch != self.epoch || given.stream != self.stream {
            return Err(Error::Selection);
        }
        if given.position > self.position {
            return Err(Error::Ahead);
        }
        if exact && given.position != self.position {
            return Err(Error::Changed);
        }
        Ok(())
    }

    pub fn page(&self, limit: u32, after: &str, given: Option<&Self>) -> Result<usize, Error> {
        let limit = page_size(limit, after)?;
        if !after.is_empty() && given.is_none() {
            return Err(Error::Missing);
        }
        self.validate(given, true)?;
        Ok(limit)
    }
}

pub fn page_size(limit: u32, after: &str) -> Result<usize, Error> {
    let after_bytes = u64::try_from(after.len()).map_err(|_| Error::AfterTooLarge)?;
    if after_bytes > AFTER_BYTES {
        return Err(Error::AfterTooLarge);
    }
    let limit = if limit == 0 { PAGE_ITEMS } else { u64::from(limit) };
    if limit == 0 || limit > MAXIMUM_PAGE_ITEMS {
        return Err(Error::Limit);
    }
    usize::try_from(limit).map_err(|_| Error::Limit)
}
