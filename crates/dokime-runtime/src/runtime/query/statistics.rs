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

// Local Imports
use super::{
    samples::{self, Plan},
    snapshots::Scope,
    tokens,
};
use crate::{
    RuntimeHandle,
    config::QueryConfig,
    storage::sqlite::{Moments, QueryWork, State, ordered, unordered},
};
use dokime_domain::model::{
    query::{
        error::{Failure, Reason},
        page::{PageLimits, PageRequest},
        reduction::{Output, QualityCounts, Statistic, StatisticValue, Statistics},
        schema::{Field, FieldType, Schema},
    },
    values::{Value, numeric_value},
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
}

#[derive(Default)]
struct Population {
    rows: u64,
    fields: BTreeMap<String, FieldState>,
}

struct Rank {
    field: String,
    ordinal: u64,
    prefix: u64,
    bins: [u64; 256],
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
    fn push(&mut self, value: Option<&Value>, operation: &str) -> Result<(), Box<Failure>> {
        match value {
            None => self.missing += 1,
            Some(Value::Null) => self.null += 1,
            Some(value) => match numeric_value(value) {
                Some(value) => self
                    .moments
                    .push(value)
                    .map_err(|_| Failure::new(Reason::WorkLimit, operation, "Numeric population counter overflow"))?,
                None => self.non_finite += 1,
            },
        }
        Ok(())
    }
}

impl Rank {
    fn push(&mut self, value: f64, depth: u32) {
        let bits = ordered(value);
        if depth != 0 && bits >> (64 - depth * 8) != self.prefix {
            return;
        }
        self.bins[((bits >> (56 - depth * 8)) & 255) as usize] += 1;
    }

    fn advance(&mut self, operation: &str) -> Result<(), Box<Failure>> {
        for (child, count) in self.bins.iter().enumerate() {
            if self.ordinal < *count {
                self.prefix = (self.prefix << 8) | child as u64;
                self.bins.fill(0);
                return Ok(());
            }
            self.ordinal -= *count;
        }
        Err(Failure::new(Reason::Corruption, operation, "Selected percentile rank is unavailable at this cut"))
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
    let fields: BTreeMap<_, _> =
        outputs.iter().filter(|output| !output.field.is_empty()).map(|output| (output.field.clone(), FieldState::default())).collect();
    config.bound(
        fields.len().saturating_mul(std::mem::size_of::<FieldState>() + 128),
        config.limits().working_bytes as usize / 4,
        "working_bytes",
        operation,
    )?;
    let selected = plan.selection.clone();
    let (population, mut work, _) = local
        .query_telemetry(
            plan.read.clone(),
            QueryWork::new(config.clone(), operation),
            Population { rows: 0, fields },
            move |state, _, sample| {
                if !samples::matches(&selected, &sample) {
                    return Ok(None);
                }
                state.rows += 1;
                for (field, state) in &mut state.fields {
                    state.push(sample.values.get(field), operation)?;
                }
                Ok(None)
            },
        )
        .await?;
    let mut ranks = BTreeMap::new();
    for output in outputs {
        let Some(percentile) = percentile(output.aggregate) else {
            continue;
        };
        let count = population.fields[&output.field].moments.count;
        if count == 0 || percentile == 0 || percentile == 100 {
            continue;
        }
        let numerator = u128::from(count - 1) * u128::from(percentile);
        let lower = (numerator / 100) as u64;
        let upper = lower + u64::from(numerator % 100 != 0);
        for ordinal in [lower, upper] {
            if !ranks.contains_key(&(output.field.clone(), ordinal)) {
                config.bound(
                    (ranks.len() + 1).saturating_mul(std::mem::size_of::<Rank>() + 128),
                    config.limits().working_bytes as usize / 4,
                    "working_bytes",
                    operation,
                )?;
            }
            ranks.entry((output.field.clone(), ordinal)).or_insert_with(|| Rank {
                field: output.field.clone(),
                ordinal,
                prefix: 0,
                bins: [0; 256],
            });
        }
    }
    config.bound(
        ranks.len().saturating_mul(std::mem::size_of::<Rank>() + 128),
        config.limits().working_bytes as usize / 4,
        "working_bytes",
        operation,
    )?;
    if !ranks.is_empty() {
        for depth in 0..8 {
            let selected = plan.selection.clone();
            let (next, next_work, _) = local
                .query_telemetry(plan.read.clone(), work, ranks, move |ranks, _, sample| {
                    if !samples::matches(&selected, &sample) {
                        return Ok(None);
                    }
                    for rank in ranks.values_mut() {
                        if let Some(value) = sample.values.get(&rank.field).and_then(numeric_value).filter(|value| value.is_finite()) {
                            rank.push(value, depth);
                        }
                    }
                    Ok(None)
                })
                .await?;
            work = next_work;
            ranks = next;
            for rank in ranks.values_mut() {
                rank.advance(operation)?;
            }
        }
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
                    rank => {
                        let numerator = u128::from(state.moments.count - 1) * u128::from(rank);
                        let lower = (numerator / 100) as u64;
                        let remainder = numerator % 100;
                        let upper = lower + u64::from(remainder != 0);
                        interpolate(
                            unordered(ranks[&(output.field.clone(), lower)].prefix),
                            unordered(ranks[&(output.field.clone(), upper)].prefix),
                            remainder as f64 / 100.0,
                        )
                    }
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

fn interpolate(low: f64, high: f64, fraction: f64) -> f64 {
    if low == high {
        low
    } else if low.signum() == high.signum() {
        low + (high - low) * fraction
    } else {
        low * (1.0 - fraction) + high * fraction
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_ranks() {
        let values = [-10.0f64, 0.0, f64::MAX, -0.0, 1e-200, -1e200, 3.0, 3.0, -f64::MAX];
        let mut expected = values;
        expected.sort_by(f64::total_cmp);
        for (ordinal, expected) in expected.into_iter().enumerate() {
            let mut rank = Rank {
                field: "value".into(),
                ordinal: ordinal as u64,
                prefix: 0,
                bins: [0; 256],
            };
            for depth in 0..8 {
                for value in values {
                    rank.push(value, depth);
                }
                rank.advance("test.query").unwrap();
            }
            assert_eq!(unordered(rank.prefix).to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn query_quality() {
        let mut field = FieldState::default();
        for value in [
            None,
            Some(Value::Null),
            Some(Value::Number(f64::NAN)),
            Some(Value::Number(f64::INFINITY)),
            Some(Value::Unsigned(u64::MAX)),
            Some(Value::Number(5.0)),
        ] {
            field.push(value.as_ref(), "test.query").unwrap();
        }
        assert_eq!((field.moments.count, field.missing, field.null, field.non_finite), (2, 1, 1, 2));
        assert!(finite(f64::INFINITY, "test.query").is_err());
    }
}
