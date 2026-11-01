/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod aggregate;
mod aggregate_samples;
mod list_cycles;
mod list_executions;
mod list_records;
mod list_samples;

/* === Re-Exports === */

pub use aggregate::Aggregate;
pub use aggregate_samples::AggregateSamples;
pub use list_cycles::ListCycles;
pub use list_executions::ListExecutions;
pub use list_records::ListRecords;
pub use list_samples::ListSamples;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::ORDINARY;
use dokime_domain::model::query::wire::*;
use dokime_domain::model::query::{
    error::{Failure, Reason},
    page::{CyclePage, ExecutionPage, PageRequest, RecordPage, SamplePage},
    predicate::Predicate,
    reduction::{BucketRequest, Output, Statistics},
    selection::{CycleSelection, ExecutionSelection, RecordScope, Window},
    snapshot::SnapshotRef,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    Aggregate(Aggregate),
    ListExecutions(ListExecutions),
    ListCycles(ListCycles),
    ListRecords(ListRecords),
    ListSamples(ListSamples),
    AggregateSamples(AggregateSamples),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn validate_page(page: &PageRequest, operation: &str) -> Result<(), Box<Failure>> {
    if page.rows == Some(0) || page.bytes == Some(0) || page.after.as_ref().is_some_and(|token| token.is_empty()) {
        return Err(Failure::new(Reason::InvalidSelection, operation, "Page budgets must be positive and continuation tokens nonempty"));
    }
    Ok(())
}

fn validate_selection(
    id: &str, versions: &[u32], predicates: &[Predicate], window: Option<&Window>, operation: &str,
) -> Result<(), Box<Failure>> {
    if id.is_empty() || versions.contains(&0) || versions.iter().collect::<std::collections::BTreeSet<_>>().len() != versions.len() {
        return Err(Failure::new(Reason::InvalidSelection, operation, "Select one concrete contract and unique positive versions"));
    }
    for predicate in predicates {
        predicate.validate_shape(operation)?;
    }
    if let Some(window) = window {
        window.validate(operation)?;
    }
    Ok(())
}

fn validate_samples(
    execution: &str, stream: &str, versions: &[u32], predicates: &[Predicate], window: Option<&Window>, operation: &str,
) -> Result<(), Box<Failure>> {
    if execution.is_empty() {
        return Err(Failure::new(Reason::InvalidSelection, operation, "Select one execution"));
    }
    validate_selection(stream, versions, predicates, window, operation)
}

fn validate_snapshot(snapshot: Option<&SnapshotRef>, operation: &str) -> Result<(), Box<Failure>> {
    let valid = match snapshot {
        None => true,
        Some(SnapshotRef::Token(token)) => !token.is_empty(),
        Some(SnapshotRef::Checkpoint(checkpoint)) => !checkpoint.execution_identifier.is_empty() && !checkpoint.version.is_empty(),
    };
    if !valid {
        return Err(Failure::new(Reason::InvalidSelection, operation, "Supply a nonempty snapshot token or saved checkpoint"));
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::query::predicate::{Operator, Scalar};
    use harmos::operations::OperationDeclaration;

    #[test]
    fn request_shapes() {
        let mut request = ListSamples {
            execution_id: "run".into(),
            stream: "example.cpu".into(),
            versions: vec![1],
            fields: vec!["cpu.load".into()],
            r#where: vec![Predicate {
                field: "cpu.load".into(),
                operator: Operator::Gte,
                values: vec![Scalar::Number(0.5)],
            }],
            between: None,
            bucket: None,
            page: PageRequest { rows: None, bytes: None, after: None },
            at: None,
        };
        assert!(request.validate_shape().is_ok());
        request.page.rows = Some(0);
        assert!(request.validate_shape().is_err());
        request.page.rows = Some(1);
        request.versions.push(1);
        assert!(request.validate_shape().is_err());
        let mut aggregate = AggregateSamples {
            execution_id: "run".into(),
            stream: "example.cpu".into(),
            versions: vec![],
            r#where: vec![],
            between: None,
            outputs: vec![Output {
                alias: "rows".into(),
                field: String::new(),
                aggregate: harmos::Aggregate::Count,
            }],
            at: None,
        };
        assert!(aggregate.validate_shape().is_ok());
        aggregate.outputs.push(aggregate.outputs[0].clone());
        assert!(aggregate.validate_shape().is_err());
        let records = ListRecords {
            scope: RecordScope::Execution(String::new()),
            record_type: "example.record".into(),
            versions: vec![],
            r#where: vec![],
            between: None,
            page: request.page,
            at: None,
        };
        assert!(records.validate_shape().is_err());
        assert!(validate_page(&PageRequest { rows: None, bytes: Some(0), after: None }, ListRecords::ID).is_err());
    }

    #[test]
    fn wire_roundtrip() {
        let request = ListSamples {
            execution_id: "run".into(),
            stream: "example.cpu".into(),
            versions: vec![],
            fields: vec!["cpu.load".into()],
            r#where: vec![Predicate {
                field: "counter".into(),
                operator: Operator::Eq,
                values: vec![Scalar::Unsigned(u64::MAX)],
            }],
            between: None,
            bucket: None,
            page: PageRequest {
                rows: Some(2),
                bytes: None,
                after: Some("opaque".into()),
            },
            at: Some(SnapshotRef::Token("cut".into())),
        };
        let json = serde_json::to_value(&request).unwrap();
        assert!(json.get("where").is_some());
        let decoded: ListSamples = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(decoded, request);
        let mut invalid = json;
        invalid.as_object_mut().unwrap().insert("timeout".into(), serde_json::json!(1000));
        assert!(serde_json::from_value::<ListSamples>(invalid).is_err());
        let raw = serde_json::json!({"Raw": {"sample_id":"s", "stream":"example.cpu", "schema_version":1, "sequence":1,
            "timestamp":null,"received_at":null,"source":"example", "scope":null,
            "values":null, "summary":null,"metadata":null}});
        let row: dokime_domain::model::query::page::SampleRow = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(serde_json::to_value(row).unwrap(), raw);
    }
}
