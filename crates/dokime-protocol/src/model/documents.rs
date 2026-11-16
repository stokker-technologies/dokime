/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Documents

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::model::{self, Card, DataExpression, Document, Section, Summary, SummaryTile, card, data_expression, metadata};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Document {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            metadata: Some(metadata::from_label(label.into())),
            ..Default::default()
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.metadata.get_or_insert_default().identifier = id.into();
        self
    }

    pub fn section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }
}

impl Section {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            metadata: Some(metadata::from_label(label.into())),
            ..Default::default()
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.metadata.get_or_insert_default().identifier = id.into();
        self
    }

    pub fn section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }

    pub fn card(mut self, card: Card) -> Self {
        self.cards.push(card);
        self
    }

    pub fn plot(self, plot: impl Into<model::Plot>) -> Self {
        let mut plot = plot.into();
        if plot.metadata.as_ref().is_none_or(|metadata| metadata.label.is_empty()) {
            let expression = plot.z.as_ref().or_else(|| plot.y.first()).or(plot.x.as_ref());
            let label = expression.map(expression_label).unwrap_or_else(|| "Plot".into());
            let identifier = plot.metadata.as_ref().map(|metadata| metadata.identifier.clone()).filter(|identifier| !identifier.is_empty());
            let mut metadata = metadata::from_label(label);
            if let Some(identifier) = identifier {
                metadata.identifier = identifier;
            }
            plot.metadata = Some(metadata);
        }
        let card = Card {
            metadata: plot.metadata.clone(),
            body: Some(card::Body::Plot(plot)),
            ..Default::default()
        };
        self.card(card)
    }

    pub fn summary(self, label: impl Into<String>, summary: Summary) -> Self {
        let card = Card {
            metadata: Some(metadata::from_label(label.into())),
            body: Some(card::Body::Summary(summary)),
            ..Default::default()
        };
        self.card(card)
    }
}

impl Summary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tile(mut self, label: impl Into<String>, value: DataExpression) -> Self {
        let tile = SummaryTile {
            metadata: Some(metadata::from_label(label.into())),
            value: Some(value),
            unit: String::new(),
        };
        self.tiles.push(tile);
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn expression_label(value: &DataExpression) -> String {
    if !value.label.is_empty() {
        return value.label.clone();
    }
    match value.kind.as_ref() {
        Some(data_expression::Kind::Stream(field) | data_expression::Kind::Record(field)) => field.field.clone(),
        Some(data_expression::Kind::Transform(transform)) => {
            transform.input.as_deref().map(expression_label).unwrap_or_else(|| "Value".into())
        }
        Some(data_expression::Kind::Aggregate(aggregate)) => {
            aggregate.input.as_deref().map(expression_label).unwrap_or_else(|| "Value".into())
        }
        _ => "Value".into(),
    }
}

/* === Tests === */
