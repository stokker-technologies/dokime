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
use harmos::storage::ReadBatch;
use serde::{Deserialize, Serialize, ser::SerializeSeq};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// An Immutable Sequence Of Owned Rows Or Shared Native Pages.
pub struct ReadRows<T> {
    source: Rows<T>,
}

pub struct RowsIter<'a, T> {
    pages: std::slice::Iter<'a, ReadBatch<Vec<T>>>,
    current: std::slice::Iter<'a, T>,
    back: std::slice::Iter<'a, T>,
}

/* === Enums === */

enum Rows<T> {
    Owned(Vec<T>),
    Native(super::projection::Pages<T>),
}

/* === Traits === */

/* === Implementations === */

impl<T> ReadRows<T> {
    pub fn get(&self, mut index: usize) -> Option<&T> {
        match &self.source {
            Rows::Owned(rows) => rows.get(index),
            Rows::Native(pages) => {
                for page in pages.iter() {
                    if index < page.len() {
                        return page.get(index);
                    }
                    index -= page.len();
                }
                None
            }
        }
    }

    pub fn iter(&self) -> RowsIter<'_, T> {
        match &self.source {
            Rows::Owned(rows) => RowsIter { pages: [].iter(), current: rows.iter(), back: [].iter() },
            Rows::Native(pages) => RowsIter { pages: pages.iter(), current: [].iter(), back: [].iter() },
        }
    }

    pub fn len(&self) -> usize {
        match &self.source {
            Rows::Owned(rows) => rows.len(),
            Rows::Native(pages) => pages.iter().map(|page| page.len()).sum(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }
}

impl<'a, T> Iterator for RowsIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(row) = self.current.next() {
                return Some(row);
            }
            let Some(page) = self.pages.next() else {
                return self.back.next();
            };
            self.current = page.iter();
        }
    }
}

impl<'a, T> Clone for RowsIter<'a, T> {
    fn clone(&self) -> Self {
        Self { pages: self.pages.clone(), current: self.current.clone(), back: self.back.clone() }
    }
}

impl<T> DoubleEndedIterator for RowsIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(row) = self.back.next_back() {
                return Some(row);
            }
            let Some(page) = self.pages.next_back() else {
                return self.current.next_back();
            };
            self.back = page.iter();
        }
    }
}

impl<T: PartialEq> PartialEq for ReadRows<T> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'a, T> IntoIterator for &'a ReadRows<T> {
    type Item = &'a T;
    type IntoIter = RowsIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Clone> Clone for ReadRows<T> {
    fn clone(&self) -> Self {
        match &self.source {
            Rows::Owned(rows) => Self::from(rows.clone()),
            Rows::Native(pages) => Self::from(pages.clone()),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ReadRows<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Default for ReadRows<T> {
    fn default() -> Self {
        Self::from(Vec::new())
    }
}

impl<T> From<Vec<T>> for ReadRows<T> {
    fn from(rows: Vec<T>) -> Self {
        Self { source: Rows::Owned(rows) }
    }
}

impl<T> From<super::projection::Pages<T>> for ReadRows<T> {
    fn from(pages: super::projection::Pages<T>) -> Self {
        Self { source: Rows::Native(pages) }
    }
}

impl<T: Serialize> Serialize for ReadRows<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.len()))?;
        for row in self {
            sequence.serialize_element(row)?;
        }
        sequence.end()
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for ReadRows<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from(Vec::<T>::deserialize(deserializer)?))
    }
}

impl<T: harmos::MemorySize> harmos::MemorySize for ReadRows<T> {
    fn memory_size(&self) -> usize {
        let heap = match &self.source {
            Rows::Owned(rows) => rows.memory_size().saturating_sub(std::mem::size_of_val(rows)),
            Rows::Native(pages) => pages.memory_size().saturating_sub(std::mem::size_of_val(pages)),
        };
        std::mem::size_of::<Self>().saturating_add(heap)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_preserve_page_order_and_flat_serialization() {
        let empty = ReadRows::<u32>::default();
        assert!(empty.is_empty());
        assert_eq!(empty.get(0), None);
        let pages =
            ReadBatch::new(vec![ReadBatch::new(vec![]), ReadBatch::new(vec![1, 2]), ReadBatch::new(vec![]), ReadBatch::new(vec![3])]);
        let rows = ReadRows::from(pages);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows.iter().copied().collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(rows.iter().rev().copied().collect::<Vec<_>>(), [3, 2, 1]);
        let mut mixed = rows.iter();
        assert_eq!(mixed.next(), Some(&1));
        assert_eq!(mixed.next_back(), Some(&3));
        assert_eq!(mixed.next(), Some(&2));
        assert_eq!(mixed.next_back(), None);
        assert_eq!(rows.clone(), rows);
        assert_eq!(serde_json::to_string(&rows).unwrap(), "[1,2,3]");
        assert_eq!(serde_json::from_str::<ReadRows<u32>>("[1,2,3]").unwrap(), rows);
    }
}
