/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// DataFusion Accumulators Preserving Sequential Moments And Exact Bucket Sums

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

// Third Party Imports
use arrow_array::{Array, ArrayRef, Float64Array, StringArray, UInt8Array, UInt32Array};
use arrow_schema::DataType;
use harmos_storage::query::datafusion::{
    Quantiles,
    engine::{
        common::ScalarValue,
        error::{DataFusionError, Result},
        logical_expr::{Accumulator, Expr, Volatility, create_udaf},
    },
};
use serde::{Deserialize, Serialize};

// Local Imports
use crate::{config::QueryConfig, storage::sqlite::Moments};
use dokime_domain::model::query::error::{EffectiveLimit, Failure, Reason};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct Reduced {
    pub samples: u64,
    pub count: u64,
    pub missing: u64,
    pub null: u64,
    pub non_finite: u64,
    pub versions: BTreeSet<u32>,
    pub values: BTreeMap<String, u64>,
    pub spilled: bool,
}

#[derive(Default, Debug)]
pub(super) struct Admission {
    groups: usize,
    partitions: BTreeSet<String>,
}

pub(super) struct Factory {
    config: QueryConfig,
    operation: &'static str,
    bucket: bool,
    admission: Arc<Mutex<Admission>>,
}

struct Reducer {
    moments: Moments,
    reduced: Reduced,
    sum: Sum,
    quantiles: Option<Quantiles>,
    percentiles: Vec<u8>,
    bucket: bool,
    meta: bool,
    admitted: bool,
    config: QueryConfig,
    admission: Arc<Mutex<Admission>>,
    operation: &'static str,
}

#[derive(Default, Debug)]
struct Sum {
    bins: BTreeMap<i16, i128>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::fmt::Debug for Reducer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reducer").field("reduced", &self.reduced).finish_non_exhaustive()
    }
}

impl Reduced {
    pub fn number(&self, aggregate: harmos::Aggregate) -> Option<f64> {
        use harmos::Aggregate;
        let key = match aggregate {
            Aggregate::Sum => "sum".to_owned(),
            Aggregate::Mean => "mean".to_owned(),
            Aggregate::Min => "min".to_owned(),
            Aggregate::Max => "max".to_owned(),
            Aggregate::Std => "std".to_owned(),
            Aggregate::Rms => "rms".to_owned(),
            Aggregate::Median => "p50".to_owned(),
            Aggregate::Percentile(percentile) => format!("p{percentile}"),
            _ => return None,
        };
        self.values.get(&key).map(|bits| f64::from_bits(*bits))
    }
}

