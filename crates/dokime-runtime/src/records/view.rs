/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Borrow Record Sequences Without Changing Their Storage Owner

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
#[cfg(feature = "storage")]
use crate::storage::ReadRows;
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Clone, Copy)]
pub enum View<'a> {
    Slice(&'a [RecordEntry]),
    #[cfg(feature = "storage")]
    Retained(&'a ReadRows<RecordEntry>),
}

/* === Traits === */

/* === Implementations === */

impl<'a> View<'a> {
    pub fn iter(self) -> impl DoubleEndedIterator<Item = &'a RecordEntry> + Clone {
        #[cfg(feature = "storage")]
        {
            let (slice, retained) = match self {
                Self::Slice(rows) => (rows, None),
                Self::Retained(rows) => (&[][..], Some(rows)),
            };
            slice.iter().chain(retained.into_iter().flat_map(ReadRows::iter))
        }
        #[cfg(not(feature = "storage"))]
        {
            let Self::Slice(rows) = self;
            rows.iter()
        }
    }

    pub fn len(self) -> usize {
        match self {
            Self::Slice(rows) => rows.len(),
            #[cfg(feature = "storage")]
            Self::Retained(rows) => rows.len(),
        }
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
}

impl Default for View<'_> {
    fn default() -> Self {
        Self::Slice(&[])
    }
}

impl<'a> From<&'a [RecordEntry]> for View<'a> {
    fn from(rows: &'a [RecordEntry]) -> Self {
        Self::Slice(rows)
    }
}

impl<'a> From<&'a Vec<RecordEntry>> for View<'a> {
    fn from(rows: &'a Vec<RecordEntry>) -> Self {
        Self::Slice(rows)
    }
}

#[cfg(feature = "storage")]
impl<'a> From<&'a ReadRows<RecordEntry>> for View<'a> {
    fn from(rows: &'a ReadRows<RecordEntry>) -> Self {
        Self::Retained(rows)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
