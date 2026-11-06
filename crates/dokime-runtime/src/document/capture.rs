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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CardFailure {
    pub code: String,
    pub message: String,
    pub budget: Option<CardBudget>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CardBudget {
    pub name: String,
    pub limit: u64,
    pub reached: u64,
    pub card_id: String,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum CardBindingResult {
    Static,
    Summary { groups: Vec<TelemetrySummaryGroupPayload> },
    Plot { descriptor: Box<PlotDescriptor> },
    Pending,
    Failed { failure: CardFailure },
}

/* === Traits === */

/* === Implementations === */

impl CardFailure {
    pub(crate) fn document(error: crate::Error) -> Self {
        let message = error.to_string();
        let budget = match error {
            crate::Error::Document(super::DocumentFailure::Budget { budget, limit, reached, card }) => Some(CardBudget {
                name: budget.into(),
                limit: limit as u64,
                reached: reached as u64,
                card_id: card,
            }),
            _ => None,
        };
        let code = budget.as_ref().map_or("document_card_failed", |budget| budget.name.as_str()).to_owned();
        Self { code, message, budget }
    }
}

impl SectionBinding {
    pub fn valid_result(&self) -> bool {
        self.unavailable.as_deref().is_none_or(super::payload::valid_unavailable_reason)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
