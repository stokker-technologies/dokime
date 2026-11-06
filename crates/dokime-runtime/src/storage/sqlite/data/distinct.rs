/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Indexed Distinct Values And Row-Local Queries

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

// Local Imports
use crate::storage::Error;
use dokime_domain::model::{execution::RecordScope, values::Value};

/* === Constants === */

pub(super) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_distinct_values (
    execution TEXT NOT NULL, kind TEXT NOT NULL, source TEXT NOT NULL, path TEXT NOT NULL,
    value_key BLOB NOT NULL, value BLOB NOT NULL, sort_key BLOB NOT NULL,
    PRIMARY KEY(execution,kind,source,path,value_key)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS storage_distinct_order
    ON storage_distinct_values(execution,kind,source,path,sort_key,value_key);
CREATE INDEX IF NOT EXISTS storage_distinct_key
    ON storage_distinct_values(execution,kind,source,value_key,sort_key);
CREATE TABLE IF NOT EXISTS storage_value_postings (
    execution TEXT NOT NULL, kind TEXT NOT NULL, source TEXT NOT NULL, path TEXT NOT NULL,
    value_key BLOB NOT NULL, value_type INTEGER NOT NULL, position TEXT NOT NULL, scope TEXT NOT NULL,
    file_id TEXT REFERENCES files(id) ON DELETE CASCADE,
    record_position TEXT REFERENCES storage_data_entries(position) ON DELETE CASCADE,
    PRIMARY KEY(execution,kind,source,path,value_key,position),
    FOREIGN KEY(execution,kind,source,path,value_key)
        REFERENCES storage_distinct_values(execution,kind,source,path,value_key)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS storage_value_field_position
    ON storage_value_postings(execution,kind,source,path,position,value_key);
CREATE INDEX IF NOT EXISTS storage_value_numeric
    ON storage_value_postings(execution,kind,source,path,value_key,position,value_type)
    WHERE value_type IN (1,2,3);
CREATE INDEX IF NOT EXISTS storage_value_file ON storage_value_postings(file_id);
CREATE INDEX IF NOT EXISTS storage_value_record ON storage_value_postings(record_position);
CREATE TRIGGER IF NOT EXISTS storage_distinct_retire AFTER DELETE ON storage_value_postings BEGIN
    DELETE FROM storage_distinct_values
    WHERE execution=old.execution AND kind=old.kind AND source=old.source AND path=old.path AND value_key=old.value_key
    AND NOT EXISTS(SELECT 1 FROM storage_value_postings p
        WHERE p.execution=old.execution AND p.kind=old.kind AND p.source=old.source
        AND p.path=old.path AND p.value_key=old.value_key);
END;
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Row<'a> {
    pub kind: &'a str,
    pub source: &'a str,
    pub position: u64,
    pub file: Option<&'a str>,
    pub record: bool,
    pub scope: &'a RecordScope,
}

#[cfg(feature = "native")]
#[derive(Clone)]
pub(crate) struct Query {
    pub execution: String,
    pub kind: String,
    pub source: String,
    pub path: Vec<String>,
    pub fallback: Option<Vec<String>>,
    pub scope: RecordScope,
    pub filters: Vec<Predicate>,
    pub order: Option<Field>,
    pub key: Option<[u8; 32]>,
}

#[cfg(feature = "native")]
#[derive(Clone)]
pub(crate) struct Predicate {
    pub field: Field,
    pub operator: dokime_domain::model::document::FilterOperator,
    pub expected: Option<Value>,
}

#[cfg(feature = "native")]
pub(crate) struct Page {
    pub values: Vec<Selected>,
}

#[cfg(feature = "native")]
#[derive(Clone)]
pub(crate) struct Selected {
    pub value: Value,
    pub key: [u8; 32],
    pub order: Vec<u8>,
    pub first: u64,
    pub last: bool,
}

/* === Enums === */

#[cfg(feature = "native")]
#[derive(Clone)]
pub(crate) enum Field {
    Constant(Option<Value>),
    Path { path: Vec<String>, fallback: Option<Vec<String>> },
}

/* === Traits === */

/* === Implementations === */

#[cfg(feature = "native")]
impl super::State {
    pub(crate) async fn distinct_page(
        &self, at: harmos::storage::StorageCursor, query: Query, after: Option<[u8; 32]>, maximum: usize, mut work: super::super::QueryWork,
    ) -> Result<Page, Error> {
        let _cancel = work.cancellation();
        self.read(move |connection| {
            connection.create_scalar_function("document_alive", 0, rusqlite::functions::FunctionFlags::SQLITE_UTF8, move |_| {
                work.check().map(|()| true).map_err(|error| rusqlite::Error::UserFunctionError(error))
            })?;
            connection.create_scalar_function(
                "document_matches",
                3,
                rusqlite::functions::FunctionFlags::SQLITE_UTF8 | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC,
                |arguments| {
                    let actual = arguments
                        .get::<Option<Vec<u8>>>(0)?
                        .map(|bytes| serde_json::from_slice::<Value>(&bytes))
                        .transpose()
                        .map_err(|error| rusqlite::Error::UserFunctionError(Box::new(error)))?;
                    let operator: dokime_domain::model::document::FilterOperator = serde_json::from_str(&arguments.get::<String>(1)?)
                        .map_err(|error| rusqlite::Error::UserFunctionError(Box::new(error)))?;
                    let expected = arguments
                        .get::<Option<Vec<u8>>>(2)?
                        .map(|bytes| serde_json::from_slice::<Value>(&bytes))
                        .transpose()
                        .map_err(|error| rusqlite::Error::UserFunctionError(Box::new(error)))?;
                    Ok(crate::document::resolve::value_matches_operator(actual.as_ref(), operator, expected.as_ref()))
                },
            )?;
            let result = read_page(connection, at, query, after, maximum);
            connection.remove_function("document_alive", 0)?;
            connection.remove_function("document_matches", 3)?;
            Ok(result)
        })
        .await?
    }
}

/* === Public Functions === */

pub(super) fn insert<'a>(
    connection: &Connection, row: Row<'_>, fields: impl IntoIterator<Item = (&'a str, &'a Value)>,
) -> Result<(), Error> {
    let Some(execution) = row.scope.test_execution_id.as_ref() else { return Ok(()) };
    let scope = serde_json::to_string(row.scope)?;
    let position = super::position(row.position);
    let mut dictionary = connection.prepare_cached("INSERT OR IGNORE INTO storage_distinct_values VALUES (?1,?2,?3,?4,?5,?6,?7)")?;
    let mut postings = connection.prepare_cached("INSERT OR IGNORE INTO storage_value_postings VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)")?;
    let mut pending = fields.into_iter().map(|(name, value)| (vec![name], value)).collect::<Vec<_>>();
    while let Some((path, value)) = pending.pop() {
        // Telemetry Schemas Retain Root Presence; Null Has No Distinct Value Or Aggregate Contribution.
        if row.kind == "telemetry" && path.len() == 1 && matches!(value, Value::Null) {
            continue;
        }
        if let Value::Object(fields) = value {
            pending.extend(fields.iter().map(|(name, value)| {
                let mut path = path.clone();
                path.push(name);
                (path, value)
            }));
        }
        let path = serde_json::to_string(&path)?;
        let encoded = serde_json::to_vec(value)?;
        let sort = sort_key(value)?;
        let identity = if numeric_key(value).is_some() { sort.as_slice() } else { encoded.as_slice() };
        let key = Sha256::digest(identity);
        let value_type = match value {
            Value::Number(_) => 1,
            Value::Integer(_) => 2,
            Value::Unsigned(_) => 3,
            _ => 0,
        };
        dictionary.execute(params![execution.as_str(), row.kind, row.source, path, key.as_slice(), encoded, sort])?;
        postings.execute(params![
            execution.as_str(),
            row.kind,
            row.source,
            path,
            key.as_slice(),
            value_type,
            position,
            scope,
            row.file,
            row.record.then_some(position.as_str())
        ])?;
    }
    Ok(())
}

