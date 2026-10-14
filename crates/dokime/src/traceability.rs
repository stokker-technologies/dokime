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
// Third-Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::core::{Identifier, Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceabilitySection {
    pub identifier: Identifier,
    pub label: String,
    #[serde(default)]
    pub rows: Vec<TraceabilityRow>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceabilityRow {
    pub identifier: Identifier,
    pub label: String,
    pub value: Value,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for TraceabilitySection {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.identifier),
                std::mem::size_of_val(&self.identifier),
            ),
            (
                harmos::MemorySize::memory_size(&self.label),
                std::mem::size_of_val(&self.label),
            ),
            (
                harmos::MemorySize::memory_size(&self.rows),
                std::mem::size_of_val(&self.rows),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for TraceabilityRow {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.identifier),
                std::mem::size_of_val(&self.identifier),
            ),
            (
                harmos::MemorySize::memory_size(&self.label),
                std::mem::size_of_val(&self.label),
            ),
            (
                harmos::MemorySize::memory_size(&self.value),
                std::mem::size_of_val(&self.value),
            ),
            (
                harmos::MemorySize::memory_size(&self.unit),
                std::mem::size_of_val(&self.unit),
            ),
            (
                harmos::MemorySize::memory_size(&self.source),
                std::mem::size_of_val(&self.source),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl TraceabilitySection {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            identifier: Identifier::declared(identifier, "traceability section"),
            label: label.into(),
            rows: Vec::new(),
        }
    }

    // Sections grow by statements, as a document `Section` does; a row is
    // built in one expression and appended.
    pub fn row(&mut self, row: TraceabilityRow) {
        self.rows.push(row);
    }

    pub fn rows(&mut self, rows: impl IntoIterator<Item = TraceabilityRow>) {
        self.rows.extend(rows);
    }
}

impl TraceabilityRow {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>, value: Value) -> Self {
        Self {
            identifier: Identifier::declared(identifier, "traceability row"),
            label: label.into(),
            value,
            unit: None,
            source: None,
        }
    }

    pub fn text(
        identifier: impl Into<String>,
        label: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::new(identifier, label, Value::Text(value.into()))
    }

    pub fn number(identifier: impl Into<String>, label: impl Into<String>, value: f64) -> Self {
        Self::new(identifier, label, Value::Number(value))
    }

    pub fn integer(identifier: impl Into<String>, label: impl Into<String>, value: i64) -> Self {
        Self::new(identifier, label, Value::Integer(value))
    }

    pub fn boolean(identifier: impl Into<String>, label: impl Into<String>, value: bool) -> Self {
        Self::new(identifier, label, Value::Bool(value))
    }

    #[must_use]
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    #[must_use]
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
