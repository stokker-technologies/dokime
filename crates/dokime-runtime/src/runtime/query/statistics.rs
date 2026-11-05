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
use std::collections::BTreeMap;

// Third Party Imports
use harmos_storage::query::quantiles::Quantiles;

// Local Imports
use super::{
    batches::{Batch, Cell},
    samples::{self, Plan},
    snapshots::Scope,
    tokens,
};
use crate::{
    RuntimeHandle,
    config::QueryConfig,
    storage::sqlite::{Moments, QueryWork, State},
};
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

#[derive(Default)]
struct FieldState {
    moments: Moments,
    missing: u64,
    null: u64,
    non_finite: u64,
    spool: Option<Quantiles>,
    percentiles: Vec<u8>,
    quantiles: BTreeMap<u8, f64>,
}

#[derive(Default)]
struct Population {
    rows: u64,
    fields: BTreeMap<String, FieldState>,
}

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

impl FieldState {
    fn push(&mut self, value: Cell, operation: &str) -> Result<(), Box<Failure>> {
        match value {
            Cell::Missing => self.missing += 1,
            Cell::Null => self.null += 1,
            Cell::NonFinite => self.non_finite += 1,
            Cell::Number(value) => {
                self.moments.push(value).map_err(|_| Failure::new(Reason::WorkLimit, operation, "Numeric population counter overflow"))?;
                if let Some(spool) = &mut self.spool {
                    spool.push(value).map_err(|_| Failure::new(Reason::ResourceLimit, operation, "Numeric spool write failed"))?;
                }
            }
        }
        Ok(())
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
    let mut fields: BTreeMap<_, _> =
        outputs.iter().filter(|output| !output.field.is_empty()).map(|output| (output.field.clone(), FieldState::default())).collect();
    config.bound(
        fields.len().saturating_mul(std::mem::size_of::<FieldState>() + 128),
        config.limits().working_bytes as usize / 4,
        "working_bytes",
        operation,
    )?;
    let spool_bytes = config.limits().working_bytes as usize / 8 / fields.len().max(1);
    for output in outputs {
        if let Some(rank @ 1..=99) = percentile(output.aggregate) {
            let state = fields.get_mut(&output.field).expect("percentile field");
            state.spool.get_or_insert_with(|| Quantiles::new(spool_bytes));
            if !state.percentiles.contains(&rank) {
                state.percentiles.push(rank);
            }
        }
    }
    let selected = plan.selection.clone();
    let (mut population, mut work) = local
        .query_native_batches(
            plan.read.clone(),
            QueryWork::new(config.clone(), operation),
            Population { rows: 0, fields },
            move |state, input, contract| {
                let batch = Batch::new(&input, &contract, operation);
                let columns = state.fields.keys().map(|field| batch.column(field)).collect::<Vec<_>>();
                for row in 0..input.num_rows() {
                    if !batch.matches(&selected, row)? {
                        continue;
                    }
                    state.rows += 1;
                    for (state, column) in state.fields.values_mut().zip(&columns) {
                        state.push(column.cell(row), operation)?;
                    }
                }
                Ok(())
            },
        )
        .await?;
    // Preserve The Existing Nine-Scan Logical Charge For Nonempty Interior Percentiles.
    if population.fields.values().any(|state| state.spool.is_some() && state.moments.count != 0) {
        work.charge(work.rows.saturating_mul(8), work.bytes.saturating_mul(8))?;
    }
    for state in population.fields.values_mut() {
        if let Some(spool) = &mut state.spool {
            state.quantiles = spool
                .values(&state.percentiles)
                .map_err(|_| Failure::new(Reason::ResourceLimit, operation, "Numeric spool selection failed"))?;
        }
        work.check()?;
    }
    work.check()?;
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
                Some(StatisticValue::Count(state.moments.count))
            } else if state.moments.count == 0 {
                None
            } else if let Some(percentile) = percentile(output.aggregate) {
                let value = match percentile {
                    0 => state.moments.value(harmos::Aggregate::Min).unwrap(),
                    100 => state.moments.value(harmos::Aggregate::Max).unwrap(),
                    rank => state.quantiles[&rank],
                };
                Some(finite(value, operation)?)
            } else {
                Some(finite(state.moments.value(output.aggregate).expect("moment aggregate"), operation)?)
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
            finite: state.moments.count,
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

fn percentile(aggregate: harmos::Aggregate) -> Option<u8> {
    match aggregate {
        harmos::Aggregate::Median => Some(50),
        harmos::Aggregate::Percentile(rank) => Some(rank),
        _ => None,
    }
}

/* === Tests === */
