/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Document Expansion

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result};
use dokime_protocol::v1;
use dokime_storage::{ExecutionCut, Store};
use prost::Message;
use sha2::{Digest, Sha256};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Expansion {
    pub document: v1::Document,
    pub frames: Vec<v1::document_frame_frame::Kind>,
    pub next: Option<usize>,
}
struct Reader<'a> {
    store: &'a Store,
    cut: &'a ExecutionCut,
    page_size: usize,
    skip: usize,
    remaining: usize,
    visited: usize,
    more: bool,
    frames: Vec<v1::document_frame_frame::Kind>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Reader<'_> {
    fn section(
        &mut self, section: &v1::Section, filters: &[v1::PlotFilter], path: &str, suffix: &str, depth: usize,
    ) -> Result<Vec<v1::Section>> {
        if depth > 32 {
            return Err(Error::Invalid("Expand Document: Reduce Section Nesting to 32 Levels".into()));
        }
        if self.more {
            return Ok(Vec::new());
        }
        if let Some(repeat) = &section.repeat {
            let expression =
                repeat.value.as_ref().ok_or_else(|| Error::Invalid("Expand Document: Supply the Repeat Value Expression".into()))?;
            let selection = format!("{path}{suffix}");
            let mut offset = 0;
            let mut output = Vec::new();
            let mut selection_frame = None;
            loop {
                let (values, total) =
                    self.store.document_distinct(self.cut, expression, repeat.order_by.as_ref(), filters, offset, self.page_size)?;
                if offset == 0 {
                    selection_frame = Some(v1::document_frame_frame::Kind::RepeatSelection(v1::DocumentRepeatSelection {
                        node_id: selection.clone(),
                        instance_count: total,
                        pending: false,
                    }));
                }
                if values.is_empty() {
                    break;
                }
                for value in values {
                    let key = Sha256::digest(value.encode_to_vec()).iter().take(16).map(|byte| format!("{byte:02x}")).collect::<String>();
                    let next_suffix = format!("{suffix}.{key}");
                    let filter = v1::PlotFilter {
                        expression: Some(expression.clone()),
                        operator: v1::FilterOperator::Eq as i32,
                        value: Some(value.clone()),
                    };
                    let mut inherited = filters.to_vec();
                    inherited.push(filter.clone());
                    let mut instance = section.clone();
                    instance.repeat = None;
                    let mut metadata = instance.metadata.clone().unwrap_or_default();
                    metadata.identifier = path.into();
                    let label = display(&value);
                    metadata.label = repeat.label_template.replace("{value}", &label);
                    let field = match expression.kind.as_ref() {
                        Some(v1::data_expression::Kind::Stream(reference) | v1::data_expression::Kind::Record(reference)) => {
                            Some(reference.field.as_str())
                        }
                        Some(v1::data_expression::Kind::Parameter(key) | v1::data_expression::Kind::Dimension(key)) => Some(key.as_str()),
                        _ => None,
                    };
                    if let Some(field) = field {
                        metadata.label = metadata.label.replace(&format!("{{{field}}}"), &label);
                    }
                    instance.metadata = Some(metadata);
                    let sections = self.section(&instance, &inherited, path, &next_suffix, depth + 1)?;
                    if !sections.is_empty() {
                        self.frames.push(v1::document_frame_frame::Kind::RepeatInstance(v1::DocumentRepeatInstance {
                            selection_id: selection.clone(),
                            ordinal: offset as u64,
                            suffix: next_suffix.trim_start_matches('.').into(),
                            node_id: format!("{path}{next_suffix}"),
                            filter: Some(filter),
                        }));
                    }
                    output.extend(sections);
                    offset += 1;
                    if self.more {
                        if !output.is_empty()
                            && let Some(frame) = selection_frame.take()
                        {
                            self.frames.push(frame);
                        }
                        return Ok(output);
                    }
                }
            }
            if !output.is_empty()
                && let Some(frame) = selection_frame
            {
                self.frames.push(frame);
            }
            return Ok(output);
        }
        let include = self.visited >= self.skip;
        if include && self.remaining == 0 {
            self.more = true;
            return Ok(Vec::new());
        }
        self.visited += 1;
        let mut result = section.clone();
        result.sections.clear();
        result.repeat = None;
        let metadata = result.metadata.get_or_insert_default();
        metadata.identifier = format!("{path}{suffix}");
        if include {
            self.remaining -= 1;
            for (index, card) in result.cards.iter_mut().enumerate() {
                let metadata = card.metadata.get_or_insert_default();
                if metadata.identifier.is_empty() {
                    metadata.identifier = format!("{path}.card-{index}");
                }
                metadata.identifier.push_str(suffix);
                card.filters.extend(filters.iter().map(|filter| v1::DocumentFilter {
                    expression: filter.expression.clone(),
                    operator: filter.operator,
                    value: filter.value.clone(),
                }));
            }
        } else {
            result.cards.clear();
        }
        for (index, child) in section.sections.iter().enumerate() {
            let path = child
                .metadata
                .as_ref()
                .filter(|metadata| !metadata.identifier.is_empty())
                .map_or_else(|| format!("{path}.section-{index}"), |metadata| metadata.identifier.clone());
            result.sections.extend(self.section(child, filters, &path, suffix, depth + 1)?);
            if self.more {
                break;
            }
        }
        if include || !result.sections.is_empty() { Ok(vec![result]) } else { Ok(Vec::new()) }
    }
}

/* === Public Functions === */

pub fn expand(store: &Store, cut: &ExecutionCut, document: &v1::Document, offset: usize, page_size: usize) -> Result<Expansion> {
    let mut reader = Reader {
        store,
        cut,
        page_size: page_size.max(1),
        skip: offset,
        remaining: page_size.max(1),
        visited: 0,
        more: false,
        frames: Vec::new(),
    };
    let mut result = document.clone();
    result.sections.clear();
    for (index, section) in document.sections.iter().enumerate() {
        let path = section
            .metadata
            .as_ref()
            .filter(|metadata| !metadata.identifier.is_empty())
            .map_or_else(|| format!("section-{index}"), |metadata| metadata.identifier.clone());
        result.sections.extend(reader.section(section, &[], &path, "", 0)?);
        if reader.more {
            break;
        }
    }
    Ok(Expansion {
        document: result,
        frames: reader.frames,
        next: reader.more.then_some(reader.visited),
    })
}

/* === Private Functions === */

fn display(value: &v1::Value) -> String {
    match &value.kind {
        Some(v1::value::Kind::Number(value)) => value.to_string(),
        Some(v1::value::Kind::Integer(value)) => value.to_string(),
        Some(v1::value::Kind::UnsignedInteger(value)) => value.to_string(),
        Some(v1::value::Kind::Text(value)) => value.clone(),
        Some(v1::value::Kind::Boolean(value)) => value.to_string(),
        _ => String::new(),
    }
}

/* === Tests === */
