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
use crate::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FieldValidation {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub minimum: Option<Value>,
    pub maximum: Option<Value>,
    pub min_items: Option<usize>,
    pub max_items: Option<usize>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos::MemorySize for FieldValidation {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.pattern), std::mem::size_of_val(&self.pattern)),
            (harmos::MemorySize::memory_size(&self.minimum), std::mem::size_of_val(&self.minimum)),
            (harmos::MemorySize::memory_size(&self.maximum), std::mem::size_of_val(&self.maximum)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