impl Accumulator for Reducer {
    fn update_batch(&mut self, values: &[ArrayRef]) -> Result<()> {
        if values[0].is_empty() {
            return Ok(());
        }
        if self.bucket && self.meta && !self.admitted {
            let source = values[3].as_any().downcast_ref::<StringArray>().expect("source").value(0);
            let mut state = self.admission.lock().expect("admission");
            if !state.partitions.contains(source) {
                self.config
                    .bound(state.partitions.len() + 1, self.config.limits().partitions, "partitions", self.operation)
                    .map_err(|error| DataFusionError::External(error))?;
                state.partitions.insert(source.to_owned());
            }
            if state.groups >= self.config.limits().buckets {
                let mut failure =
                    Failure::new(Reason::BucketLimit, self.operation, "Bucket state exhausted; narrow the window or increase every");
                failure.limit = Some(EffectiveLimit {
                    name: "buckets".into(),
                    value: self.config.limits().buckets as u64,
                });
                return Err(DataFusionError::External(failure));
            }
            state.groups += 1;
            self.admitted = true;
        }
        let numbers = values[0].as_any().downcast_ref::<Float64Array>().expect("numbers");
        let quality = values[1].as_any().downcast_ref::<UInt8Array>().expect("quality");
        let versions = values[2].as_any().downcast_ref::<UInt32Array>().expect("versions");
        if self.meta {
            self.reduced.samples += numbers.len() as u64;
            self.reduced.versions.extend(versions.values().iter().copied());
            return Ok(());
        }
        for row in 0..numbers.len() {
            self.reduced.samples += 1;
            self.reduced.versions.insert(versions.value(row));
            match quality.value(row) {
                0 => self.reduced.missing += 1,
                1 => self.reduced.null += 1,
                2 => self.reduced.non_finite += 1,
                _ => {
                    let number = numbers.value(row);
                    let number = if self.bucket && number == 0.0 { 0.0 } else { number };
                    self.moments.push(number).map_err(|_| {
                        DataFusionError::External(Failure::new(Reason::WorkLimit, self.operation, "Numeric population counter overflow"))
                    })?;
                    if self.bucket {
                        self.sum.push(number).map_err(|error| DataFusionError::External(error))?;
                    }
                    if let Some(quantiles) = &mut self.quantiles {
                        quantiles.push(number)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn evaluate(&mut self) -> Result<ScalarValue> {
        self.reduced.count = self.moments.count;
        if self.moments.count != 0 {
            for (name, aggregate) in [
                ("sum", harmos::Aggregate::Sum),
                ("mean", harmos::Aggregate::Mean),
                ("min", harmos::Aggregate::Min),
                ("max", harmos::Aggregate::Max),
                ("std", harmos::Aggregate::Std),
                ("rms", harmos::Aggregate::Rms),
            ] {
                let value = if self.bucket && name == "sum" {
                    self.sum.value(1)
                } else if self.bucket && name == "mean" {
                    self.sum.value(self.moments.count)
                } else {
                    self.moments.value(aggregate).expect("moment result")
                };
                self.reduced.values.insert(name.into(), value.to_bits());
            }
            self.reduced.values.insert("p0".into(), self.moments.value(harmos::Aggregate::Min).unwrap().to_bits());
            self.reduced.values.insert("p100".into(), self.moments.value(harmos::Aggregate::Max).unwrap().to_bits());
            if let Some(quantiles) = &mut self.quantiles {
                self.reduced.spilled = quantiles.spilled();
                for (percentile, value) in quantiles.values(&self.percentiles)? {
                    self.reduced.values.insert(format!("p{percentile}"), value.to_bits());
                }
            }
        }
        Ok(ScalarValue::Utf8(Some(serde_json::to_string(&self.reduced).map_err(|error| DataFusionError::External(Box::new(error)))?)))
    }

    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.quantiles.as_ref().map_or(0, Quantiles::size)
            + self.sum.bins.len() * 128
            + self.reduced.versions.len() * 64
            + self.reduced.values.len() * 128
            + self.percentiles.capacity()
    }

    fn state(&mut self) -> Result<Vec<ScalarValue>> {
        Err(DataFusionError::Execution("Spike requires a single aggregate stage to preserve sequential float results".into()))
    }
    fn merge_batch(&mut self, _states: &[ArrayRef]) -> Result<()> {
        Err(DataFusionError::Execution("Spike cannot merge sequential moment states".into()))
    }
}

impl Sum {
    fn push(&mut self, number: f64) -> std::result::Result<(), Box<Failure>> {
        let bits = number.to_bits();
        let exponent = ((bits >> 52) & 0x7ff) as i16;
        let fraction = bits & ((1u64 << 52) - 1);
        let mantissa = if exponent == 0 { fraction } else { fraction | (1u64 << 52) };
        let mantissa = i128::from(mantissa) * if bits >> 63 == 0 { 1 } else { -1 };
        let exponent = if exponent == 0 { -1074 } else { exponent - 1075 };
        let bin = self.bins.entry(exponent).or_default();
        *bin = bin
            .checked_add(mantissa)
            .ok_or_else(|| Failure::new(Reason::WorkLimit, "dokime.query.list_samples", "Bucket sum counter overflow"))?;
        Ok(())
    }
    fn value(&self, divisor: u64) -> f64 {
        let mut total = 0.0;
        for (exponent, mantissa) in &self.bins {
            let scaled = *mantissa as f64 / divisor as f64;
            let term = if *exponent < -1022 {
                scaled * 2f64.powi(i32::from(*exponent) + 1074) * f64::from_bits(1)
            } else {
                scaled * 2f64.powi(i32::from(*exponent))
            };
            total += term;
        }
        total
    }
}

impl Factory {
    pub fn new(config: &QueryConfig, operation: &'static str, bucket: bool, admission: Arc<Mutex<Admission>>) -> Self {
        Self {
            config: config.clone(),
            operation,
            bucket,
            admission,
        }
    }

    pub fn expression(&self, name: &str, arguments: Vec<Expr>, percentiles: Vec<u8>, meta: bool, memory: usize) -> Expr {
        let config = self.config.clone();
        let admission = self.admission.clone();
        let operation = self.operation;
        let bucket = self.bucket;
        let function = create_udaf(
            name,
            vec![DataType::Float64, DataType::UInt8, DataType::UInt32, DataType::Utf8],
            Arc::new(DataType::Utf8),
            Volatility::Immutable,
            Arc::new(move |_| {
                Ok(Box::new(Reducer {
                    moments: Moments::default(),
                    reduced: Reduced::default(),
                    sum: Sum::default(),
                    quantiles: if percentiles.is_empty() { None } else { Some(Quantiles::new(memory)) },
                    percentiles: percentiles.clone(),
                    bucket,
                    meta,
                    admitted: false,
                    config: config.clone(),
                    admission: admission.clone(),
                    operation,
                }))
            }),
            Arc::new(vec![DataType::Utf8]),
        );
        function.call(arguments)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
