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
use prost::Message;
use prost_reflect::{DynamicMessage, ReflectMessage};

// Local Imports
use crate::model::{
    self, Card, DataExpression, Document, EvidenceCard, Metadata, Section, Summary, SummaryTile, card, data_expression, metadata,
    transform_expression,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Card {
    pub fn plot(plot: model::Plot) -> Self {
        Self {
            metadata: plot.metadata.clone(),
            body: Some(card::Body::Plot(plot)),
            ..Default::default()
        }
    }

    pub fn records(identifier: impl Into<String>) -> Self {
        Self {
            metadata: Some(Metadata::new(identifier, "Records")),
            body: Some(card::Body::Evidence(EvidenceCard::Records as i32)),
            ..Default::default()
        }
    }

    pub fn columns(mut self, columns: u32) -> Self {
        self.layout.get_or_insert_default().columns = columns;
        self
    }
}

impl Document {
    pub fn validate(&self) -> Result<(), String> {
        validate_message(&self.transcode_to_dynamic())?;
        for section in &self.sections {
            validate_repeat(section)?;
        }
        Ok(())
    }

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

fn validate_message(message: &DynamicMessage) -> Result<(), String> {
    if message.descriptor().full_name() == "dokime.model.v1.DataExpression" {
        return DataExpression::decode(message.encode_to_vec().as_slice()).map_err(|error| error.to_string())?.validate();
    }
    for (_, value) in message.fields() {
        match value {
            prost_reflect::Value::Message(child) => validate_message(child)?,
            prost_reflect::Value::List(children) => {
                for child in children {
                    if let prost_reflect::Value::Message(child) = child {
                        validate_message(child)?;
                    }
                }
            }
            _ => (),
        }
    }
    Ok(())
}

fn validate_repeat(section: &Section) -> Result<(), String> {
    if let Some(repeat) = &section.repeat {
        let expressions = repeat
            .value
            .iter()
            .chain(repeat.order_by.iter())
            .chain(section.cards.iter().flat_map(|card| card.filters.iter().filter_map(|filter| filter.expression.as_ref())));
        if expressions.into_iter().any(row_local) {
            let name = section.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str());
            return Err(format!("Validate Section '{name}': Remove Row-Local Transforms from Repeat Values, Ordering and Filters"));
        }
    }
    for child in &section.sections {
        validate_repeat(child)?;
    }
    Ok(())
}

fn row_local(expression: &DataExpression) -> bool {
    match expression.kind.as_ref() {
        Some(data_expression::Kind::Transform(transform)) => {
            matches!(transform.kind, Some(transform_expression::Kind::At(_) | transform_expression::Kind::TimeBetween(_)))
                || transform.input.as_deref().is_some_and(row_local)
        }
        Some(data_expression::Kind::Aggregate(aggregate)) => aggregate.input.as_deref().is_some_and(row_local),
        _ => false,
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DocumentFilter, FieldReference, SectionRepeat};

    #[test]
    fn repeat_refuses_row_local_values_ordering_and_filters() {
        let sampled = FieldReference::new("readings").field("value").at("status");
        let interval = DataExpression {
            kind: Some(data_expression::Kind::Record(FieldReference::new("status"))),
            ..Default::default()
        }
        .time_between("status");
        for expression in [sampled, interval] {
            for position in 0..3 {
                let mut section = Section {
                    metadata: Some(Metadata::new("events", "Events")),
                    repeat: Some(SectionRepeat::default()),
                    ..Default::default()
                };
                match position {
                    0 => section.repeat.as_mut().unwrap().value = Some(expression.clone()),
                    1 => section.repeat.as_mut().unwrap().order_by = Some(expression.clone().mean()),
                    _ => section.cards.push(Card {
                        filters: vec![DocumentFilter {
                            expression: Some(expression.clone()),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                }
                let document = Document {
                    sections: vec![section],
                    ..Default::default()
                };
                assert!(document.validate().unwrap_err().contains("Section 'events'"));
            }
        }
    }
}
