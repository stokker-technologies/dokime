/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Line matchers and markers over captured bytes. Whoever runs a watch splits
// what it captures into lines and offers each line here; a matcher yields one
// row for its stream, a marker one record of its type.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use regex::{Regex, RegexBuilder};

// Local Imports
use dokime_contract::core::Value;
use dokime_contract::operations::files::{LineFieldKind, LineMarker, LineMatcher};

/* === Constants === */
// The most matchers and markers one watch may carry together.
pub const MAX_LINE_MATCHERS: usize = 32;

// The longest pattern one matcher or marker may carry, in UTF-8 bytes.
pub const MAX_PATTERN_BYTES: usize = 1024;

// Compiled program size one pattern may reach.
const PATTERN_SIZE: usize = 1 << 20;

// A line longer than this is offered truncated at the bound.
pub const MAX_LINE_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default)]
pub struct LineMatchers {
    rows: Vec<CompiledRow>,
    markers: Vec<(Regex, String)>,
}

#[derive(Clone, Debug)]
struct CompiledRow {
    storage_schema: harmos::schema::graph::Schema,
    pattern: Regex,
    stream: String,
    fields: Vec<(String, LineFieldKind)>,
}

// Splits bytes arriving in arbitrary pieces into whole lines; a partial line
// waits for its end.
#[derive(Clone, Debug, Default)]
pub struct LineBuffer {
    pending: Vec<u8>,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq)]
