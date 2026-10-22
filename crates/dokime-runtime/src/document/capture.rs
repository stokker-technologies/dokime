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
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::document::{
    DocumentDefinition, DocumentFilter, DocumentSelection, DocumentWarning, PlotDescriptor, TelemetrySummaryGroupPayload,
};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Capture {
    pub definition: DocumentDefinition,
    pub selection: DocumentSelection,
    pub cards: Vec<CardBinding>,
    pub repeats: BTreeMap<Identifier, Vec<RepeatInstance>>,
    pub sections: BTreeMap<Identifier, SectionBinding>,
    pub warnings: Vec<DocumentWarning>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CardBinding {
    pub source_id: Identifier,
    pub metadata: Metadata,
    pub result: CardBindingResult,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SectionBinding {
    pub metadata: Metadata,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RepeatInstance {
    pub suffix: String,
    pub node_id: Identifier,
    pub filter: DocumentFilter,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum CardBindingResult {
    Static,
    Summary { groups: Vec<TelemetrySummaryGroupPayload> },
    Plot { descriptor: Box<PlotDescriptor> },
}

/* === Traits === */

/* === Implementations === */

impl harmos::MemorySize for Capture {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.definition), std::mem::size_of_val(&self.definition)),
            (harmos::MemorySize::memory_size(&self.selection), std::mem::size_of_val(&self.selection)),
            (harmos::MemorySize::memory_size(&self.cards), std::mem::size_of_val(&self.cards)),
            (harmos::MemorySize::memory_size(&self.repeats), std::mem::size_of_val(&self.repeats)),
            (harmos::MemorySize::memory_size(&self.sections), std::mem::size_of_val(&self.sections)),
            (harmos::MemorySize::memory_size(&self.warnings), std::mem::size_of_val(&self.warnings)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for CardBinding {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.source_id), std::mem::size_of_val(&self.source_id)),
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos::MemorySize::memory_size(&self.result), std::mem::size_of_val(&self.result)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl SectionBinding {
    pub fn valid_result(&self) -> bool {
        self.unavailable.as_deref().is_none_or(super::payload::valid_unavailable_reason)
    }
}

impl harmos::MemorySize for SectionBinding {
    fn memory_size(&self) -> usize {
        let metadata = harmos::MemorySize::memory_size(&self.metadata);
        let unavailable = harmos::MemorySize::memory_size(&self.unavailable);
        if metadata == usize::MAX || unavailable == usize::MAX {
            return usize::MAX;
        }
        std::mem::size_of::<Self>()
            .saturating_add(metadata.saturating_sub(std::mem::size_of_val(&self.metadata)))
            .saturating_add(unavailable.saturating_sub(std::mem::size_of_val(&self.unavailable)))
    }
}

impl harmos::MemorySize for RepeatInstance {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.suffix), std::mem::size_of_val(&self.suffix)),
            (harmos::MemorySize::memory_size(&self.node_id), std::mem::size_of_val(&self.node_id)),
            (harmos::MemorySize::memory_size(&self.filter), std::mem::size_of_val(&self.filter)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for CardBindingResult {
    fn memory_size(&self) -> usize {
        let (owned, inline) = match self {
            Self::Static => return std::mem::size_of::<Self>(),
            Self::Summary { groups } => (harmos::MemorySize::memory_size(groups), std::mem::size_of_val(groups)),
            Self::Plot { descriptor } => (harmos::MemorySize::memory_size(descriptor.as_ref()), 0),
        };
        if owned == usize::MAX { usize::MAX } else { std::mem::size_of::<Self>().saturating_add(owned.saturating_sub(inline)) }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
