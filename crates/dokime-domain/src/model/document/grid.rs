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
use std::collections::BTreeSet;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::model::expressions::DataExpression;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GridAxisEntry {
    pub key: String,
    pub label: String,
    pub order: i32,
    pub group: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GridDefinition {
    pub row: Option<DataExpression>,
    pub column: Option<DataExpression>,
    pub frame: Option<DataExpression>,
    pub rows: Vec<GridAxisEntry>,
    pub columns: Vec<GridAxisEntry>,
    pub highlights: Vec<GridHighlight>,
    pub summary_label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GridHighlight {
    pub row: DataExpression,
    pub column: DataExpression,
    pub label: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl GridDefinition {
    pub fn validate(&self) -> Result<(), String> {
        if self.row.is_none() || self.column.is_none() || self.frame.is_none() {
            return Err("GRID_CHART requires row, column and timestamp frame expressions".into());
        }
        if self
            .row
            .iter()
            .chain(self.column.iter())
            .chain(self.frame.iter())
            .chain(self.highlights.iter().flat_map(|highlight| [&highlight.row, &highlight.column]))
            .any(|expression| !expression.transforms.is_empty())
        {
            return Err("GRID_CHART identity and frame bindings must select scalar fields without transforms".into());
        }
        for (label, entries) in [("rows", &self.rows), ("columns", &self.columns)] {
            let mut keys = BTreeSet::new();
            if entries.is_empty() || entries.iter().any(|entry| entry.key.is_empty() || !keys.insert(&entry.key)) {
                return Err(format!("GRID_CHART {label} require nonempty distinct keys"));
            }
        }
        Ok(())
    }
}

impl harmos::MemorySize for GridHighlight {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.row.memory_size().saturating_sub(std::mem::size_of_val(&self.row)))
            .saturating_add(self.column.memory_size().saturating_sub(std::mem::size_of_val(&self.column)))
            .saturating_add(self.label.capacity())
    }
}

impl harmos::MemorySize for GridDefinition {
    fn memory_size(&self) -> usize {
        let fields = [
            (self.row.memory_size(), std::mem::size_of_val(&self.row)),
            (self.column.memory_size(), std::mem::size_of_val(&self.column)),
            (self.frame.memory_size(), std::mem::size_of_val(&self.frame)),
            (self.rows.memory_size(), std::mem::size_of_val(&self.rows)),
            (self.columns.memory_size(), std::mem::size_of_val(&self.columns)),
            (self.highlights.memory_size(), std::mem::size_of_val(&self.highlights)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>().saturating_add(self.summary_label.capacity()), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
