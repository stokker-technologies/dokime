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
use std::{num::NonZeroUsize, sync::Arc};

// Third Party Imports
use harmos::{schema::graph::Field, storage::*};
use rusqlite::{Connection, types::Value};

// Local Imports
use super::super::{Error, State, query};
use super::{SavedData, entry, index::keys, position, sequence};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Read {
    pub view: ReadView,
    pub request: ObjectBrowseRequest,
    pub cursor: Option<ObjectBrowseCursor>,
    pub limits: ReadLimits,
    pub maximum: NonZeroUsize,
}

struct Plan {
    candidates: String,
    arguments: Vec<Value>,
    predicates: Vec<String>,
    order: Vec<(String, ReadOrder, Value)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl State {
    pub(super) async fn data_browse(
        &self,
        cut: SavedData,
        read: Read,
        execution: Option<Arc<query::Budget>>,
    ) -> Result<ObjectBrowsePage, Error> {
        let Read {
            view,
            request,
            cursor,
            limits,
            maximum,
        } = read;
        self.call_read(execution, move |connection| {
            Ok((|| {
                request.check_view(&view, cursor.as_ref())?;
                let plan = Plan::new(&cut, &request, cursor.as_ref())?;
                validate(connection, &plan, &request, maximum)?;
                let count = limits.rows.get();
                count
                    .checked_add(1)
                    .filter(|count| *count <= i64::MAX as usize)
                    .ok_or(Error::Data("browse count exceeds SQLite range"))?;
                let sql = plan.sql(count + 1);
                let mut statement = connection.prepare(&sql)?;
                let mut rows =
                    statement.query(rusqlite::params_from_iter(plan.arguments.iter()))?;
                let mut entries = Vec::with_capacity(count);
                let mut more = false;
                while let Some(row) = rows.next()? {
                    if entries.len() == count {
                        more = true;
                        break;
                    }
                    let schema = row
                        .get_ref(1)?
                        .as_str()
                        .map_err(|_| Error::Corrupt("invalid object schema index"))?;
                    let item = row
                        .get_ref(2)?
                        .as_str()
                        .map_err(|_| Error::Corrupt("invalid object item index"))?;
                    let version = row
                        .get_ref(3)?
                        .as_str()
                        .map_err(|_| Error::Corrupt("invalid object version index"))?;
                    let mut values = Vec::with_capacity(request.columns.len());
                    for (index, column) in request.columns.iter().enumerate() {
                        values.push(keys::decode(column, row.get_ref(6 + 2 * index)?)?);
                    }
                    let metadata =
                        ObjectMetadata {
                            schema: serde_json::from_str(schema)?,
                            item: ObjectKey::new(item, NonZeroUsize::new(1024).unwrap())?,
                            position: StorageCursor {
                                history: cut.history.clone(),
                                sequence: sequence(row.get_ref(0)?.as_str().map_err(|_| {
                                    Error::Corrupt("invalid object position index")
                                })?)?,
                            },
                            version: serde_json::from_str(version)?,
                            values,
                        };
                    request.check_row(&view, &metadata)?;
                    entries.push(metadata);
                    if entry::check_size(&entries, limits.bytes.get()).is_err() {
                        entries.pop();
                        more = true;
                        break;
                    }
                }
                // Cursor Framing Is Part Of The Encoded Output Bound
                // Trim a suffix if the request-bound continuation needs its space.
                loop {
                    let next = if more {
                        Some(ObjectBrowseCursor::new(
                            &view,
                            request.clone(),
                            entries
                                .last()
                                .ok_or(Error::Data(
                                    "first scalar row or cursor exceeds page byte bound",
                                ))?
                                .clone(),
                        )?)
                    } else {
                        None
                    };
                    let page = ObjectBrowsePage { entries, next };
                    if entry::check_size(&page, limits.bytes.get()).is_ok() {
                        page.check(&view, &request, cursor.as_ref(), limits)?;
                        return Ok(page);
                    }
                    entries = page.entries;
                    if entries.pop().is_none() {
                        return Err(Error::Data("scalar page exceeds byte bound"));
                    }
                    more = true;
                }
            })())
        })
        .await?
    }
}

impl Plan {
    fn new(
        cut: &SavedData,
        request: &ObjectBrowseRequest,
        cursor: Option<&ObjectBrowseCursor>,
    ) -> Result<Self, Error> {
        request.check()?;
        let selection = &request.selection;
        let schema = selection
            .schema
            .as_ref()
            .ok_or(Error::Data("browse schema is absent"))?;
        let mut arguments = vec![
            Value::Text(schema.id.as_str().into()),
            Value::Text(position(
                selection.through.unwrap_or(cut.sequence).min(cut.sequence),
            )),
            Value::Text(position(cut.retention_generation)),
            Value::Text(serde_json::to_string(schema)?),
            Value::Text(position(selection.after)),
            Value::Text(position(
                selection.through.unwrap_or(cut.sequence).min(cut.sequence),
            )),
        ];
        let mut clauses = vec![
            "entries.kind='object'",
            "entries.schema_id=?1",
            "entries.position<=?2",
            "(entries.retired_at IS NULL OR entries.retired_at>?3)",
            "entries.schema_key=?4",
            "entries.position>?5",
            "entries.position<=?6",
        ];
        if selection.object_versions == ObjectVersions::Latest {
            clauses.push("NOT EXISTS (SELECT 1 FROM storage_data_entries AS newer WHERE newer.kind='object' AND newer.schema_id=entries.schema_id AND newer.item=entries.item AND newer.position>entries.position AND newer.position<=?2 AND (newer.retired_at IS NULL OR newer.retired_at>?3))");
        }
        let item = selection
            .object_key
            .as_ref()
            .map(|item| {
                arguments.push(Value::Text(item.as_str().into()));
                format!(" AND entries.item=?{}", arguments.len())
            })
            .unwrap_or_default();
        let mut columns = String::new();
        for (index, column) in request.columns.iter().enumerate() {
            columns.push_str(&format!(",(SELECT kind FROM storage_object_scalars WHERE position=entries.position AND tag={}) AS k{index},(SELECT value FROM storage_object_scalars WHERE position=entries.position AND tag={}) AS v{index}", column.tag, column.tag));
        }
        let candidates = format!(
            "SELECT entries.position,entries.schema_key,entries.item,entries.object_version,entries.object_fields{columns} FROM storage_data_entries AS entries WHERE {}{item}",
            clauses.join(" AND ")
        );
        let mut predicates = Vec::with_capacity(request.predicates.len());
        for predicate in &request.predicates {
            arguments.push(keys::operand(&predicate.value));
            let operator = match (predicate.comparison, &predicate.value) {
                (ObjectComparison::Equal, ObjectScalar::Null) => "IS",
                (ObjectComparison::NotEqual, ObjectScalar::Null) => "IS NOT",
                (ObjectComparison::Equal, _) => "=",
                (ObjectComparison::NotEqual, _) => "!=",
                (ObjectComparison::Less, _) => "<",
                (ObjectComparison::LessOrEqual, _) => "<=",
                (ObjectComparison::Greater, _) => ">",
                (ObjectComparison::GreaterOrEqual, _) => ">=",
            };
            predicates.push(format!(
                "v{} {operator} ?{}",
                predicate.column,
                arguments.len()
            ));
        }
        let last = cursor.map(ObjectBrowseCursor::last);
        let mut order = Vec::with_capacity(request.order.len() * 2 + 2);
        for sort in &request.order {
            let value = last
                .map(|row| &row.values[sort.column])
                .unwrap_or(&ObjectScalar::Null);
            order.push((
                format!("(v{} IS NULL)", sort.column),
                if sort.nulls == ObjectNullOrder::First {
                    ReadOrder::Descending
                } else {
                    ReadOrder::Ascending
                },
                Value::Integer(i64::from(*value == ObjectScalar::Null)),
            ));
            order.push((
                format!("v{}", sort.column),
                sort.direction,
                keys::operand(value),
            ));
        }
        if request.order.is_empty() {
            order.push((
                "position".into(),
                selection.order,
                Value::Text(position(last.map_or(0, |row| row.position.sequence))),
            ));
        } else {
            order.push((
                "item".into(),
                ReadOrder::Ascending,
                Value::Text(last.map_or("", |row| row.item.as_str()).into()),
            ));
            order.push((
                "position".into(),
                ReadOrder::Ascending,
                Value::Text(position(last.map_or(0, |row| row.position.sequence))),
            ));
        }
        if last.is_some() {
            let mut equal = Vec::with_capacity(order.len());
            let mut continuation = Vec::with_capacity(order.len());
            for (expression, direction, value) in &order {
                arguments.push(value.clone());
                let parameter = arguments.len();
                let comparison = if *direction == ReadOrder::Ascending {
                    ">"
                } else {
                    "<"
                };
                let step = format!("{expression}{comparison}?{parameter}");
                continuation.push(if equal.is_empty() {
                    step
                } else {
                    format!("({} AND {step})", equal.join(" AND "))
                });
                equal.push(format!("{expression} IS ?{parameter}"));
            }
            predicates.push(format!("({})", continuation.join(" OR ")));
        }
        Ok(Self {
            candidates,
            arguments,
            predicates,
            order,
        })
    }

