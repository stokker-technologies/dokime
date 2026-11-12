/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Compiled Plot Selections

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, ExecutionCut, Result, Store};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct PlotGroup {
    pub label: String,
    value: rusqlite::types::Value,
}

pub struct PlotSelection {
    pub request: v1::PlotRequest,
    pub projected: std::collections::BTreeSet<String>,
    pub labels: std::collections::BTreeMap<String, String>,
}
struct Compiler<'a> {
    cut: &'a ExecutionCut,
    stream: &'a str,
    catalog: harmos_protocol::v1::Catalog,
    pool: prost_reflect::DescriptorPool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn plot_group_values(&self, window: crate::NumericWindow<'_>, field: &str, maximum: usize) -> Result<Vec<PlotGroup>> {
        let connection = self.lock()?;
        let mut query=connection.prepare("SELECT DISTINCT COALESCE(number,label) FROM sample_fields WHERE execution=?1 AND stream=?2 AND field=?3 AND source_ns>=?4 AND source_ns<=?5 AND position>?6 AND position<=?7 AND COALESCE(number,label) IS NOT NULL LIMIT ?8")?;
        let mut groups = query
            .query_map(
                rusqlite::params![
                    window.execution,
                    window.stream,
                    field,
                    window.start_ns,
                    window.end_ns,
                    window.after as i64,
                    window.through as i64,
                    maximum as i64 + 1
                ],
                |row| row.get::<_, rusqlite::types::Value>(0),
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if groups.len() > maximum {
            return Err(Error::Invalid("Query Grouped Plot: Narrow the Window or Reduce the Number of Groups".into()));
        }
        let label = |value: &rusqlite::types::Value| match value {
            rusqlite::types::Value::Text(value) => value.clone(),
            rusqlite::types::Value::Integer(value) => value.to_string(),
            rusqlite::types::Value::Real(value) => value.to_string(),
            _ => String::new(),
        };
        groups.sort_by(|left, right| compare_groups(&label(left), &label(right)));
        Ok(groups.into_iter().map(|value| PlotGroup { label: label(&value), value }).collect())
    }
    pub fn select_plot_group(&self, group: Option<&PlotGroup>) -> Result<()> {
        let connection = self.lock()?;
        connection.execute("DELETE FROM temp.plot_group", [])?;
        if let Some(group) = group {
            match &group.value {
                rusqlite::types::Value::Text(value) => {
                    connection.execute("INSERT INTO temp.plot_group VALUES(NULL,?1)", [value])?;
                }
                value => {
                    connection.execute("INSERT INTO temp.plot_group VALUES(?1,NULL)", [value])?;
                }
            }
        }
        Ok(())
    }
    pub fn plot_selection(&self, cut: &ExecutionCut, request: &v1::PlotRequest) -> Result<PlotSelection> {
        let catalog = harmos_protocol::v1::Catalog::decode(cut.execution.catalog.as_slice())?;
        let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(
            catalog.types.clone().ok_or_else(|| Error::Invalid("Query Plot: Restore Frozen Descriptors".into()))?,
        )
        .map_err(|error| Error::Invalid(error.to_string()))?;
        let compiler = Compiler {
            cut,
            stream: &request.stream,
            catalog,
            pool,
        };
        let mut selected = request.clone();
        let filters = request.filters.iter().map(|filter| compiler.filter(filter)).collect::<Result<Vec<_>>>()?;
        let mut condition = if filters.is_empty() { "1".into() } else { filters.join(" AND ") };
        let grouping = request.series_by_expression.clone().or_else(|| {
            request.series_by.as_ref().map(|field| v1::DataExpression {
                kind: Some(v1::data_expression::Kind::Stream(v1::FieldReference {
                    identifier: request.stream.clone(),
                    field: field.clone(),
                })),
                ..Default::default()
            })
        });
        if let Some(group) = &grouping {
            let group = compiler.expression(group, 0)?;
            condition.push_str(&format!(
                " AND (NOT EXISTS(SELECT 1 FROM temp.plot_group) OR ({group})=(SELECT COALESCE(number,label) FROM temp.plot_group))"
            ));
        }
        let mut projections = Vec::new();
        let mut labels = std::collections::BTreeMap::new();
        let mut projected = std::collections::BTreeSet::new();
        let grid_bindings = selected.grid.iter_mut().flat_map(|grid| {
            grid.row
                .iter_mut()
                .chain(grid.column.iter_mut())
                .chain(grid.frame.iter_mut())
                .chain(grid.highlights.iter_mut().flat_map(|highlight| highlight.row.iter_mut().chain(highlight.column.iter_mut())))
        });
        for expression in selected
            .y_expressions
            .iter_mut()
            .chain(selected.x_expression.iter_mut())
            .chain(selected.z_expression.iter_mut())
            .chain(selected.series_by_expression.iter_mut())
            .chain(grid_bindings)
        {
            let authored_label = expression.label.clone();
            let input = match expression.kind.as_mut() {
                Some(v1::data_expression::Kind::Aggregate(aggregate)) => aggregate
                    .input
                    .as_mut()
                    .map(|input| input.as_mut())
                    .ok_or_else(|| Error::Invalid("Query Plot Aggregate: Supply Its Input".into()))?,
                _ => expression,
            };
            let simple =
                matches!(input.kind.as_ref(),Some(v1::data_expression::Kind::Stream(reference)) if reference.identifier==request.stream);
            if simple {
                continue;
            }
            let sql = compiler.expression(input, 0)?;
            let key = format!("__dokime_expression_{}", projections.len());
            projected.insert(key.clone());
            labels.insert(key.clone(), if authored_label.is_empty() { format!("Value {}", projections.len() + 1) } else { authored_label });
            projections.push((key.clone(), sql));
            input.kind = Some(v1::data_expression::Kind::Stream(v1::FieldReference {
                identifier: request.stream.clone(),
                field: key,
            }));
        }
        if let Some(stream) = &selected.z_stream
            && stream != &request.stream
        {
            let expression = v1::DataExpression {
                kind: Some(v1::data_expression::Kind::Stream(v1::FieldReference {
                    identifier: stream.clone(),
                    field: selected.z_field.clone().unwrap_or_default(),
                })),
                ..Default::default()
            };
            let sql = compiler.expression(&expression, 0)?;
            let key = format!("__dokime_expression_{}", projections.len());
            projected.insert(key.clone());
            projections.push((key.clone(), sql));
            selected.z_stream = Some(request.stream.clone());
            selected.z_field = Some(key);
        }

        if !projections.is_empty() || !filters.is_empty() || grouping.is_some() {
            let execution = quote(&cut.execution.identifier);
            let stream = quote(&request.stream);
            let original_condition = condition.replace("a.", "f.");
            let mut sql = format!(
                "CREATE TEMP VIEW sample_fields AS SELECT f.execution,f.stream,f.field,f.position,f.source_ns,f.number,f.label FROM main.sample_fields f WHERE f.execution!={execution} OR f.stream!={stream} OR ({original_condition})"
            );
            for (field, expression) in projections {
                let field = quote(&field);
                sql.push_str(&format!(" UNION ALL SELECT a.execution,{stream},{field},a.position,a.source_ns,CASE WHEN typeof({expression}) IN ('integer','real') THEN ({expression}) END,CASE WHEN typeof({expression})='text' THEN ({expression}) END FROM main.samples a WHERE a.execution={execution} AND a.stream={stream} AND a.position<={} AND ({condition})",cut.frontier));
            }
            self.prepare_projection(&format!("CREATE TEMP TABLE IF NOT EXISTS plot_group(number REAL,label TEXT); DELETE FROM plot_group; DROP VIEW IF EXISTS temp.sample_fields; {sql}"))?;
        }
        selected.filters.clear();
        Ok(PlotSelection { request: selected, projected, labels })
    }
}
impl Compiler<'_> {
    fn expression(&self, expression: &v1::DataExpression, depth: usize) -> Result<String> {
        if depth > 32 {
            return Err(Error::Invalid("Query Plot: Simplify the Expression Nesting".into()));
        }
        use v1::data_expression::Kind;
        match expression.kind.as_ref() {
            Some(Kind::Stream(reference)) => self.reference(reference, false),
            Some(Kind::Record(reference)) => self.reference(reference, true),
            Some(Kind::Literal(value)) => literal(value),
            Some(Kind::Parameter(key) | Kind::Dimension(key)) => {
                let Some(v1::origin::Kind::PluginRun(run)) = self.cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref())
                else {
                    return Err(Error::Invalid("Query Plot: Select Parameters from a PluginRun Origin".into()));
                };
                let value = run
                    .test_case
                    .as_ref()
                    .and_then(|case| case.parameters.get(key))
                    .ok_or_else(|| Error::Invalid(format!("Query Plot: Declare Parameter {key}")))?;
                literal(value)
            }
            Some(Kind::Transform(transform)) => {
                let input = transform.input.as_ref().ok_or_else(|| Error::Invalid("Query Plot Transform: Supply Its Input".into()))?;
                match transform.kind.as_ref() {
                    Some(v1::transform_expression::Kind::Detail(field)) => {
                        let mut input = input.as_ref().clone();
                        let reference = match input.kind.as_mut() {
                            Some(Kind::Stream(reference) | Kind::Record(reference)) => reference,
                            _ => return Err(Error::Invalid("Query Plot Detail: Select a Stream or Record Field".into())),
                        };
                        reference.field = if reference.field.is_empty() { field.clone() } else { format!("{}.{field}", reference.field) };
                        self.expression(&input, depth + 1)
                    }
                    Some(v1::transform_expression::Kind::Scale(value)) => {
                        if !value.is_finite() {
                            return Err(Error::Invalid("Query Plot Scale: Supply a Finite Factor".into()));
                        }
                        Ok(format!("({})*({value})", self.expression(input, depth + 1)?))
                    }
                    Some(v1::transform_expression::Kind::Offset(value)) => {
                        if !value.is_finite() {
                            return Err(Error::Invalid("Query Plot Offset: Supply a Finite Offset".into()));
                        }
                        Ok(format!("({})+({value})", self.expression(input, depth + 1)?))
                    }
                    Some(v1::transform_expression::Kind::PerCase(true)) => self.expression(input, depth + 1),
                    Some(v1::transform_expression::Kind::TimeBetween(previous)) => {
                        let Some(Kind::Record(current)) = input.kind.as_ref() else {
                            return Err(Error::Invalid("Query Plot Time Between: Select Record Occurrences".into()));
                        };
                        let current = quote(&current.identifier);
                        let previous = quote(previous);
                        let time = format!(
                            "(SELECT MAX(e.source_ns) FROM main.record_fields e WHERE e.execution=a.execution AND e.record={current} AND e.source_ns<=a.source_ns AND e.position<={})",
                            self.cut.frontier
                        );
                        Ok(format!(
                            "(({time})-(SELECT MAX(e.source_ns) FROM main.record_fields e WHERE e.execution=a.execution AND e.record={previous} AND e.source_ns<=({time}) AND e.position<={}))/1000000000.0",
                            self.cut.frontier
                        ))
                    }
                    _ => Err(Error::Invalid("Query Plot Transform: Select a Declared Transform".into())),
                }
            }
            _ => Err(Error::Invalid("Query Plot: Select a Row Expression Before Its Aggregate".into())),
        }
    }
    fn reference(&self, reference: &v1::FieldReference, record: bool) -> Result<String> {
        let message = if record {
            let mut message = None;
            for section in &self.catalog.sections {
                if section.r#type == "dokime.v1.RecordContracts" {
                    let contracts = v1::RecordContracts::decode(section.payload.as_slice())?;
                    message = contracts
                        .records
                        .into_iter()
                        .find(|contract| contract.metadata.as_ref().is_some_and(|metadata| metadata.identifier == reference.identifier))
                        .map(|contract| contract.message);
                }
            }
            message.ok_or_else(|| Error::Invalid(format!("Query Plot: Declare Record {}", reference.identifier)))?
        } else {
            self.catalog
                .streams
                .iter()
                .find(|stream| stream.id == reference.identifier)
                .map(|stream| stream.row.clone())
                .ok_or_else(|| Error::Invalid(format!("Query Plot: Declare Stream {}", reference.identifier)))?
        };
        let mut descriptor =
            self.pool.get_message_by_name(&message).ok_or_else(|| Error::Invalid("Query Plot: Restore Its Row Descriptor".into()))?;
        let mut field_name = reference.field.clone();
        if field_name.is_empty() {
            let fields = descriptor
                .fields()
                .filter(|field| !field.is_list() && !field.is_map() && !matches!(field.kind(), prost_reflect::Kind::Message(_)))
                .collect::<Vec<_>>();
            if fields.len() != 1 {
                return Err(Error::Invalid("Query Plot: Name a Scalar Field in This Row".into()));
            }
            field_name = fields[0].name().into();
        }
        for (index, name) in field_name.split('.').enumerate() {
            let field = descriptor
                .get_field_by_name(name)
                .ok_or_else(|| Error::Invalid(format!("Query Plot: Declare Field {}.{field_name}", reference.identifier)))?;
            if field.is_list() || field.is_map() {
                return Err(Error::Invalid("Query Plot: Select a Scalar Field".into()));
            }
            if index + 1 < field_name.split('.').count() {
                let prost_reflect::Kind::Message(next) = field.kind() else {
                    return Err(Error::Invalid("Query Plot Detail: Select a Message Field".into()));
                };
                descriptor = next;
            }
        }
        let field = quote(&field_name);
        let source = quote(&reference.identifier);
        let table = if record { "record_fields" } else { "sample_fields" };
        let key = if record { "record" } else { "stream" };
        let position = if !record && reference.identifier == self.stream {
            "p.position=a.position".into()
        } else {
            format!("p.source_ns<=a.source_ns AND p.position<={}", self.cut.frontier)
        };
        Ok(format!(
            "(SELECT COALESCE(p.number,p.label) FROM main.{table} p WHERE p.execution=a.execution AND p.{key}={source} AND p.field={field} AND {position} ORDER BY p.source_ns DESC,p.position DESC LIMIT 1)"
        ))
    }
    fn filter(&self, filter: &v1::PlotFilter) -> Result<String> {
        let expression = filter.expression.as_ref().ok_or_else(|| Error::Invalid("Query Plot Filter: Supply Its Expression".into()))?;
        let expression = self.expression(expression, 0)?;
        let operator = v1::FilterOperator::try_from(filter.operator)
            .map_err(|_| Error::Invalid("Query Plot Filter: Select a Declared Comparison".into()))?;
        if operator == v1::FilterOperator::Exists {
            return Ok(format!("({expression}) IS NOT NULL"));
        }
        let value = literal(filter.value.as_ref().ok_or_else(|| Error::Invalid("Query Plot Filter: Supply Its Value".into()))?)?;
        if operator == v1::FilterOperator::Contains {
            return Ok(format!("instr(CAST(({expression}) AS TEXT),{value})>0"));
        }
        let operator = match operator {
            v1::FilterOperator::Eq => "=",
            v1::FilterOperator::Ne => "!=",
            v1::FilterOperator::Gt => ">",
            v1::FilterOperator::Ge => ">=",
            v1::FilterOperator::Lt => "<",
            v1::FilterOperator::Le => "<=",
            _ => return Err(Error::Invalid("Query Plot Filter: Select a Declared Comparison".into())),
        };
        Ok(format!("({expression}) {operator} {value}"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn compare_groups(left: &str, right: &str) -> std::cmp::Ordering {
    let number = |value: &str| value.parse::<f64>().ok().filter(|value| value.is_finite());
    match (number(left), number(right)) {
        (Some(a), Some(b)) => a.total_cmp(&b).then_with(|| left.cmp(right)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        _ => left.cmp(right),
    }
}
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
fn literal(value: &v1::Value) -> Result<String> {
    Ok(match value.kind.as_ref() {
        Some(v1::value::Kind::Number(value)) if value.is_finite() => value.to_string(),
        Some(v1::value::Kind::Integer(value)) => value.to_string(),
        Some(v1::value::Kind::UnsignedInteger(value)) => value.to_string(),
        Some(v1::value::Kind::Boolean(value)) => u8::from(*value).to_string(),
        Some(v1::value::Kind::Text(value)) => quote(value),
        Some(v1::value::Kind::Null(_)) => "NULL".into(),
        _ => return Err(Error::Invalid("Query Plot: Select a Finite Scalar Value".into())),
    })
}

/* === Tests === */
