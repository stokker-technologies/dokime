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
use std::ops::{Deref, DerefMut};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::model::records::evidence::Record;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordEntry {
    pub sequence: u64,
    #[serde(skip)]
    pub ordinal: u64,

    #[serde(skip)]
    pub captured_scopes: Vec<crate::model::execution::RecordScope>,

    #[serde(flatten)]
    pub record: Record,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos_domain::MemorySize for RecordEntry {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.record), std::mem::size_of_val(&self.record)),
            (harmos_domain::MemorySize::memory_size(&self.captured_scopes), std::mem::size_of_val(&self.captured_scopes)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl RecordEntry {
    pub fn captured_scope(&self, owner: &crate::model::identity::TestExecutionId) -> Option<&crate::model::execution::RecordScope> {
        self.captured_scopes.iter().find(|scope| scope.test_execution_id.as_ref() == Some(owner))
    }

    pub fn from_record(sequence: u64, record: Record) -> Self {
        Self { sequence, ordinal: 0, captured_scopes: Vec::new(), record }
    }
}

impl Deref for RecordEntry {
    type Target = Record;

    fn deref(&self) -> &Self::Target {
        &self.record
    }
}

impl DerefMut for RecordEntry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.record
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
