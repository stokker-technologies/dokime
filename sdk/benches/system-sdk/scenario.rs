/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Construct and Shape the Shared SDK Scenario

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports
use prost::Message;
use prost_reflect::{DescriptorPool, Kind, MessageDescriptor};
use prost_types::Timestamp;

// Local Imports
use dokime::{Declarations, Suites};

/* === Constants === */

pub const INTERVALS_NS: [u64; 5] = [1_000_000, 100_000, 10_000, 1_000, 100];
pub const CALLS: usize = 256;
pub const CONCURRENCY: usize = 8;
pub const PAYLOAD: &[u8] = b"SDK Benchmark Echo";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, PartialEq)]
pub struct Scenario {
    pub channels: Vec<String>,
    pub ceiling: String,
    pub intervals_ns: Vec<u64>,
}

#[derive(Debug, PartialEq)]
pub struct Measurement {
    pub case: String,
    pub label: &'static str,
    pub unit: &'static str,
    pub value: Option<f64>,
    pub reason: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Scenario {
    pub fn new() -> Self {
        Self {
            channels: (0..128).map(|index| format!("thermal.load.channel_{index:03}")).collect(),
            ceiling: "thermal.load.burst".into(),
            intervals_ns: INTERVALS_NS.to_vec(),
        }
    }
}

impl Measurement {
    pub fn new(language: &str, case: &str, label: &'static str, unit: &'static str, value: Option<f64>, reason: Option<String>) -> Self {
        Self {
            case: format!("sdk/{language}/{case}"),
            label,
            unit,
            value,
            reason,
        }
    }
}

/* === Public Functions === */

pub fn accepted(received: Timestamp, start: Timestamp, end: Timestamp) -> bool {
    let received = (received.seconds, received.nanos);
    (start.seconds, start.nanos) <= received && received < (end.seconds, end.nanos)
}

pub fn percentile(values: &[f64], numerator: usize) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    Some(ordered[(ordered.len() * numerator).div_ceil(100).saturating_sub(1)])
}

pub fn catalog(declarations: &Declarations) -> Result<BTreeMap<String, String>, String> {
    let types = declarations.types.clone().ok_or("Inspect Catalog: Include Row Descriptors")?;
    let pool = DescriptorPool::from_file_descriptor_set(types).map_err(|error| error.to_string())?;
    let mut result = BTreeMap::new();
    for operation in &declarations.operations {
        result.insert(
            format!("operation/{}", operation.id),
            format!(
                "{}:{}:{}:{:?}",
                schema(
                    pool.get_message_by_name(&operation.request)
                        .ok_or_else(|| format!("Inspect {}: Include Its Request Schema", operation.id))?,
                    &mut Vec::new()
                ),
                schema(
                    pool.get_message_by_name(&operation.response)
                        .ok_or_else(|| format!("Inspect {}: Include Its Response Schema", operation.id))?,
                    &mut Vec::new()
                ),
                operation.visibility,
                operation.policy
            ),
        );
    }
    for stream in &declarations.streams {
        let row = pool.get_message_by_name(&stream.row).ok_or_else(|| format!("Inspect {}: Include Its Row Schema", stream.id))?;
        result.insert(
            format!("stream/{}", stream.id),
            format!("{:?}:{}:{}:{}", stream.interval, stream.interval_setting, stream.window_setting, schema(row, &mut Vec::new())),
        );
    }
    for section in &declarations.sections {
        if section.r#type.ends_with(".Suites") {
            let suites = Suites::decode(section.payload.as_slice()).map_err(|error| error.to_string())?;
            for suite in suites.suites {
                for template in suite.templates {
                    let identifier = template.metadata.as_ref().ok_or("Inspect Template: Include Its Identifier")?.identifier.clone();
                    result.insert(
                        format!("template/{identifier}"),
                        format!("{:?}:{:?}", template.parameters, template.steps.iter().map(|step| &step.operation).collect::<Vec<_>>()),
                    );
                }
            }
        }
    }
    Ok(result)
}