pub(crate) fn value_key(value: &Value) -> Result<[u8; 32], Error> {
    let encoded = match numeric_key(value) {
        Some(key) => key,
        None => serde_json::to_vec(value)?,
    };
    Ok(Sha256::digest(encoded).into())
}

/* === Private Functions === */

pub(crate) fn posted_value(bytes: &[u8], kind: u8) -> Result<Value, Error> {
    let original: Value = serde_json::from_slice(bytes)?;
    let value = match (kind, &original) {
        (0, Value::Number(_) | Value::Integer(_) | Value::Unsigned(_)) => return Err(Error::Corrupt("numeric posting type is absent")),
        (0, _) => return Ok(original),
        (1, value) => {
            Value::Number(dokime_domain::model::values::numeric_value(value).ok_or(Error::Corrupt("numeric posting is not numeric"))?)
        }
        (2, Value::Integer(value)) => Value::Integer(*value),
        (2, Value::Unsigned(value)) => Value::Integer(i64::try_from(*value).map_err(|_| Error::Corrupt("signed posting is out of range"))?),
        (2, Value::Number(value)) => Value::Integer(*value as i64),
        (3, Value::Unsigned(value)) => Value::Unsigned(*value),
        (3, Value::Integer(value)) => Value::Unsigned(u64::try_from(*value).map_err(|_| Error::Corrupt("unsigned posting is negative"))?),
        (3, Value::Number(value)) => Value::Unsigned(*value as u64),
        _ => return Err(Error::Corrupt("numeric posting type is invalid")),
    };
    if dokime_domain::model::values::numeric_cmp(&original, &value) != Some(std::cmp::Ordering::Equal) && original != value {
        return Err(Error::Corrupt("numeric posting type changes its value"));
    }
    Ok(value)
}

