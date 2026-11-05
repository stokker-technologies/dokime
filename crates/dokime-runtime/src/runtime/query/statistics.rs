/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared Exact Arrow Query Reductions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{
    samples::{self, Plan},
    snapshots::Scope,
    tokens,
};
use crate::{RuntimeHandle, config::QueryConfig, storage::sqlite::State};
use dokime_domain::model::query::{
    error::{Failure, Reason},
    page::{PageLimits, PageRequest},
    reduction::{Output, QualityCounts, Statistic, StatisticValue, Statistics},
    schema::{Field, FieldType, Schema},
};
use dokime_operations::query::{AggregateSamples, ListSamples};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn query_aggregate_samples(&self, request: AggregateSamples, message_bytes: usize) -> Result<Statistics, Box<Failure>> {
        let operation = "dokime.query.aggregate_samples";
        tokio::time::timeout(self.query_config().limits().active_time, async {
            request.validate_shape()?;
            self.query_config().bound(
                request.outputs.len(),
                self.query_config().limits().aggregate_outputs,
                "aggregate_outputs",
                operation,
            )?;
            let binding = tokens::binding(&request, self.query_config().limits().working_bytes as usize, operation)?;
            let snapshot =
                self.query_snapshot(Scope::Execution(request.execution_id.clone()), request.at.as_ref(), None, binding, operation).await?;
            let contracts = self.query_snapshot_contracts(&snapshot, operation).await?;
            let mut fields = Vec::new();
            for output in &request.outputs {
                if !output.field.is_empty() && !fields.contains(&output.field) {
                    fields.push(output.field.clone());
                }
            }
            let selection = ListSamples {
                execution_id: request.execution_id,
                stream: request.stream,
                versions: request.versions,
                fields,
                r#where: request.r#where,
                between: request.between,
                bucket: None,
                page: PageRequest { rows: None, bytes: None, after: None },
                at: None,
            };
            let mut plan = Plan::new(&contracts, &selection, &snapshot, self.query_config(), operation)?;
            for output in &request.outputs {
                if !matches!(
                    output.aggregate,
                    harmos::Aggregate::Count
                        | harmos::Aggregate::Sum
                        | harmos::Aggregate::Mean
                        | harmos::Aggregate::Min
                        | harmos::Aggregate::Max
                        | harmos::Aggregate::Median
                        | harmos::Aggregate::Std
                        | harmos::Aggregate::Rms
                        | harmos::Aggregate::Percentile(_)
                ) {
                    return Err(Failure::new(Reason::InvalidSelection, operation, "Unsupported sample aggregate"));
                }
                if output.field.is_empty() {
                    continue;
                }
                let field = plan.selection.schema().fields.iter().find(|field| field.id == output.field).expect("projected output");
                numeric_field(field, operation)?;
            }
            // A Row-Count-Only Request Needs Only Predicate Columns.
            if selection.fields.is_empty() {
                plan.read.fields.retain(|field| selection.r#where.iter().any(|predicate| predicate.field == *field));
            }
            let mut result = reduce(&self.local_storage, plan, &request.outputs, self.query_config().clone()).await?;
            result.snapshot = Some(snapshot.snapshot);
            let limits = PageLimits {
                rows: self.query_config().limits().aggregate_outputs as u32,
                bytes: self.query_config().limits().max_page_bytes.min(message_bytes as u64),
            };
            samples::check_page(&result, &limits, operation)?;
            Ok(result)
        })
        .await
        .map_err(|_| Failure::new(Reason::Deadline, operation, "Statistics exceeded the active-work deadline"))?
    }
}

/* === Public Functions === */

pub(super) fn numeric_field(field: &Field, operation: &str) -> Result<(), Box<Failure>> {
    if matches!(field.value_type, FieldType::Integer | FieldType::Int64 | FieldType::Uint64 | FieldType::Float64) {
        return Ok(());
    }
    let mut failure = Failure::new(Reason::TypeMismatch, operation, "Reductions require a declared numeric field");
    failure.field = Some(field.id.clone());
    Err(failure)
}

pub(super) fn finite(value: f64, operation: &str) -> Result<StatisticValue, Box<Failure>> {
    if !value.is_finite() {
        return Err(Failure::new(Reason::ResourceLimit, operation, "Requested numeric result exceeds finite arithmetic range"));
    }
    Ok(StatisticValue::Number(value))
}

/* === Private Functions === */

async fn reduce(local: &State, plan: Plan, outputs: &[Output], config: QueryConfig) -> Result<Statistics, Box<Failure>> {
    let operation = "dokime.query.aggregate_samples";
    let population = super::datafusion::statistics(local, &plan, outputs, &config).await?;
    let mut schema = Schema {
        identity: plan.selection.schema().identity.clone(),
        versions: plan.selection.schema().versions.clone(),
        fields: Vec::new(),
    };
    let mut values = Vec::new();
    for output in outputs {
        let count = matches!(output.aggregate, harmos::Aggregate::Count);
        let value = if output.field.is_empty() {
            Some(StatisticValue::Count(population.rows))
        } else {
            let state = &population.fields[&output.field];
            if count {
                Some(StatisticValue::Count(state.count))
            } else if state.count == 0 {
                None
            } else {
                Some(finite(state.number(output.aggregate).expect("requested aggregate"), operation)?)
            }
        };
        let unit = if count {
            None
        } else {
            plan.selection.schema().fields.iter().find(|field| field.id == output.field).and_then(|field| field.unit.clone())
        };
        schema.fields.push(Field {
            id: output.alias.clone(),
            value_type: if count { FieldType::Uint64 } else { FieldType::Float64 },
            nullable: !count,
            unit,
        });
        values.push(Statistic {
            alias: output.alias.clone(),
            field: output.field.clone(),
            aggregate: output.aggregate,
            value,
        });
    }
    let quality_counts = population
        .fields
        .into_iter()
        .map(|(field, state)| QualityCounts {
            field,
            finite: state.count,
            missing: state.missing,
            null: state.null,
            non_finite: state.non_finite,
        })
        .collect();
    Ok(Statistics {
        schema: Some(schema),
        values,
        selected_rows: population.rows,
        quality_counts,
        snapshot: None,
        complete: true,
    })
}

/* === Tests === */
