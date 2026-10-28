/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::document::{
    DocumentDefinition, DocumentFilter, DocumentSelection, DocumentWarning, PlotDescriptor, TelemetrySummaryGroupPayload,
};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum CardBindingResult {
    Static,
    Summary { groups: Vec<TelemetrySummaryGroupPayload> },
    Plot { descriptor: Box<PlotDescriptor> },
}

impl SectionBinding {
    pub fn valid_result(&self) -> bool {
        self.unavailable.as_deref().is_none_or(super::payload::valid_unavailable_reason)
    }
}