#[cfg(feature = "native")]
fn read_page(
    connection: &Connection, at: harmos::storage::StorageCursor, query: Query, after: Option<[u8; 32]>, maximum: usize,
) -> Result<Page, Error> {
    use rusqlite::types::Value as Sql;
    super::saved(connection)?.history.check(&at.history)?;
    let mut arguments = vec![
        Sql::Text(query.execution),
        Sql::Text(query.kind),
        Sql::Text(query.source),
        Sql::Text(serde_json::to_string(&query.path)?),
        Sql::Text(super::position(at.sequence)),
    ];
    let mut selected = String::from(
        "p.execution=d.execution AND p.kind=d.kind AND p.source=d.source
        AND p.path=d.path AND p.value_key=d.value_key AND p.position<=?5",
    );
    let mut candidates = "d.execution=?1 AND d.kind=?2 AND d.source=?3 AND d.path=?4".to_owned();
    if let Some(fallback) = &query.fallback {
        arguments.push(Sql::Text(serde_json::to_string(fallback)?));
        let fallback = arguments.len();
        arguments.push(Sql::Text(serde_json::to_string(&query.path[..1])?));
        let root = arguments.len();
        let declared = schema_field("p", root);
        selected = format!(
            "p.execution=d.execution AND p.kind=d.kind AND p.source=d.source
            AND p.value_key=d.value_key AND p.position<=?5 AND p.path IN (?4,?{fallback})
            AND (p.path=?4 OR NOT EXISTS(SELECT 1 FROM storage_value_postings exact
                WHERE exact.execution=p.execution AND exact.kind=p.kind AND exact.source=p.source
                AND exact.path=?{root} AND exact.position=p.position)
                AND NOT {declared})"
        );
        candidates = format!(
            "d.execution=?1 AND d.kind=?2 AND d.source=?3 AND (d.path=?4 OR (d.path=?{fallback}
            AND NOT EXISTS(SELECT 1 FROM storage_distinct_values preferred WHERE preferred.execution=d.execution AND preferred.kind=d.kind
                AND preferred.source=d.source AND preferred.path=?4 AND preferred.value_key=d.value_key)))"
        );
    }
    for (field, value) in [
        ("session_id", query.scope.session_id.as_ref().map(ToString::to_string)),
        ("test_cycle_id", query.scope.test_cycle_id.as_ref().map(ToString::to_string)),
        ("test_case_id", query.scope.test_case_id.as_ref().map(ToString::to_string)),
        ("template_id", query.scope.template_id.as_ref().map(ToString::to_string)),
        ("suite_id", query.scope.suite_id.as_ref().map(ToString::to_string)),
        ("step_id", query.scope.step_id.as_ref().map(ToString::to_string)),
        ("criterion_id", query.scope.criterion_id.as_ref().map(ToString::to_string)),
    ] {
        if let Some(value) = value {
            arguments.push(Sql::Text(value));
            selected.push_str(&format!(" AND json_extract(p.scope,'$.{field}')=?{}", arguments.len()));
        }
    }
    for filter in query.filters {
        let field = field_sql(filter.field, &mut arguments)?;
        arguments.push(Sql::Text(serde_json::to_string(&filter.operator)?));
        let operator = arguments.len();
        arguments.push(filter.expected.map(|value| serde_json::to_vec(&value).map(Sql::Blob)).transpose()?.unwrap_or(Sql::Null));
        selected.push_str(&format!(" AND document_matches({field},?{operator},?{})", arguments.len()));
    }
    let selector = format!("FROM storage_value_postings p WHERE {selected}");
    let ordered = query.order.is_some();
    let order = if let Some(field) = query.order {
        let field = field_sql(field, &mut arguments)?;
        connection.create_scalar_function(
            "document_order",
            1,
            rusqlite::functions::FunctionFlags::SQLITE_UTF8 | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC,
            |arguments| {
                arguments
                    .get::<Option<Vec<u8>>>(0)?
                    .map(|bytes| {
                        let value: Value =
                            serde_json::from_slice(&bytes).map_err(|error| rusqlite::Error::UserFunctionError(Box::new(error)))?;
                        sort_key(&value).map_err(|error| rusqlite::Error::UserFunctionError(Box::new(error)))
                    })
                    .transpose()
            },
        )?;
        format!("coalesce((SELECT document_order({field}) {selector} AND {field} IS NOT NULL ORDER BY p.position LIMIT 1),x'ff')")
    } else {
        "d.sort_key".into()
    };
    if let Some(key) = query.key {
        arguments.push(Sql::Blob(key.to_vec()));
        candidates.push_str(&format!(" AND d.value_key=?{}", arguments.len()));
    }
    arguments.push(Sql::Text(serde_json::to_string(&dokime_domain::model::document::FilterOperator::Exists)?));
    candidates.push_str(&format!(" AND document_matches(d.value,?{},NULL)", arguments.len()));
    let cursor = after
        .map(|key| {
            arguments.push(Sql::Blob(key.to_vec()));
            if ordered {
                format!(
                    " WHERE (ordering,first,value_key)>(SELECT ordering,first,value_key FROM selected WHERE value_key=?{} LIMIT 1)",
                    arguments.len()
                )
            } else {
                format!(
                    " AND (d.sort_key,d.value_key)>(SELECT sort_key,value_key FROM storage_distinct_values
                    WHERE execution=?1 AND kind=?2 AND source=?3 AND value_key=?{} LIMIT 1)",
                    arguments.len()
                )
            }
        })
        .unwrap_or_default();
    arguments.push(Sql::Integer(i64::try_from(maximum.saturating_add(1)).map_err(|_| Error::Invalid("distinct page is too large"))?));
    let sql = if ordered {
        format!(
            "WITH selected AS NOT MATERIALIZED (SELECT d.value_key,d.value,{order} AS ordering,
                (SELECT p.position {selector} ORDER BY p.position LIMIT 1) AS first,
                (SELECT p.value_type {selector} ORDER BY p.position LIMIT 1) AS value_type
        FROM storage_distinct_values d WHERE {candidates} AND document_alive() AND EXISTS(SELECT 1 {selector}))
        SELECT value_key,value,ordering,first,value_type FROM selected{cursor} ORDER BY ordering,first,value_key LIMIT ?{}",
            arguments.len()
        )
    } else {
        format!(
            "SELECT d.value_key,d.value,d.sort_key,(SELECT p.position {selector} ORDER BY p.position LIMIT 1),
            (SELECT p.value_type {selector} ORDER BY p.position LIMIT 1) FROM storage_distinct_values d WHERE {candidates}{cursor}
        AND document_alive() AND EXISTS(SELECT 1 {selector}) ORDER BY d.sort_key,d.value_key LIMIT ?{}",
            arguments.len()
        )
    };
    let mut statement = connection.prepare(&sql)?;
    let mut rows = statement.query(rusqlite::params_from_iter(arguments))?;
    let mut values = Vec::new();
    let mut more = false;
    while let Some(row) = rows.next()? {
        if values.len() == maximum {
            more = true;
            break;
        }
        let key: Vec<u8> = row.get(0)?;
        let key = key.try_into().map_err(|_| Error::Corrupt("distinct key is invalid"))?;
        let value = super::blob(row, 1)?;
        values.push(Selected {
            value: posted_value(value, row.get(4)?)?,
            key,
            order: row.get(2)?,
            first: super::sequence(&row.get::<_, String>(3)?)?,
            last: false,
        });
    }
    if !more && let Some(value) = values.last_mut() {
        value.last = true;
    }
    Ok(Page { values })
}

