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
use harmos_domain::Identifier;
use harmos_domain::Metadata;
use serde::Deserialize;
use serde::Serialize;
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasurementUnit {
    pub metadata: Metadata,
    pub family: UnitFamily,
    pub symbol: String,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitFamily {
    Count,
    Distance,
    Duration,
    Frequency,
    Power,
    Temperature,
    Voltage,
    Ratio,
    Custom,
}

/* === Traits === */

/* === Implementations === */
impl harmos_domain::MemorySize for MeasurementUnit {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.symbol), std::mem::size_of_val(&self.symbol)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl MeasurementUnit {
    pub fn custom(identifier: Identifier, label: impl Into<String>, symbol: impl Into<String>) -> Self {
        Self { metadata: Metadata::new(identifier, label, None), family: UnitFamily::Custom, symbol: symbol.into() }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