pub enum LineMatch {
    Row {
        storage_schema: harmos::schema::graph::Schema,
        stream: String,
        values: BTreeMap<String, Value>,
    },
    Marker {
        record_type: String,
        details: BTreeMap<String, Value>,
        line: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidLineMatcher {
    #[error("a watch carries at most {MAX_LINE_MATCHERS} matchers and markers")]
    Count,
    #[error("pattern {index} must contain 1–{MAX_PATTERN_BYTES} bytes")]
    Shape { index: usize },
    #[error("pattern {index} is invalid: {reason}")]
    Pattern { index: usize, reason: String },
    #[error("matcher {index} schema is incompatible with its finite scalar captures")]
    Schema { index: usize },
    #[error("matcher {index} names no stream or no field")]
    Target { index: usize },
    #[error("matcher {index} field {field:?} has no named capture group or no kind")]
    Field { index: usize, field: String },
    #[error("marker {index} names no record type")]
    RecordType { index: usize },
}

/* === Traits === */

/* === Implementations === */
impl LineMatchers {
    pub fn compile(
        matchers: &[LineMatcher],
        markers: &[LineMarker],
    ) -> Result<Self, InvalidLineMatcher> {
        if matchers.len() + markers.len() > MAX_LINE_MATCHERS {
            return Err(InvalidLineMatcher::Count);
        }
        let mut compiled = Self::default();
        for (index, matcher) in matchers.iter().enumerate() {
            let pattern = pattern(index, &matcher.pattern)?;
            if matcher.stream.is_empty() || matcher.fields.is_empty() {
                return Err(InvalidLineMatcher::Target { index });
            }
            use harmos::schema::{Cardinality, graph::Kind};
            matcher
                .storage_schema
                .preparation(&harmos::config::SchemaSettings::default())
                .map_err(|_| InvalidLineMatcher::Schema { index })?;
            let declaration = &matcher.storage_schema.definitions[matcher.storage_schema.root];
            if declaration.len() != matcher.fields.len()
                || declaration.iter().any(|field| {
                    let expected = match field.kind {
                        Kind::Bool => LineFieldKind::Boolean,
                        Kind::I32 | Kind::I64 | Kind::U32 | Kind::U64 => LineFieldKind::Integer,
                        Kind::F32 | Kind::F64 => LineFieldKind::Number,
                        Kind::String => LineFieldKind::Text,
                        _ => return true,
                    };
                    matches!(
                        field.cardinality,
                        Cardinality::Repeated | Cardinality::Variant
                    ) || !matcher
                        .fields
                        .iter()
                        .any(|capture| capture.name == field.name && capture.kind == expected)
                })
            {
                return Err(InvalidLineMatcher::Schema { index });
            }
            let named = pattern.capture_names().flatten().collect::<Vec<_>>();
            if declaration.iter().any(|field| {
                field.cardinality == Cardinality::Required && !named.contains(&field.name.as_ref())
            }) {
                return Err(InvalidLineMatcher::Schema { index });
            }
            let mut fields = Vec::with_capacity(matcher.fields.len());
            for field in &matcher.fields {
                if field.kind == LineFieldKind::Unspecified {
                    return Err(InvalidLineMatcher::Field {
                        index,
                        field: field.name.clone(),
                    });
                }
                // A column the pattern does not capture is left to its default.
                if named.contains(&field.name.as_str()) {
                    fields.push((field.name.clone(), field.kind));
                }
            }
            if fields.is_empty() {
                return Err(InvalidLineMatcher::Target { index });
            }
            compiled.rows.push(CompiledRow {
                storage_schema: matcher.storage_schema.clone(),
                pattern,
                stream: matcher.stream.clone(),
                fields,
            });
        }
        for (offset, marker) in markers.iter().enumerate() {
            let index = matchers.len() + offset;
            let pattern = pattern(index, &marker.pattern)?;
            if marker.record_type.is_empty() {
                return Err(InvalidLineMatcher::RecordType { index });
            }
            compiled.markers.push((pattern, marker.record_type.clone()));
        }
        Ok(compiled)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.markers.is_empty()
    }

    // Every row and record one line yields, matchers first, each in order. A
    // row whose capture does not read as its column's kind is skipped.
    pub fn matches(&self, line: &str) -> Vec<LineMatch> {
        let mut found = Vec::new();
        for row in &self.rows {
            let Some(captures) = row.pattern.captures(line) else {
                continue;
            };
            let mut values = BTreeMap::new();
            let mut readable = true;
            for (name, kind) in &row.fields {
                let Some(text) = captures.name(name).map(|capture| capture.as_str()) else {
                    continue;
                };
                match read(text, *kind) {
                    Some(value) => {
                        values.insert(name.clone(), value);
                    }
                    None => readable = false,
                }
            }
            if readable && !values.is_empty() {
                found.push(LineMatch::Row {
                    storage_schema: row.storage_schema.clone(),
                    stream: row.stream.clone(),
                    values,
                });
            }
        }
        for (pattern, record_type) in &self.markers {
            let Some(captures) = pattern.captures(line) else {
                continue;
            };
            let details = pattern
                .capture_names()
                .flatten()
                .filter_map(|name| {
                    captures
                        .name(name)
                        .map(|capture| (name.to_owned(), Value::Text(capture.as_str().to_owned())))
                })
                .collect();
            found.push(LineMatch::Marker {
                record_type: record_type.clone(),
                details,
                line: line.to_owned(),
            });
        }
        found
    }
}

impl LineBuffer {
    // The whole lines these bytes complete, without their line endings.
    pub fn push(&mut self, bytes: &[u8]) -> Vec<String> {
        let mut lines = Vec::new();
        for byte in bytes {
            if *byte == b'\n' {
                lines.push(self.take());
            } else if self.pending.len() < MAX_LINE_BYTES {
                self.pending.push(*byte);
            }
        }
        lines
    }

    // The last line when the bytes end without a line ending.
    pub fn finish(&mut self) -> Option<String> {
        (!self.pending.is_empty()).then(|| self.take())
    }

    // A restarted source drops the partial line it had begun.
    pub fn reset(&mut self) {
        self.pending.clear();
    }

    fn take(&mut self) -> String {
        let mut line = std::mem::take(&mut self.pending);
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        String::from_utf8_lossy(&line).into_owned()
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn pattern(index: usize, source: &str) -> Result<Regex, InvalidLineMatcher> {
    if source.is_empty() || source.len() > MAX_PATTERN_BYTES {
        return Err(InvalidLineMatcher::Shape { index });
    }
    RegexBuilder::new(source)
        .size_limit(PATTERN_SIZE)
        .build()
        .map_err(|source| InvalidLineMatcher::Pattern {
            index,
            reason: source.to_string(),
        })
}

fn read(text: &str, kind: LineFieldKind) -> Option<Value> {
    let text = text.trim();
    match kind {
        LineFieldKind::Text => Some(Value::Text(text.to_owned())),
        LineFieldKind::Integer => text
            .parse::<i64>()
            .ok()
            .map(Value::Integer)
            .or_else(|| text.parse::<u64>().ok().map(Value::Unsigned)),
        LineFieldKind::Number => text
            .parse::<f64>()
            .ok()
            .filter(|number| number.is_finite())
            .map(Value::Number),
        LineFieldKind::Boolean => match text.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(Value::Bool(true)),
            "false" | "0" | "no" | "off" => Some(Value::Bool(false)),
            _ => None,
        },
        LineFieldKind::Unspecified => None,
    }
}

/* === Tests === */
#[cfg(test)]
mod tests {
    use super::*;
    use dokime_contract::operations::files::LineField;

    fn row_schema() -> harmos::schema::graph::Schema {
        #[harmos::message]
        struct Log {
            temperature_c: f64,
            state: String,
        }
        harmos::schema::graph::Schema::from_declaration(
            <Log as harmos::schema::Described>::DEFINITION,
            &harmos::config::SchemaSettings::default(),
        )
        .unwrap()
    }

    #[test]
    fn lines_become_rows_and_markers() {
        let matchers = LineMatchers::compile(
            &[LineMatcher {
                storage_schema: row_schema(),
                pattern: r"temperature=(?P<temperature_c>[-\d.]+) state=(?P<state>\w+)".into(),
                stream: "bench.charger.log".into(),
                fields: vec![
                    LineField {
                        name: "temperature_c".into(),
                        kind: LineFieldKind::Number,
                    },
                    LineField {
                        name: "state".into(),
                        kind: LineFieldKind::Text,
                    },
                ],
            }],
            &[LineMarker::new(
                r"FAULT (?P<code>\w+)",
                "bench.charger.fault",
            )],
        )
        .unwrap();
        let mut buffer = LineBuffer::default();
        let mut lines = buffer.push(b"temperature=41.5 state=charging\r\nFAULT E7\npartial");
        lines.extend(buffer.finish());
        assert_eq!(lines.len(), 3);
        assert_eq!(
            matchers.matches(&lines[0]),
            vec![LineMatch::Row {
                storage_schema: row_schema(),
                stream: "bench.charger.log".into(),
                values: BTreeMap::from([
                    ("state".to_owned(), Value::Text("charging".into())),
                    ("temperature_c".to_owned(), Value::Number(41.5)),
                ]),
            }]
        );
        assert_eq!(
            matchers.matches(&lines[1]),
            vec![LineMatch::Marker {
                record_type: "bench.charger.fault".into(),
                details: BTreeMap::from([("code".to_owned(), Value::Text("E7".into()))]),
                line: "FAULT E7".into(),
            }]
        );
        assert!(matchers.matches("temperature=hot state=x").is_empty());
        assert!(matchers.matches(&lines[2]).is_empty());
    }

    #[test]
    fn malformed_matchers_are_refused() {
        let refused = LineMatchers::compile(&[], &[LineMarker::new("(", "x")]).unwrap_err();
        assert!(matches!(
            refused,
            InvalidLineMatcher::Pattern { index: 0, .. }
        ));
        let unnamed = LineMatchers::compile(
            &[LineMatcher {
                storage_schema: row_schema(),
                pattern: r"(\d+)".into(),
                stream: "s".into(),
                fields: vec![LineField {
                    name: "value".into(),
                    kind: LineFieldKind::Integer,
                }],
            }],
            &[],
        )
        .unwrap_err();
        assert_eq!(unnamed, InvalidLineMatcher::Schema { index: 0 });
    }
}