#[cfg(feature = "native")]
fn field_sql(field: Field, arguments: &mut Vec<rusqlite::types::Value>) -> Result<String, Error> {
    use rusqlite::types::Value as Sql;
    match field {
        Field::Constant(value) => {
            arguments.push(value.map(|value| serde_json::to_vec(&value).map(Sql::Blob)).transpose()?.unwrap_or(Sql::Null));
            Ok(format!("?{}", arguments.len()))
        }
        Field::Path { path, fallback } => {
            let scalar = path.len() == 1;
            let root = serde_json::to_string(&path[..1])?;
            arguments.push(Sql::Text(serde_json::to_string(&path)?));
            let primary = arguments.len();
            let declared = schema_field("p", primary);
            let mut nulls = if scalar { declared.clone() } else { "0".into() };
            let paths = if let Some(fallback) = fallback {
                let scalar = fallback.len() == 1;
                arguments.push(Sql::Text(serde_json::to_string(&fallback)?));
                let fallback = arguments.len();
                if scalar {
                    nulls.push_str(&format!(" OR (NOT {declared} AND {})", schema_field("p", fallback)));
                }
                arguments.push(Sql::Text(root));
                let root = arguments.len();
                let declared = schema_field("q", root);
                format!(
                    "(q.path=?{primary} OR (q.path=?{fallback} AND NOT EXISTS(SELECT 1 FROM storage_value_postings exact
                    WHERE exact.execution=q.execution AND exact.kind=q.kind AND exact.source=q.source
                    AND exact.position=q.position AND exact.path=?{root})
                    AND NOT {declared}))"
                )
            } else {
                format!("q.path=?{primary}")
            };
            arguments.push(Sql::Blob(serde_json::to_vec(&Value::Null)?));
            Ok(format!(
                "coalesce((SELECT v.value FROM storage_value_postings q JOIN storage_distinct_values v
                USING(execution,kind,source,path,value_key) WHERE q.execution=p.execution AND q.kind=p.kind
                AND q.source=p.source AND q.position=p.position AND {paths}
                ORDER BY (q.path=?{primary}) DESC LIMIT 1),CASE WHEN {nulls} THEN ?{} END)",
                arguments.len()
            ))
        }
    }
}

