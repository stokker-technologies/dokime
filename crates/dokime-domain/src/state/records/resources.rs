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
use crate::model::records::evidence::Record;
use crate::model::records::resources::{ResourceRecord, ResourceLifecycle};
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::execution::RecordScope;
use crate::model::time::Timestamp;
use crate::model::values::Value;
use crate::model::execution::scope_visible_in;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResourceRegistry {
    resources: BTreeMap<String, ResourceRecord>,
    active_instances: BTreeMap<String, String>,
    latest_instances: BTreeMap<String, String>,
    instance_counts: BTreeMap<String, usize>,
    instance_sequences: BTreeMap<String, usize>,
}

struct ResourceInstance {
    instance_id: String,
    label: String,
    sequence: usize,
}

/* === Enums === */
/* === Traits === */

/* === Implementations === */
impl ResourceRegistry {
    pub fn clear(&mut self) {
        self.resources.clear();
        self.active_instances.clear();
        self.latest_instances.clear();
        self.instance_counts.clear();
        self.instance_sequences.clear();
    }

    pub fn records(&self) -> impl Iterator<Item = &ResourceRecord> {
        self.resources.values()
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&ResourceRecord) -> bool) {
        self.resources.retain(|_, record| keep(record));
        self.rebuild_activity();
    }

    pub fn record_authored_record(&mut self, record: &Record) -> Option<RecordScope> {
        self.record_fields(
            &record.record_type,
            &record.source,
            &record.timestamp,
            record.received_at.to_datetime().ok(),
            &record.details,
            &record.scope,
        )
    }

    fn record_fields(
        &mut self,
        record_type: &Identifier,
        source: &str,
        timestamp: &Timestamp,
        created_at: Option<time::OffsetDateTime>,
        details: &BTreeMap<String, Value>,
        record_scope: &Option<RecordScope>,
    ) -> Option<RecordScope> {
        let lifecycle = resource_lifecycle(record_type)?;
        let scope = record_scope.clone();
        let resource_id = resource_identifier(details, record_scope);
        let resource_kind = text_detail(details, "resource_kind")
            .or_else(|| text_detail(details, "kind"))
            .unwrap_or_else(|| resource_id.clone());
        let resource_name = text_detail(details, "resource_name")
            .or_else(|| text_detail(details, "name"))
            .unwrap_or_else(|| resource_id.clone());
        let instance =
            self.resolve_instance(details, &scope, &resource_id, &resource_name, &lifecycle);
        let key = registry_key(scope.as_ref(), &instance.instance_id);

        let mut record = self
            .resources
            .remove(&key)
            .unwrap_or_else(|| ResourceRecord {
                metadata: {
                    let mut metadata = Metadata::definition(
                        identifier(&format!("resource.{}", instance.instance_id)),
                        instance.label.clone(),
                        Some("Runtime resource observed during test execution.".to_owned()),
                        ["resource", resource_kind.as_str()],
                    );
                    metadata.created_at = created_at;
                    metadata
                },
                resource_id: resource_id.clone(),
                resource_kind: resource_kind.clone(),
                source: source.to_owned(),
                lifecycle: lifecycle.clone(),
                scope: scope.clone(),
                details: BTreeMap::new(),
                updated_at: timestamp.clone(),
            });

        record.resource_id = resource_id.clone();
        record.resource_kind = resource_kind;
        record.source = source.to_owned();
        record.lifecycle = lifecycle.clone();
        record.scope = scope.clone().or(record.scope);
        record.updated_at = timestamp.clone();
        merge_details(&mut record.details, details);
        record
            .details
            .entry("resource_name".to_owned())
            .or_insert_with(|| Value::from(resource_name));
        record
            .details
            .entry("resource_kind".to_owned())
            .or_insert_with(|| Value::from(record.resource_kind.clone()));
        record
            .details
            .entry("resource_id".to_owned())
            .or_insert_with(|| Value::from(record.resource_id.clone()));
        record.details.insert(
            "resource_instance_id".to_owned(),
            Value::from(instance.instance_id.clone()),
        );
        if text_detail(details, "resource_instance_label").is_some() {
            record.details.insert(
                "resource_instance_label".to_owned(),
                Value::from(instance.label.clone()),
            );
        } else {
            record
                .details
                .entry("resource_instance_label".to_owned())
                .or_insert_with(|| Value::from(instance.label.clone()));
        }
        record
            .details
            .entry("resource_sequence".to_owned())
            .or_insert_with(|| Value::from(instance.sequence as i64));

        self.resources.insert(key, record);
        self.update_instance_activity(&scope, &resource_id, &instance.instance_id, &lifecycle);
        scope
    }

    pub fn records_for_scope(&self, selected: &RecordScope) -> Vec<&ResourceRecord> {
        let mut records = self
            .resources
            .values()
            .filter(|record| {
                record
                    .scope
                    .as_ref()
                    .is_some_and(|scope| scope_visible_in(Some(scope), selected))
            })
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            left.metadata
                .label
                .cmp(&right.metadata.label)
                .then_with(|| left.resource_id.cmp(&right.resource_id))
        });
        records
    }

    fn resolve_instance(
        &mut self,
        details: &BTreeMap<String, Value>,
        scope: &Option<RecordScope>,
        resource_id: &str,
        resource_name: &str,
        lifecycle: &ResourceLifecycle,
    ) -> ResourceInstance {
        let base_key = registry_key(scope.as_ref(), resource_id);
        if let Some(instance_id) = text_detail(details, "resource_instance_id") {
            let sequence = self.sequence_for_explicit_instance(&base_key, &instance_id);
            return ResourceInstance {
                label: text_detail(details, "resource_instance_label")
                    .unwrap_or_else(|| instance_label(resource_name, sequence)),
                instance_id,
                sequence,
            };
        }

        if let Some(instance_id) = self.instance_for_existing_record(&base_key, lifecycle) {
            let sequence = instance_sequence(&instance_id).unwrap_or(1);
            return ResourceInstance {
                label: text_detail(details, "resource_instance_label")
                    .unwrap_or_else(|| instance_label(resource_name, sequence)),
                instance_id,
                sequence,
            };
        }

        let sequence = next_instance_sequence(&mut self.instance_counts, &base_key);
        ResourceInstance {
            instance_id: generated_instance_id(resource_id, sequence),
            label: text_detail(details, "resource_instance_label")
                .unwrap_or_else(|| instance_label(resource_name, sequence)),
            sequence,
        }
    }

    fn sequence_for_explicit_instance(&mut self, base_key: &str, instance_id: &str) -> usize {
        let sequence_key = format!("{base_key}|instance:{instance_id}");
        if let Some(sequence) = self.instance_sequences.get(&sequence_key) {
            return *sequence;
        }

        if let Some(sequence) = instance_sequence(instance_id) {
            let count = self.instance_counts.entry(base_key.to_owned()).or_default();
            *count = (*count).max(sequence);
            self.instance_sequences.insert(sequence_key, sequence);
            return sequence;
        }
        let sequence = next_instance_sequence(&mut self.instance_counts, base_key);
        self.instance_sequences.insert(sequence_key, sequence);
        sequence
    }

    fn instance_for_existing_record(
        &self,
        base_key: &str,
        lifecycle: &ResourceLifecycle,
    ) -> Option<String> {
        match lifecycle {
            ResourceLifecycle::Closed | ResourceLifecycle::Failed | ResourceLifecycle::Removed => {
                self.active_instances
                    .get(base_key)
                    .or_else(|| self.latest_instances.get(base_key))
                    .cloned()
            }
            ResourceLifecycle::Registered | ResourceLifecycle::Opened => {
                self.active_instances.get(base_key).cloned()
            }
        }
    }

    fn update_instance_activity(
        &mut self,
        scope: &Option<RecordScope>,
        resource_id: &str,
        instance_id: &str,
        lifecycle: &ResourceLifecycle,
    ) {
        let base_key = registry_key(scope.as_ref(), resource_id);
        self.latest_instances
            .insert(base_key.clone(), instance_id.to_owned());

        match lifecycle {
            ResourceLifecycle::Registered | ResourceLifecycle::Opened => {
                self.active_instances
                    .insert(base_key, instance_id.to_owned());
            }
            ResourceLifecycle::Closed | ResourceLifecycle::Failed | ResourceLifecycle::Removed => {
                if self.active_instances.get(&base_key).map(String::as_str) == Some(instance_id) {
                    self.active_instances.remove(&base_key);
                }
            }
        }
    }

    fn rebuild_activity(&mut self) {
        self.active_instances.clear();
        self.latest_instances.clear();
        self.instance_counts.clear();
        self.instance_sequences.clear();

        let mut records = self.resources.values().cloned().collect::<Vec<_>>();
        records.sort_by(|left, right| left.updated_at.cmp(&right.updated_at));
        for record in records {
            let Some(instance_id) = text_detail(&record.details, "resource_instance_id") else {
                continue;
            };
            let base_key = registry_key(record.scope.as_ref(), &record.resource_id);
            let sequence = instance_sequence(&instance_id).unwrap_or(1);
            self.instance_counts
                .entry(base_key.clone())
                .and_modify(|count| *count = (*count).max(sequence))
                .or_insert(sequence);
            self.instance_sequences
                .insert(format!("{base_key}|instance:{instance_id}"), sequence);
            self.update_instance_activity(
                &record.scope,
                &record.resource_id,
                &instance_id,
                &record.lifecycle,
            );
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn resource_lifecycle(record_type: &Identifier) -> Option<ResourceLifecycle> {
    match record_type.as_str() {
        "resource.registered" | "resource.attached" | "resource.opening" => {
            Some(ResourceLifecycle::Registered)
        }
        "resource.opened" | "resource.traceability.updated" => Some(ResourceLifecycle::Opened),
        "resource.closed" => Some(ResourceLifecycle::Closed),
        "resource.failed" => Some(ResourceLifecycle::Failed),
        "resource.removed" => Some(ResourceLifecycle::Removed),
        _ => None,
    }
}

fn merge_details(target: &mut BTreeMap<String, Value>, details: &BTreeMap<String, Value>) {
    for (key, value) in details {
        target.insert(key.clone(), value.clone());
    }
}

fn resource_identifier(details: &BTreeMap<String, Value>, scope: &Option<RecordScope>) -> String {
    text_detail(details, "resource_id")
        .or_else(|| text_detail(details, "resource_name"))
        .or_else(|| text_detail(details, "name"))
        .unwrap_or_else(|| {
            scope
                .as_ref()
                .and_then(|scope| scope.test_execution_id.as_ref())
                .map(|test_execution_id| format!("resource-{}", test_execution_id.as_str()))
                .unwrap_or_else(|| "resource".to_owned())
        })
}

fn registry_key(scope: Option<&RecordScope>, resource_id: &str) -> String {
    let mut parts = Vec::new();
    if let Some(scope) = scope {
        if let Some(test_cycle_id) = &scope.test_cycle_id {
            parts.push(format!("cycle:{}", test_cycle_id.as_str()));
        }
        if let Some(test_execution_id) = &scope.test_execution_id {
            parts.push(format!("execution:{}", test_execution_id.as_str()));
        }
        if let Some(case_id) = &scope.test_case_id {
            parts.push(format!("case:{}", case_id.as_str()));
        }
    }
    parts.push(format!("resource:{resource_id}"));
    parts.join("|")
}

fn generated_instance_id(resource_id: &str, sequence: usize) -> String {
    format!("{}_{}", sanitize_resource_id(resource_id), sequence)
}

fn sanitize_resource_id(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "resource".to_owned()
    } else {
        sanitized
    }
}

fn next_instance_sequence(counts: &mut BTreeMap<String, usize>, base_key: &str) -> usize {
    let count = counts.entry(base_key.to_owned()).or_default();
    *count += 1;
    *count
}

fn instance_sequence(instance_id: &str) -> Option<usize> {
    instance_id.rsplit_once('_')?.1.parse::<usize>().ok()
}

fn instance_label(resource_name: &str, sequence: usize) -> String {
    format!("{} Session #{sequence}", title_label(resource_name))
}

fn text_detail(details: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    match details.get(key) {
        Some(Value::Text(value)) if !value.trim().is_empty() => Some(value.clone()),
        _ => None,
    }
}

fn title_label(value: &str) -> String {
    value
        .split(['_', '-', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_uppercase(), chars.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn identifier(value: &str) -> Identifier {
    Identifier::new(value)
        .unwrap_or_else(|error| panic!("invalid resource identifier {value}: {error}"))
}

/* === Tests === */
