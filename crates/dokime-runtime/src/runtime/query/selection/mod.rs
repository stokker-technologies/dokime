/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Captured Query Selection

/* === Modules === */

mod fields;
mod predicate;

/* === Re-Exports === */

/* === Imports === */

// Third Party Imports

// Local Imports
use crate::config::QueryConfig;
use dokime_domain::model::{
    identity::wire::Timestamp,
    query::{
        error::{Failure, Reason},
        predicate::Predicate,
        schema::Schema,
        selection::Window,
    },
    records::RecordContract,
    telemetry::TelemetryContract,
    values::Value,
};
use predicate::PreparedPredicate;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct SelectionRequest<'a> {
    pub versions: &'a [u32],
    pub fields: &'a [String],
    pub predicates: &'a [Predicate],
    pub between: Option<&'a Window>,
    pub operation: &'a str,
}

// Prepared Against Captured Contracts Before Any Evidence Reader Is Opened.
pub struct Selection {
    schema: Schema,
    predicates: Vec<PreparedPredicate>,
    required_fields: Vec<FieldRef>,
    window: Option<Window>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldRef {
    Declared(String),
    Envelope(Envelope),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Envelope {
    Id,
    Timestamp,
    ReceivedAt,
    Source,
    SchemaVersion,
    Severity,
}

/* === Traits === */

/* === Implementations === */

impl Selection {
    pub fn records(
        contracts: &[RecordContract], id: &str, request: SelectionRequest<'_>, config: &QueryConfig,
    ) -> Result<Self, Box<Failure>> {
        validate_request(&request, config)?;
        if !request.fields.is_empty() {
            return Err(Failure::new(Reason::InvalidSelection, request.operation, "Record queries retain all captured details fields"));
        }
        let schema = fields::records(contracts, id, &request, config)?;
        Self::prepare(schema, request, config, true)
    }

    pub fn samples(
        contracts: &[TelemetryContract], id: &str, request: SelectionRequest<'_>, config: &QueryConfig,
    ) -> Result<Self, Box<Failure>> {
        validate_request(&request, config)?;
        let schema = fields::samples(contracts, id, &request, config)?;
        Self::prepare(schema, request, config, false)
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }
    pub fn required_fields(&self) -> &[FieldRef] {
        &self.required_fields
    }

    // The Reader Supplies Declared Values And Reserved Envelope Values Separately.
    // Absent Values Stay Absent; No Ingestion Defaults Are Applied Here.
    pub fn matches<'a>(&self, timestamp: Option<Timestamp>, mut value: impl FnMut(&FieldRef) -> Option<&'a Value>) -> bool {
        if let Some(window) = &self.window {
            let bounded = window.start.is_some() || window.end.is_some();
            if bounded {
                let Some(timestamp) = timestamp.and_then(|time| time.to_datetime().ok()) else {
                    return false;
                };
                if window.start.is_some_and(|start| timestamp < start.to_datetime().expect("validated window"))
                    || window.end.is_some_and(|end| timestamp >= end.to_datetime().expect("validated window"))
                {
                    return false;
                }
            }
        }
        self.predicates.iter().all(|predicate| predicate.matches(value(&predicate.field)))
    }

    fn prepare(mut schema: Schema, request: SelectionRequest<'_>, config: &QueryConfig, records: bool) -> Result<Self, Box<Failure>> {
        let mut predicates = Vec::with_capacity(request.predicates.len());
        let mut required_fields = Vec::new();
        for predicate in request.predicates {
            let (field, kind) = fields::resolve(&schema, &predicate.field, records, request.operation)?;
            if !required_fields.contains(&field) {
                required_fields.push(field.clone());
            }
            predicates.push(PreparedPredicate::new(predicate, field, kind, request.operation)?);
        }
        if !request.fields.is_empty() {
            let mut projected = Vec::with_capacity(request.fields.len());
            for id in request.fields {
                let (field, _) = fields::resolve(&schema, id, records, request.operation)?;
                let FieldRef::Declared(_) = field else {
                    return Err(Failure::new(
                        Reason::InvalidSelection,
                        request.operation,
                        "Projection selects declared columns; envelope attribution is always retained",
                    ));
                };
                if projected.iter().any(|field: &dokime_domain::model::query::schema::Field| field.id == *id) {
                    return Err(Failure::new(Reason::InvalidSelection, request.operation, "Projection fields must be unique"));
                }
                projected.push(schema.fields.iter().find(|field| field.id == *id).expect("resolved field").clone());
            }
            schema.fields = projected;
        }
        for field in &schema.fields {
            let field = FieldRef::Declared(field.id.clone());
            if !required_fields.contains(&field) {
                required_fields.push(field);
            }
        }
        if request.between.is_some_and(|window| window.start.is_some() || window.end.is_some()) {
            let field = FieldRef::Envelope(Envelope::Timestamp);
            if !required_fields.contains(&field) {
                required_fields.push(field);
            }
        }
        config.bound(required_fields.len(), config.limits().fields, "fields", request.operation)?;
        Ok(Self {
            schema,
            predicates,
            required_fields,
            window: request.between.cloned(),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn validate_request(request: &SelectionRequest<'_>, config: &QueryConfig) -> Result<(), Box<Failure>> {
    config.bound(request.predicates.len(), config.limits().predicate_terms, "predicate_terms", request.operation)?;
    config.bound(request.fields.len(), config.limits().fields, "fields", request.operation)?;
    let mut working_bytes = 0_usize;
    for predicate in request.predicates {
        predicate.validate_shape(request.operation)?;
        working_bytes = working_bytes.saturating_add(harmos::MemorySize::memory_size(predicate));
        config.bound(predicate.values.len(), config.limits().in_members, "in_members", request.operation)?;
        config.bound(working_bytes, config.limits().working_bytes as usize, "working_bytes", request.operation)?;
    }
    if let Some(window) = request.between {
        window.validate(request.operation)?;
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::{
        identity::{Identifier, Metadata},
        query::predicate::{Operator, Scalar},
        records::RecordField,
        telemetry::{TelemetryColumn, TelemetryRole},
        values::{MeasurementUnit, ValueType},
    };

    fn config() -> QueryConfig {
        crate::config::query_defaults().compose(&crate::config::storage_defaults(), &crate::config::schema_defaults()).unwrap()
    }

    fn request(predicates: &[Predicate]) -> SelectionRequest<'_> {
        SelectionRequest {
            versions: &[],
            fields: &[],
            predicates,
            between: None,
            operation: "test.query",
        }
    }

    fn predicate(field: &str, operator: Operator, values: Vec<Scalar>) -> Predicate {
        Predicate { field: field.into(), operator, values }
    }

    fn contracts() -> (Vec<RecordContract>, Vec<TelemetryContract>) {
        let mut record = RecordContract::declared("example.measurement", "Measurement");
        let mut sample = TelemetryContract::declared("example.measurement", "Measurement");
        for (id, kind) in
            [("cpu.load", ValueType::Integer), ("status", ValueType::Text), ("object", ValueType::Object), ("enabled", ValueType::Boolean)]
        {
            let metadata = Metadata::new(Identifier::new(id).unwrap(), id, None);
            record.fields.push(RecordField::new(metadata.clone(), kind.clone()));
            sample.columns.push(TelemetryColumn::new(metadata, kind, TelemetryRole::Value));
        }
        record.fields[0].required = false;
        record.fields[0].default = Some(Value::Unsigned(99));
        sample.columns.iter_mut().find(|field| field.metadata.identifier.as_str() == "cpu.load").unwrap().required = false;
        (vec![record], vec![sample])
    }

    #[test]
    fn declared_fields() {
        let config = config();
        let (records, samples) = contracts();
        let terms = [predicate("cpu.load", Operator::Eq, vec![Scalar::Integer(1)])];
        let prepared = Selection::records(&records, "example.measurement", request(&terms), &config).unwrap();
        assert!(prepared.required_fields().contains(&FieldRef::Declared("cpu.load".into())));
        for field in ["cpu", "object.child", "cpu.load[0]", "cpu.load OR 1=1", "@unknown"] {
            let terms = [predicate(field, Operator::Eq, vec![Scalar::Integer(1)])];
            let error = Selection::records(&records, "example.measurement", request(&terms), &config).err().unwrap();
            assert_eq!(error.reason, Reason::UnknownField);
            assert_eq!(error.field.as_deref(), Some(field));
        }
        for (field, operator, value) in [
            ("cpu.load", Operator::Eq, Scalar::Text("1".into())),
            ("enabled", Operator::Gt, Scalar::Boolean(false)),
            ("object", Operator::Eq, Scalar::Integer(1)),
        ] {
            let terms = [predicate(field, operator, vec![value])];
            assert_eq!(
                Selection::samples(&samples, "example.measurement", request(&terms), &config).err().unwrap().reason,
                Reason::TypeMismatch
            );
        }
        let terms = [predicate("cpu.load", Operator::In, vec![Scalar::Integer(1), Scalar::Text("1".into())])];
        assert!(Selection::records(&records, "example.measurement", request(&terms), &config).is_err());
        let terms = [predicate("@severity", Operator::Eq, vec![Scalar::Severity(dokime_domain::model::identity::wire::Severity::Warn)])];
        assert!(Selection::records(&records, "example.measurement", request(&terms), &config).is_ok());
        assert_eq!(
            Selection::samples(&samples, "example.measurement", request(&terms), &config).err().unwrap().reason,
            Reason::UnknownField
        );
        assert_eq!(Selection::records(&records, "example.unknown", request(&[]), &config).err().unwrap().reason, Reason::ContractNotFound);
        let projected = ["status".into()];
        let terms = [predicate("cpu.load", Operator::Gte, vec![Scalar::Integer(1)])];
        let prepared =
            Selection::samples(&samples, "example.measurement", SelectionRequest { fields: &projected, ..request(&terms) }, &config)
                .unwrap();
        assert_eq!(prepared.schema().fields.iter().map(|field| field.id.as_str()).collect::<Vec<_>>(), vec!["status"]);
        assert!(prepared.required_fields().contains(&FieldRef::Declared("cpu.load".into())));
    }

    #[test]
    fn missing_and_null() {
        let config = config();
        let (records, samples) = contracts();
        for (operator, values, expected) in [
            (Operator::Exists, vec![Scalar::Boolean(false)], vec![true, true, false]),
            (Operator::Exists, vec![Scalar::Boolean(true)], vec![false, false, true]),
            (Operator::Eq, vec![Scalar::Integer(0)], vec![false, false, true]),
            (Operator::In, vec![Scalar::Integer(0)], vec![false, false, true]),
        ] {
            let terms = [predicate("cpu.load", operator, values)];
            let record = Selection::records(&records, "example.measurement", request(&terms), &config).unwrap();
            let sample = Selection::samples(&samples, "example.measurement", request(&terms), &config).unwrap();
            for selection in [&record, &sample] {
                let null = Value::Null;
                let zero = Value::Integer(0);
                let matched: Vec<_> =
                    [None, Some(&null), Some(&zero)].into_iter().map(|value| selection.matches(None, |_| value)).collect();
                assert_eq!(matched, expected);
            }
        }
        let all = Selection::records(&records, "example.measurement", request(&[]), &config).unwrap();
        assert!(all.matches(None, |_| None));
        let terms = [predicate("status", Operator::Eq, vec![Scalar::Text("failed".into())])];
        let prepared = Selection::records(&records, "example.measurement", request(&terms), &config).unwrap();
        let capitalized = Value::Text("Failed".into());
        assert!(!prepared.matches(None, |_| Some(&capitalized)));
    }

    #[test]
    fn conjunctive_selection() {
        let config = config();
        let (records, samples) = contracts();
        let mut terms = vec![
            predicate("status", Operator::Eq, vec![Scalar::Text("failed".into())]),
            predicate("cpu.load", Operator::Gte, vec![Scalar::Integer(10)]),
            predicate("cpu.load", Operator::Lt, vec![Scalar::Integer(12)]),
        ];
        for _ in 0..2 {
            let record = Selection::records(&records, "example.measurement", request(&terms), &config).unwrap();
            let sample = Selection::samples(&samples, "example.measurement", request(&terms), &config).unwrap();
            for (load, status, expected) in [(11, "failed", true), (12, "failed", false), (11, "passed", false)] {
                let row = std::collections::BTreeMap::from([("cpu.load", Value::Integer(load)), ("status", Value::Text(status.into()))]);
                for selection in [&record, &sample] {
                    assert_eq!(
                        selection.matches(None, |field| match field {
                            FieldRef::Declared(id) => row.get(id.as_str()),
                            FieldRef::Envelope(_) => None,
                        }),
                        expected
                    );
                }
            }
            terms.reverse();
        }
    }

    #[test]
    fn captured_versions() {
        let config = config();
        let (mut records, _) = contracts();
        let mut second = records[0].clone();
        second.schema_version = 2;
        second.fields.push(RecordField::new(Metadata::new(Identifier::new("extra").unwrap(), "Extra", None), ValueType::Text));
        records.push(second);
        let prepared = Selection::records(&records, "example.measurement", request(&[]), &config).unwrap();
        assert!(prepared.schema().fields.iter().find(|field| field.id == "extra").unwrap().nullable);
        records[1].fields[0].value_type = ValueType::Number;
        let failure = Selection::records(&records, "example.measurement", request(&[]), &config).err().unwrap();
        assert_eq!(failure.reason, Reason::IncompatibleSchema);
        assert_eq!(failure.available_versions, vec![1, 2]);
        let narrowed =
            Selection::records(&records, "example.measurement", SelectionRequest { versions: &[1], ..request(&[]) }, &config).unwrap();
        assert_eq!(
            narrowed.schema().fields.iter().find(|field| field.id == "cpu.load").unwrap().value_type,
            dokime_domain::model::query::schema::FieldType::Integer
        );
        records[1].fields[0].value_type = ValueType::Integer;
        records[1].fields[0].unit = Some(MeasurementUnit::custom(Identifier::new("example.voltage").unwrap(), "Voltage", "V"));
        assert_eq!(
            Selection::records(&records, "example.measurement", request(&[]), &config).err().unwrap().reason,
            Reason::IncompatibleSchema
        );
        assert!(Selection::records(&records, "example.measurement", SelectionRequest { versions: &[3], ..request(&[]) }, &config).is_err());
        let empty = RecordContract::declared("example.empty", "Empty");
        assert!(Selection::records(&[empty], "example.empty", request(&[]), &config).unwrap().schema().fields.is_empty());
    }

    #[test]
    fn utc_windows() {
        let config = config();
        let (records, _) = contracts();
        let start = Timestamp { seconds: 0, nanos: 10 };
        let middle = Timestamp { seconds: 0, nanos: 20 };
        let end = Timestamp { seconds: 0, nanos: 30 };
        let first = Window { start: Some(start), end: Some(middle) };
        let second = Window { start: Some(middle), end: Some(end) };
        let a = Selection::records(&records, "example.measurement", SelectionRequest { between: Some(&first), ..request(&[]) }, &config)
            .unwrap();
        let b = Selection::records(&records, "example.measurement", SelectionRequest { between: Some(&second), ..request(&[]) }, &config)
            .unwrap();
        assert!(a.matches(Some(start), |_| None));
        assert!(!a.matches(Some(middle), |_| None));
        assert!(b.matches(Some(middle), |_| None));
        assert!(!a.matches(None, |_| None));
        let unbounded = Window { start: None, end: None };
        assert!(
            Selection::records(
                &records,
                "example.measurement",
                SelectionRequest {
                    between: Some(&unbounded),
                    ..request(&[])
                },
                &config
            )
            .unwrap()
            .matches(None, |_| None)
        );
        for window in [
            Window { start: Some(end), end: Some(start) },
            Window { start: Some(start), end: Some(start) },
            Window {
                start: Some(Timestamp { seconds: 0, nanos: 1_000_000_000 }),
                end: None,
            },
        ] {
            assert_eq!(
                Selection::records(&records, "example.measurement", SelectionRequest { between: Some(&window), ..request(&[]) }, &config)
                    .err()
                    .unwrap()
                    .reason,
                Reason::InvalidWindow
            );
        }
        let utc = time::OffsetDateTime::UNIX_EPOCH;
        let offset = utc.to_offset(time::UtcOffset::from_hms(2, 0, 0).unwrap());
        assert_eq!(Timestamp::from(utc), Timestamp::from(offset));
    }

    #[test]
    fn selection_limits() {
        let mut settings = crate::config::query_defaults();
        settings.predicate_terms = 1;
        settings.in_members = 2;
        let config = settings.compose(&crate::config::storage_defaults(), &crate::config::schema_defaults()).unwrap();
        let (records, samples) = contracts();
        let terms = [predicate("cpu.load", Operator::In, vec![Scalar::Integer(1), Scalar::Integer(2), Scalar::Integer(3)])];
        let error = Selection::records(&records, "example.measurement", request(&terms), &config).err().unwrap();
        assert_eq!(error.limit.unwrap().name, "in_members");
        let terms = [
            predicate("cpu.load", Operator::Eq, vec![Scalar::Integer(1)]),
            predicate("status", Operator::Eq, vec![Scalar::Text("failed".into())]),
        ];
        let error = Selection::records(&records, "example.measurement", request(&terms), &config).err().unwrap();
        assert_eq!(error.limit.unwrap().name, "predicate_terms");
        let fields = ["cpu.load".into(), "cpu.load".into()];
        assert!(
            Selection::samples(&samples, "example.measurement", SelectionRequest { fields: &fields, ..request(&[]) }, &config).is_err()
        );
    }
}