#[cfg(feature = "native")]
fn schema_field(alias: &str, path: usize) -> String {
    format!(
        "EXISTS(SELECT 1 FROM streams s, json_each(s.contract,'$.columns') c
        WHERE {alias}.kind='telemetry' AND s.file_id={alias}.file_id
        AND json_extract(c.value,'$.metadata.identifier')=json_extract(?{path},'$[0]'))"
    )
}

pub(crate) fn sort_key(value: &Value) -> Result<Vec<u8>, Error> {
    if let Some(key) = numeric_key(value) {
        return Ok(key);
    }
    let mut key = vec![match value {
        Value::Bool(_) => 4,
        Value::Datetime(_) => 5,
        Value::Array(_) => 6,
        Value::Object(_) => 7,
        _ => 3,
    }];
    match value {
        Value::Text(text) => key.extend_from_slice(text.as_bytes()),
        Value::Null => (),
        Value::Bool(value) => key.extend_from_slice(if *value { b"true" } else { b"false" }),
        Value::Datetime(value) => key.extend_from_slice(&((value.unix_timestamp_nanos() as u128) ^ (1 << 127)).to_be_bytes()),
        value => key.extend_from_slice(&serde_json::to_vec(value)?),
    }
    Ok(key)
}

// Exact Binary Scientific Notation Gives Integers And Floats The Same Equality And Order.
fn numeric_key(value: &Value) -> Option<Vec<u8>> {
    let (negative, magnitude, exponent) = match value {
        Value::Integer(value) => (*value < 0, value.unsigned_abs(), 0i16),
        Value::Unsigned(value) => (false, *value, 0),
        Value::Number(value) if value.is_finite() => {
            let bits = value.to_bits();
            let fraction = bits & ((1u64 << 52) - 1);
            let exponent = ((bits >> 52) & 2047) as i16;
            let (magnitude, exponent) = if exponent == 0 { (fraction, -1074) } else { (fraction | (1u64 << 52), exponent - 1075) };
            (value.is_sign_negative(), magnitude, exponent)
        }
        _ => return None,
    };
    if magnitude == 0 {
        return Some(vec![1]);
    }
    let leading = magnitude.leading_zeros();
    let exponent = exponent + (63 - leading) as i16;
    let mut key = Vec::with_capacity(11);
    key.push(if negative { 0 } else { 2 });
    key.extend_from_slice(&((exponent as u16) ^ 0x8000).to_be_bytes());
    key.extend_from_slice(&(magnitude << leading).to_be_bytes());
    if negative {
        for byte in &mut key[1..] {
            *byte = !*byte;
        }
    }
    Some(key)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn distinct_pages_match_filtered_rows_at_old_cut() {
        use dokime_domain::model::{document::FilterOperator, identity::TestExecutionId};
        let root = tempfile::tempdir().unwrap();
        let state = super::super::State::open(root.path()).await.unwrap();
        let saved = state.data_open(None).await.unwrap();
        let execution = TestExecutionId::from_seed("distinct-page-owner");
        let scope = RecordScope::default().with_test_execution(execution.clone());
        let raw = (1..=247u64).map(|position| (position, ((position * 7) % 19) as i64, position % 3 != 0)).collect::<Vec<_>>();
        let admitted = raw.clone();
        let admitted_scope = scope.clone();
        state
            .call(move |connection| {
                Ok((|| -> Result<(), Error> {
                    let transaction = connection.transaction()?;
                    transaction.execute("INSERT INTO files VALUES ('fixture','telemetry','fixture.arrow',0,zeroblob(32),'fixture')", [])?;
                    for (position, value, included) in admitted {
                        let fields: BTreeMap<String, Value> = BTreeMap::from([
                            ("metric.name".into(), Value::Object(BTreeMap::from([("group".into(), Value::Integer(value))]))),
                            ("include".into(), Value::Bool(included)),
                            ("order".into(), Value::Unsigned(1000 - position)),
                        ]);
                        insert(
                            &transaction,
                            Row {
                                kind: "telemetry",
                                source: "fixture",
                                position,
                                file: Some("fixture"),
                                record: false,
                                scope: &admitted_scope,
                            },
                            fields.iter().map(|(name, value)| (name.as_str(), value)),
                        )?;
                    }
                    transaction.execute("UPDATE storage_data_head SET saved=?1", [super::super::position(247)])?;
                    transaction.commit()?;
                    Ok(())
                })())
            })
            .await
            .unwrap()
            .unwrap();
        let cut = harmos::storage::StorageCursor { history: saved.history, sequence: 123 };
        let settings = crate::config::query_defaults();
        let config = settings.compose(&crate::config::storage_defaults(), &crate::config::schema_defaults()).unwrap();
        let mut reference = BTreeMap::new();
        for (position, value, included) in raw {
            if position <= cut.sequence && included {
                reference.entry(value).or_insert((1000 - position, position));
            }
        }
        let mut reference = reference.into_iter().collect::<Vec<_>>();
        reference.sort_by_key(|(_, (order, first))| (*order, *first));
        let expected = reference.into_iter().map(|(value, _)| Value::Integer(value)).collect::<Vec<_>>();
        let mut actual = Vec::new();
        let mut after = None;
        loop {
            let query = Query {
                execution: execution.to_string(),
                kind: "telemetry".into(),
                source: "fixture".into(),
                path: vec!["metric.name".into(), "group".into()],
                fallback: None,
                scope: scope.clone(),
                filters: vec![Predicate {
                    field: Field::Path {
                        path: vec!["include".into()],
                        fallback: None,
                    },
                    operator: FilterOperator::Eq,
                    expected: Some(Value::Bool(true)),
                }],
                order: Some(Field::Path {
                    path: vec!["order".into()],
                    fallback: None,
                }),
                key: None,
            };
            let page = state
                .distinct_page(cut.clone(), query, after, 3, super::super::super::QueryWork::document(config.clone(), "fixture"))
                .await
                .unwrap();
            let Some(last) = page.values.last() else { break };
            after = Some(last.key);
            let complete = last.last;
            actual.extend(page.values.into_iter().map(|selected| selected.value));
            if complete {
                break;
            }
        }
        assert_eq!(actual, expected);
        state
            .call(|connection| {
                connection.execute("DELETE FROM files WHERE id='fixture'", [])?;
                Ok(())
            })
            .await
            .unwrap();
        let retained: i64 = state
            .read(|connection| connection.query_row("SELECT count(*) FROM storage_distinct_values", [], |row| row.get(0)))
            .await
            .unwrap();
        assert_eq!(retained, 0);
        state.close().await.unwrap();
    }

    #[tokio::test]
    async fn nullable_aliases_use_the_saved_schema() {
        use crate::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole};
        use dokime_domain::model::{
            document::FilterOperator,
            identity::{Identifier, Metadata, TestExecutionId},
        };
        let root = tempfile::tempdir().unwrap();
        let state = super::super::State::open(root.path()).await.unwrap();
        let saved = state.data_open(None).await.unwrap();
        let execution = TestExecutionId::from_seed("nullable-alias-owner");
        let scope = RecordScope::default().with_test_execution(execution.clone());
        let owner = scope.clone();
        state.call(move |connection| {
            let result = (|| -> Result<(), Error> {
                let transaction = connection.transaction()?;
                for (position, names) in [(1, vec!["group", "name", "metric.name", "detail"]), (2, vec!["group", "name", "detail"])] {
                    let file = format!("fixture.{position}");
                    let columns = names.iter().map(|name| {
                        let value_type = if *name == "detail" {
                            dokime_domain::model::values::ValueType::Object
                        } else {
                            dokime_domain::model::values::ValueType::Integer
                        };
                        TelemetryColumn::new(Metadata::new(Identifier::new(*name).unwrap(), *name, None), value_type, TelemetryRole::Value)
                    }).collect();
                    let contract = TelemetryContract {
                        stream: Identifier::new("fixture").unwrap(),
                        schema_version: position as u32,
                        metadata: Metadata::new(Identifier::new("fixture").unwrap(), "Fixture", None),
                        template_id: None,
                        owner: None,
                        storage_schema: None,
                        columns,
                        default_x: None,
                        interval: None,
                        allow_extra_fields: false,
                        scalar: false,
                        validation_mode: None,
                        foreign: false,
                    };
                    transaction.execute(
                        "INSERT INTO files VALUES (?1,'telemetry',?2,0,zeroblob(32),'fixture')",
                        params![file, format!("{file}.arrow")],
                    )?;
                    transaction.execute(
                        "INSERT INTO streams(file_id,execution,session,stream,schema_key,contract,rows,first_position,last_position,durable_end,hash_state,finalized)
                        VALUES (?1,?2,'fixture','fixture','fixture',?3,1,?4,?4,0,x'',0)",
                        params![file, owner.test_execution_id.as_ref().unwrap().as_str(), serde_json::to_string(&contract)?, super::super::position(position)],
                    )?;
                    let mut fields = BTreeMap::from([
                        ("group", Value::Unsigned(position)),
                        ("name", Value::Integer(7)),
                        ("detail", Value::Object(BTreeMap::from([("nullable".into(), Value::Null)]))),
                    ]);
                    if position == 1 {
                        fields.insert("metric.name", Value::Null);
                    }
                    insert(
                        &transaction,
                        Row {kind: "telemetry", source: "fixture", position, file: Some(&file), record: false, scope: &owner},
                        fields.iter().map(|(name,value)| (*name,value)),
                    )?;
                }
                transaction.execute("UPDATE storage_data_head SET saved=?1", [super::super::position(2)])?;
                transaction.commit()?;
                Ok(())
            })();
            Ok(result)
        }).await.unwrap().unwrap();
        let cut = harmos::storage::StorageCursor { history: saved.history, sequence: 2 };
        let config =
            crate::config::query_defaults().compose(&crate::config::storage_defaults(), &crate::config::schema_defaults()).unwrap();
        let query = Query {
            execution: execution.to_string(),
            kind: "telemetry".into(),
            source: "fixture".into(),
            path: vec!["metric.name".into()],
            fallback: Some(vec!["name".into()]),
            scope: scope.clone(),
            filters: vec![],
            order: None,
            key: None,
        };
        let page = state
            .distinct_page(cut.clone(), query.clone(), None, 10, super::super::super::QueryWork::document(config.clone(), "fixture"))
            .await
            .unwrap();
        assert_eq!(page.values.into_iter().map(|value| value.value).collect::<Vec<_>>(), vec![Value::Integer(7)]);
        for (path, fallback, operator, expected) in [
            (vec!["metric.name"], Some(vec!["name"]), FilterOperator::Eq, vec![Value::Unsigned(1)]),
            (vec!["metric.name"], Some(vec!["name"]), FilterOperator::Ne, vec![Value::Unsigned(2)]),
            (vec!["detail", "nullable"], None, FilterOperator::Eq, vec![Value::Unsigned(1), Value::Unsigned(2)]),
            (vec!["detail", "missing"], None, FilterOperator::Eq, vec![]),
        ] {
            let query = Query {
                path: vec!["group".into()],
                fallback: None,
                filters: vec![Predicate {
                    field: Field::Path {
                        path: path.into_iter().map(String::from).collect(),
                        fallback: fallback.map(|names| names.into_iter().map(String::from).collect()),
                    },
                    operator,
                    expected: Some(Value::Null),
                }],
                ..query.clone()
            };
            let page = state
                .distinct_page(cut.clone(), query, None, 10, super::super::super::QueryWork::document(config.clone(), "fixture"))
                .await
                .unwrap();
            assert_eq!(page.values.into_iter().map(|value| value.value).collect::<Vec<_>>(), expected);
        }
        state.close().await.unwrap();
    }

    #[test]
    fn mixed_numeric_keys_preserve_exact_order_and_equality() {
        let values = [
            Value::Integer(i64::MIN),
            Value::Number(-0.5),
            Value::Integer(0),
            Value::Number(f64::from_bits(1)),
            Value::Number(0.5),
            Value::Unsigned(9_007_199_254_740_992),
            Value::Unsigned(9_007_199_254_740_993),
            Value::Unsigned(u64::MAX),
            Value::Number(f64::MAX),
        ];
        for pair in values.windows(2) {
            assert_eq!(
                numeric_key(&pair[0]).cmp(&numeric_key(&pair[1])),
                dokime_domain::model::values::numeric_cmp(&pair[0], &pair[1]).unwrap()
            );
        }
        for (left, right) in [(Value::Integer(3), Value::Number(3.0)), (Value::Unsigned(0), Value::Number(-0.0))] {
            assert_eq!(value_key(&left).unwrap(), value_key(&right).unwrap());
        }
    }
}
