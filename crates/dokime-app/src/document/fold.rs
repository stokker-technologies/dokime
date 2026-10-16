/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Scalar Folds Over In-Memory Evidence
//
// HARMOS GAP WORKAROUND: Harmos folds scalars inside its query transforms but
// keeps the accumulator crate-private, so documents and domain aggregates over
// rows Dokime already holds fold here with the same semantics: exact columns
// keep `Sum`, `Mean` and `Median` in rational arithmetic, `Min`, `Max` and
// `Last` select by scalar ordering, and floating statistics use the Harmos
// signal kernels. Replace once Harmos exposes its accumulator.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroU128;

// Third Party Imports
use harmos::{
    Aggregate,
    signal::{Series, Summary},
    storage::query::scalar::{Agg, ExactNumber, ScalarKind, ScalarValue},
};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub(crate) type Result<T> = std::result::Result<T, &'static str>;

/* === Structs === */
#[derive(Clone, Debug)]
pub(crate) struct Accumulator {
    agg: Agg,
    rows: u64,
    count: u64,
    kind: Option<ScalarKind>,
    selected: Option<(u64, ScalarValue)>,
    total: ExactNumber,
    running: Summary,
    values: Vec<ScalarValue>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Accumulator {
    pub(crate) fn new(agg: Agg) -> Self {
        Self {
            agg,
            rows: 0,
            count: 0,
            kind: None,
            selected: None,
            total: ExactNumber::new(false, 0, NonZeroU128::MIN),
            running: Summary::new(),
            values: Vec::new(),
        }
    }

    /// Fold one value observed at `capture` order.
    pub(crate) fn push(&mut self, capture: u64, value: &ScalarValue) -> Result<()> {
        value.check().map_err(|_| "aggregate value is invalid")?;
        self.rows += 1;
        if self.agg == Agg::Last && !matches!(value, ScalarValue::Missing) {
            let later = self
                .selected
                .as_ref()
                .is_none_or(|(previous, _)| capture > *previous);
            if later {
                self.selected = Some((capture, value.clone()));
            }
        }
        if matches!(value, ScalarValue::Missing | ScalarValue::Null) {
            return Ok(());
        }
        self.count += 1;
        let Agg::Of(statistic) = self.agg else {
            return Ok(());
        };
        if statistic == Aggregate::Count {
            return Ok(());
        }
        let value_kind = value.kind().expect("checked non-missing scalar has a kind");
        match self.kind {
            Some(previous)
                if previous != value_kind && !(exact_kind(previous) && exact_kind(value_kind)) =>
            {
                return Err("mixed scalar aggregate types");
            }
            Some(_) => {}
            None => self.kind = Some(value_kind),
        }
        match statistic {
            Aggregate::Min | Aggregate::Max => {
                let replace = match &self.selected {
                    None => true,
                    Some((_, previous)) => {
                        let order = value
                            .compare(previous)
                            .map_err(|_| "aggregate values do not compare")?;
                        if statistic == Aggregate::Min {
                            order.is_lt()
                        } else {
                            order.is_gt()
                        }
                    }
                };
                if replace {
                    self.selected = Some((capture, value.clone()));
                }
            }
            Aggregate::Sum | Aggregate::Mean => {
                if let ScalarValue::Float(value) = value {
                    self.running.push(*value);
                } else {
                    let exact = exact(value).ok_or("aggregate requires numbers")?;
                    self.total = self
                        .total
                        .checked_add(exact)
                        .map_err(|_| "aggregate exceeds exact range")?;
                }
            }
            Aggregate::Rms | Aggregate::Std => {
                self.running
                    .push(approximate(value).ok_or("aggregate requires numbers")?);
            }
            Aggregate::Median | Aggregate::Percentile(_) => {
                if !matches!(
                    value_kind,
                    ScalarKind::Signed
                        | ScalarKind::Unsigned
                        | ScalarKind::Float
                        | ScalarKind::Exact
                ) {
                    return Err("order statistic requires numbers");
                }
                self.values.push(value.clone());
            }
            Aggregate::Count => unreachable!("handled before type checks"),
        }
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<ScalarValue> {
        let statistic = match self.agg {
            Agg::Len => return Ok(ScalarValue::Unsigned(self.rows)),
            Agg::Last => {
                return Ok(self
                    .selected
                    .map_or(ScalarValue::Missing, |(_, value)| value));
            }
            Agg::Of(statistic) => statistic,
        };
        let floating = self.kind == Some(ScalarKind::Float);
        Ok(match statistic {
            Aggregate::Count => ScalarValue::Unsigned(self.count),
            Aggregate::Min | Aggregate::Max => self
                .selected
                .map_or(ScalarValue::Missing, |(_, value)| value),
            // The empty sum is zero, as `Summary` answers it.
            Aggregate::Sum if self.count == 0 => ScalarValue::Unsigned(0),
            Aggregate::Sum if floating => ScalarValue::Float(finite(Some(self.running.sum()))?),
            Aggregate::Mean if floating => ScalarValue::Float(finite(self.running.mean())?),
            Aggregate::Sum => integral(self.total, self.kind),
            Aggregate::Mean if self.count == 0 => ScalarValue::Missing,
            Aggregate::Mean => integral(
                self.total
                    .divide(u128::from(self.count))
                    .map_err(|_| "aggregate exceeds exact range")?,
                self.kind,
            ),
            Aggregate::Rms | Aggregate::Std if self.count == 0 => ScalarValue::Missing,
            Aggregate::Rms => ScalarValue::Float(finite(self.running.rms())?),
            Aggregate::Std => ScalarValue::Float(finite(self.running.std())?),
            Aggregate::Median | Aggregate::Percentile(_) if self.count == 0 => ScalarValue::Missing,
            Aggregate::Percentile(rank) => ordered(&self.values, f64::from(rank))?,
            Aggregate::Median if floating => ordered(&self.values, 50.0)?,
            Aggregate::Median => {
                let mut values = self.values;
                let mut failure = None;
                values.sort_by(|left, right| {
                    left.compare(right).unwrap_or_else(|_| {
                        failure = Some("median values do not compare");
                        std::cmp::Ordering::Equal
                    })
                });
                if let Some(failure) = failure {
                    return Err(failure);
                }
                let values = values
                    .iter()
                    .map(|value| exact(value).expect("median values are numeric"))
                    .collect::<Vec<_>>();
                let lower = values[(values.len() - 1) / 2];
                let upper = values[values.len() / 2];
                let midpoint = lower
                    .checked_add(upper)
                    .and_then(|sum| sum.divide(2))
                    .map_err(|_| "aggregate exceeds exact range")?;
                integral(midpoint, self.kind)
            }
        })
    }
}

/* === Public Functions === */
/// The exact rational value of an integral or exact scalar.
pub(crate) fn exact(value: &ScalarValue) -> Option<ExactNumber> {
    Some(match value {
        ScalarValue::Signed(value) => ExactNumber::new(
            *value < 0,
            u128::from(value.unsigned_abs()),
            NonZeroU128::MIN,
        ),
        ScalarValue::Unsigned(value) => {
            ExactNumber::new(false, u128::from(*value), NonZeroU128::MIN)
        }
        ScalarValue::Exact(value) => *value,
        _ => return None,
    })
}

/* === Private Functions === */
fn exact_kind(kind: ScalarKind) -> bool {
    matches!(
        kind,
        ScalarKind::Signed | ScalarKind::Unsigned | ScalarKind::Exact
    )
}

// The nearest float to a numeric scalar, for statistics that are not exact.
fn approximate(value: &ScalarValue) -> Option<f64> {
    if let ScalarValue::Float(value) = value {
        return Some(*value);
    }
    let exact = exact(value)?;
    let magnitude = exact.numerator as f64 / exact.denominator.get() as f64;
    Some(if exact.negative {
        -magnitude
    } else {
        magnitude
    })
}

fn ordered(values: &[ScalarValue], rank: f64) -> Result<ScalarValue> {
    let mut series = Series::from(
        values
            .iter()
            .map(|value| approximate(value).expect("order statistic values are numeric"))
            .collect::<Vec<_>>(),
    );
    series.sort();
    Ok(ScalarValue::Float(finite(series.percentile(rank))?))
}

fn finite(answer: Option<f64>) -> Result<f64> {
    answer
        .filter(|value| value.is_finite())
        .ok_or("aggregate exceeds finite range")
}

fn integral(value: ExactNumber, kind: Option<ScalarKind>) -> ScalarValue {
    if value.denominator.get() == 1 {
        if kind == Some(ScalarKind::Signed) {
            let signed = i128::try_from(value.numerator)
                .ok()
                .and_then(|n| i64::try_from(if value.negative { -n } else { n }).ok());
            if let Some(signed) = signed {
                return ScalarValue::Signed(signed);
            }
        } else if !value.negative
            && let Ok(unsigned) = u64::try_from(value.numerator)
        {
            return ScalarValue::Unsigned(unsigned);
        }
    }
    ScalarValue::Exact(value)
}

/* === Tests === */
