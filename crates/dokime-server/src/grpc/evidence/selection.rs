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
use sha2::{Digest, Sha256};
use tonic::Status;
// Local Imports
use dokime_domain::model::identity::wire as common;
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */
pub(super) const MAX_ALTERNATIVES: usize = 64;
pub(super) const MAX_SEVERITIES: usize = 4;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Selection {
    value: dokime_domain::model::evidence::wire::CapturedRecordSelection,
    pub(super) stream: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Selection {
    pub(super) fn new(value: Option<dokime_domain::model::evidence::wire::CapturedRecordSelection>) -> Result<Self, Status> {
        let mut value = value.unwrap_or_default();
        for items in [&mut value.record_types, &mut value.sources] {
            if items.len() > MAX_ALTERNATIVES {
                return Err(Status::invalid_argument("selection exceeds 64 alternatives"));
            }
            for item in items.iter() {
                super::super::value::text(item)?;
            }
            items.sort();
            items.dedup();
        }
        for kind in &value.record_types {
            dokime_domain::model::identity::Identifier::new(kind).map_err(|_| Status::invalid_argument("invalid record type"))?;
        }
        if value.severities.len() > MAX_SEVERITIES
            || value.severities.iter().any(|severity| {
                !matches!(*severity, common::Severity::Debug | common::Severity::Info | common::Severity::Warn | common::Severity::Error)
            })
        {
            return Err(Status::invalid_argument("expected Debug, Info, Warn or Error severity"));
        }
        value.severities.sort_by_key(|severity| match severity {
            common::Severity::Debug => 2,
            common::Severity::Info => 3,
            common::Severity::Warn => 4,
            common::Severity::Error => 5,
            _ => 0,
        });
        value.severities.dedup();
        if let Some(scope) = &mut value.scope {
            for id in [
                &mut scope.session_id,
                &mut scope.cycle_id,
                &mut scope.case_id,
                &mut scope.execution_id,
                &mut scope.step_id,
                &mut scope.criterion_id,
                &mut scope.template_id,
                &mut scope.suite_id,
            ]
            .into_iter()
            .flatten()
            {
                let parsed = uuid::Uuid::parse_str(id).map_err(|_| Status::invalid_argument("invalid scope UUID"))?;
                if parsed.is_nil() {
                    return Err(Status::invalid_argument("scope UUID must not be nil"));
                }
                *id = parsed.to_string();
            }
            if scope == &common::EvidenceScope::default() {
                value.scope = None;
            }
        }
        let stream = format!(
            "evidence.records:{}",
            Sha256::digest(harmos::Message::encode_to_vec(&value)).iter().map(|byte| format!("{byte:02x}")).collect::<String>()
        );
        Ok(Self { value, stream })
    }

    pub(super) fn matches(&self, record: &RecordEntry) -> bool {
        let value = &self.value;
        if !value.record_types.is_empty() && !value.record_types.iter().any(|v| v == record.record_type.as_str())
            || !value.sources.is_empty() && !value.sources.contains(&record.source)
            || !value.severities.is_empty() && !value.severities.contains(&super::canonical::severity(record.severity))
        {
            return false;
        }
        let Some(selected) = &value.scope else {
            return true;
        };
        let Some(scope) = &record.scope else {
            return false;
        };
        let Ok(actual) = super::super::value::canonical::scope(scope) else {
            return false;
        };
        [
            (&selected.session_id, &actual.session_id),
            (&selected.cycle_id, &actual.cycle_id),
            (&selected.case_id, &actual.case_id),
            (&selected.execution_id, &actual.execution_id),
            (&selected.step_id, &actual.step_id),
            (&selected.criterion_id, &actual.criterion_id),
            (&selected.template_id, &actual.template_id),
            (&selected.suite_id, &actual.suite_id),
        ]
        .into_iter()
        .all(|(selected, actual)| selected.as_ref().is_none_or(|v| actual.as_ref() == Some(v)))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