pub fn guard(reference: &BTreeMap<String, String>, candidate: &BTreeMap<String, String>, language: &str) -> Result<(), String> {
    for identifier in reference.keys().chain(candidate.keys()).collect::<BTreeSet<_>>() {
        if reference.get(identifier) != candidate.get(identifier) {
            return Err(format!("Measure SDK: Harmonize {identifier} Between rust and {language}"));
        }
    }
    Ok(())
}

/* === Private Functions === */

fn schema(message: MessageDescriptor, stack: &mut Vec<String>) -> String {
    if let Some(index) = stack.iter().position(|name| name == message.full_name()) {
        return format!("recursive/{index}");
    }
    stack.push(message.full_name().into());
    let fields = message
        .fields()
        .map(|field| {
            let kind = match field.kind() {
                Kind::Message(nested) => schema(nested, stack),
                Kind::Enum(definition) => {
                    format!("{:?}", definition.values().map(|value| (value.name().to_owned(), value.number())).collect::<Vec<_>>())
                }
                kind => format!("{kind:?}"),
            };
            format!("{}:{}:{:?}:{kind}", field.number(), field.name(), field.cardinality())
        })
        .collect::<Vec<_>>()
        .join(",");
    stack.pop();
    fields
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Measurement, Scenario, accepted, guard, percentile};
    use dokime::encoding::contract::Message;
    use dokime::{Declarations, StreamDeclaration};
    use prost_types::Timestamp;

    #[derive(dokime::Row)]
    struct Counter {
        value: u64,
    }

    #[test]
    fn accepted_count_uses_host_receipt_time_with_an_exclusive_end() {
        let start = Timestamp { seconds: 5, nanos: 0 };
        let end = Timestamp { seconds: 6, nanos: 0 };
        assert!(!accepted(Timestamp { seconds: 4, nanos: 999_999_999 }, start, end));
        assert!(accepted(start, start, end));
        assert!(accepted(Timestamp { seconds: 5, nanos: 999_999_999 }, start, end));
        assert!(!accepted(end, start, end));
    }

    #[test]
    fn scenario_separates_channels_and_ceiling_and_increases_row_rate() {
        let scenario = Scenario::new();
        assert_eq!(scenario.channels.first().unwrap(), "thermal.load.channel_000");
        assert_eq!(scenario.channels.last().unwrap(), "thermal.load.channel_127");
        assert!(!scenario.channels.contains(&scenario.ceiling));
        assert!(scenario.intervals_ns.windows(2).all(|pair| pair[0] > pair[1]));
    }

    #[test]
    fn extra_stream_refuses_before_measurement() {
        let declarations = Declarations {
            types: Some(Counter::types()),
            streams: vec![StreamDeclaration {
                id: "thermal.load.channel_000".into(),
                row: Counter::name(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let reference = super::catalog(&declarations).unwrap();
        let mut candidate = declarations;
        candidate.streams.push(StreamDeclaration {
            id: "thermal.load.extra".into(),
            row: Counter::name(),
            ..Default::default()
        });
        let failure = guard(&reference, &super::catalog(&candidate).unwrap(), "python").unwrap_err();
        assert!(failure.contains("thermal.load.extra") && failure.contains("rust") && failure.contains("python"));
    }

    #[test]
    fn result_shape_preserves_unavailable_measure_and_percentile_rank() {
        let measurement = Measurement::new(
            "rust",
            "memory/idle",
            "Memory [bytes]",
            "bytes",
            None,
            Some("Read Plugin Memory: Grant Process Inspection".into()),
        );
        assert_eq!(measurement.case, "sdk/rust/memory/idle");
        assert!(measurement.value.is_none() && measurement.reason.is_some());
        assert_eq!(percentile(&[4.0, 1.0, 3.0, 2.0], 50), Some(2.0));
        assert_eq!(percentile(&[], 95), None);
    }
}