    fn sql(&self, count: usize) -> String {
        let predicate = if self.predicates.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", self.predicates.join(" AND "))
        };
        let order = self
            .order
            .iter()
            .map(|(expression, direction, _)| {
                format!(
                    "{expression} {}",
                    if *direction == ReadOrder::Ascending {
                        "ASC"
                    } else {
                        "DESC"
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "WITH selected AS ({}) SELECT * FROM selected{predicate} ORDER BY {order} LIMIT {count}",
            self.candidates
        )
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn validate(
    connection: &Connection,
    plan: &Plan,
    request: &ObjectBrowseRequest,
    maximum: NonZeroUsize,
) -> Result<(), Error> {
    // Validate The Entire Fixed Selection Before Field Predicates
    // Stream metadata and scalar keys under the native budget, without reading
    // or decoding object bodies. Invalid/nonfinite keys cannot become SQL Null.
    let sql = format!(
        "WITH selected AS ({}) SELECT * FROM selected",
        plan.candidates
    );
    let mut statement = connection.prepare(&sql)?;
    // Only the candidate parameters occur in this statement. Numbered bindings
    // allow the predicate/cursor suffix to remain private to the page statement.
    let parameters = if request.selection.object_key.is_some() {
        7
    } else {
        6
    };
    let mut rows = statement.query(rusqlite::params_from_iter(
        plan.arguments[..parameters].iter(),
    ))?;
    while let Some(row) = rows.next()? {
        let fields = super::blob(row, 4)?;
        if fields.len() > maximum.get() {
            return Err(Error::Corrupt(
                "scalar field metadata exceeds backend bound",
            ));
        }
        let fields: Vec<Field> = serde_json::from_slice(fields)?;
        request.check_schema(&fields)?;
        for (index, column) in request.columns.iter().enumerate() {
            if row.get_ref(5 + 2 * index)?.as_str().ok() != Some(keys::scalar_kind(column.kind)) {
                return Err(Error::Corrupt(
                    "persisted scalar kind differs or is missing",
                ));
            }
            keys::decode(column, row.get_ref(6 + 2 * index)?)?;
        }
    }
    Ok(())
}

/* === Tests === */
