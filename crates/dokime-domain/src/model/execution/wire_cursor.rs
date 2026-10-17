/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports

/* === Constants === */
pub const MAX_PAGE_ITEMS: u32 = 100;
pub const DEFAULT_PAGE_ITEMS: u32 = 50;
pub const MAX_AFTER_BYTES: usize = 4096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos_domain::message]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cursor {
    pub epoch: String,
    pub stream: String,
    pub position: u64,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("cursor epoch/stream mismatch; resynchronize without cursor")]
    Selection,
    #[error("cursor is ahead of runtime")]
    Ahead,
    #[error("runtime changed; restart pagination")]
    Changed,
    #[error("text exceeds 4096 bytes")]
    AfterTooLarge,
    #[error("page limit exceeds 100")]
    Limit,
    #[error("subsequent page requires cursor")]
    Missing,
}

/* === Traits === */

/* === Implementations === */
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
        if after.len() > MAX_AFTER_BYTES {
            return Err(Error::AfterTooLarge);
        }
        if limit > MAX_PAGE_ITEMS {
            return Err(Error::Limit);
        }
        if !after.is_empty() && given.is_none() {
            return Err(Error::Missing);
        }
        self.validate(given, true)?;
        Ok(if limit == 0 { DEFAULT_PAGE_ITEMS } else { limit } as usize)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
