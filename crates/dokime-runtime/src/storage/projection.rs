/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use harmos::storage::ReadBatch;

use super::{Error, Result};

pub(crate) type Pages<T> = ReadBatch<Vec<ReadBatch<Vec<T>>>>;

pub(crate) fn push<T>(pages: Pages<T>, page: ReadBatch<Vec<T>>) -> Result<Pages<T>> {
    pages.try_map(|mut pages| {
        pages.try_reserve(1).map_err(|_| Error::Invalid("projection collection allocation failed"))?;
        pages.push(page);
        Ok(pages)
    })
}

pub(crate) fn page<T, U>(source: ReadBatch<Vec<T>>, map: impl Fn(T) -> Result<U>) -> Result<ReadBatch<Vec<U>>> {
    source.try_map(|source| {
        let mut target = Vec::new();
        target.try_reserve_exact(source.len()).map_err(|_| Error::Invalid("projection allocation failed"))?;
        for value in source {
            target.push(map(value)?);
        }
        Ok(target)
    })
}

pub(crate) fn pages<T, U>(source: Pages<T>, map: impl Fn(T) -> Result<U>) -> Result<Pages<U>> {
    source.try_map(|source| {
        let mut target = Vec::new();
        target.try_reserve_exact(source.len()).map_err(|_| Error::Invalid("projection pages allocation failed"))?;
        for source in source {
            target.push(page(source, &map)?);
        }
        Ok(target)
    })
}
