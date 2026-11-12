/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Fixed Document Values

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, ExecutionCut, Result, Store};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn document_value(&self, cut: &ExecutionCut, expression: &v1::DataExpression) -> Result<v1::Value> {
        self.document_value_filtered(cut, expression, &[])
    }
    pub fn document_value_filtered(
        &self, cut: &ExecutionCut, expression: &v1::DataExpression, filters: &[v1::PlotFilter],
    ) -> Result<v1::Value> {
        let stream = crate::plot_selection::reference(expression)
            .filter(|(_, record)| !*record)
            .map_or("", |(reference, _)| reference.identifier.as_str());
        let compiler = crate::plot_selection::Compiler::new(
            cut,
            stream,
            filters,
            crate::plot_selection::reference(expression).filter(|(_, record)| *record).map(|(reference, _)| reference.identifier.as_str()),
        )?;
        let sql = compiler.expression(expression, 0)?;
        let rows = compiler.rows(expression)?;
        let conditions = filters.iter().map(|filter| compiler.filter(filter)).collect::<Result<Vec<_>>>()?;
        let condition = if conditions.is_empty() { "1".into() } else { conditions.join(" AND ") };
        let mut root = expression;
        while let Some(v1::data_expression::Kind::Transform(transform)) = &root.kind {
            let Some(input) = transform.input.as_deref() else { break };
            root = input;
        }
        let aggregate = matches!(root.kind, Some(v1::data_expression::Kind::Aggregate(_)));
        let query = if aggregate {
            format!("SELECT {sql}")
        } else {
            format!("SELECT {sql} FROM {rows} a WHERE {condition} ORDER BY source_ns DESC,position DESC LIMIT 1")
        };
        let connection = self.lock()?;
        use rusqlite::OptionalExtension;
        let value = connection.query_row(&query, [], |row| row.get::<_, rusqlite::types::Value>(0)).optional()?;
        let value = value
            .ok_or_else(|| Error::Invalid("Read Document Value: Capture Matching Evidence Before Evaluating This Expression".into()))?;
        if let Some(v1::data_expression::Kind::Literal(value)) = &expression.kind {
            return Ok(value.clone());
        }
        if let Some(v1::data_expression::Kind::Parameter(key) | v1::data_expression::Kind::Dimension(key)) = &expression.kind
            && let Some(v1::origin::Kind::PluginRun(run)) = cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref())
            && let Some(value) = run.test_case.as_ref().and_then(|case| case.parameters.get(key))
        {
            return Ok(value.clone());
        }
        if value == rusqlite::types::Value::Null {
            return Err(Error::Invalid("Read Document Value: Capture Matching Evidence Before Evaluating This Expression".into()));
        }
        wire_value(value)
    }
    pub fn document_distinct(
        &self, cut: &ExecutionCut, expression: &v1::DataExpression, order: Option<&v1::DataExpression>, filters: &[v1::PlotFilter],
        offset: usize, maximum: usize,
    ) -> Result<(Vec<v1::Value>, u64)> {
        let stream = crate::plot_selection::reference(expression)
            .filter(|(_, record)| !*record)
            .map_or("", |(reference, _)| reference.identifier.as_str());
        let compiler = crate::plot_selection::Compiler::new(
            cut,
            stream,
            filters,
            crate::plot_selection::reference(expression).filter(|(_, record)| *record).map(|(reference, _)| reference.identifier.as_str()),
        )?;
        let value = compiler.expression(expression, 0)?;
        let order = order.map(|order| compiler.expression(order, 0)).transpose()?.unwrap_or_else(|| value.clone());
        let rows = compiler.rows(expression)?;
        let conditions = filters.iter().map(|filter| compiler.filter(filter)).collect::<Result<Vec<_>>>()?;
        let condition = if conditions.is_empty() { "1".into() } else { conditions.join(" AND ") };
        let query = format!(
            "SELECT value FROM (SELECT ({value}) value,({order}) ordering,source_ns,position FROM {rows} a WHERE {condition}) WHERE value IS NOT NULL GROUP BY value ORDER BY ordering,MIN(position) LIMIT ?1 OFFSET ?2"
        );
        let connection = self.lock()?;
        let total: i64 = connection.query_row(
            &format!(
                "SELECT COUNT(*) FROM (SELECT ({value}) value FROM {rows} a WHERE {condition} GROUP BY value) WHERE value IS NOT NULL"
            ),
            [],
            |row| row.get(0),
        )?;
        let mut query = connection.prepare(&query)?;
        let values = query
            .query_map(rusqlite::params![maximum as i64, offset as i64], |row| row.get::<_, rusqlite::types::Value>(0))?
            .map(|row| wire_value(row?))
            .collect::<Result<Vec<_>>>()?;
        Ok((values, total as u64))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn wire_value(value: rusqlite::types::Value) -> Result<v1::Value> {
    let kind = match value {
        rusqlite::types::Value::Null => v1::value::Kind::Null(v1::Null {}),
        rusqlite::types::Value::Integer(value) => v1::value::Kind::Integer(value),
        rusqlite::types::Value::Real(value) if value.is_finite() => v1::value::Kind::Number(value),
        rusqlite::types::Value::Text(value) => v1::value::Kind::Text(value),
        _ => return Err(Error::Invalid("Read Document Value: Select a Finite Scalar Expression".into())),
    };
    Ok(v1::Value { kind: Some(kind) })
}

/* === Tests === */
