/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Incremental Numeric Statistics
//
// A field's statistics advance one value at a time in capture order, so the statistics a writer
// saves after each chunk, extended with rows still pending, equal one pass over every row. The
// moments follow the library's scaled population moments, integer sums and means stay exact, and the value
// histogram keeps exact counts on fine power-of-two bins: their width is always the narrowest that
// fits the extremes in the histogram's resolution, so it depends only on the values it holds.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::VecDeque;
use std::num::NonZeroU128;

// Third Party Imports
use harmos_query::scalar::ExactNumber;
use serde::{Deserialize, Serialize};

// Local Imports
use super::aggregates::{NumericSummary, Values};
use crate::scalar::ScalarValue;
use crate::{Error, Result};

/* === Constants === */

/// The Widest Bin Index a Histogram Keeps; Narrower Bins Would Not Be Representable.
const KEY_BITS: i32 = 100;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Running Count, Extremes, Capture-Order Edges, Exact Integer Total and Scaled Moments.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Running {
    count: u64,
    min: Option<Number>,
    max: Option<Number>,
    first: Option<Number>,
    last: Option<Number>,
    /// Whether Any Value Was Floating, So Sum and Mean Are Floating Too.
    float: bool,
    /// Exact Integer Total, or None After It Overflowed.
    total: Option<i128>,
    moments: Moments,
}

/// Scaled Population Moments, as the Library Keeps Them, So Extremes Stay Finite.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct Moments {
    count: u64,
    sum: f64,
    squares: f64,
    mean: f64,
    lowest: f64,
    highest: f64,
    scale: f64,
    deviations: f64,
}

/// Exact Counts of Finite Values on Bins of Width 2^exponent, at Most `resolution` Bins From the Lowest to the Highest.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Grid {
    resolution: usize,
    count: u64,
    low: f64,
    high: f64,
    exponent: i32,
    /// The Bin Index of `counts[0]`.
    origin: i128,
    /// Empty While Every Value Equals `low`; Grows at Either End as the Extremes Widen.
    counts: VecDeque<u64>,
}

/// One Histogram Bin's Bounds and Count.
#[derive(Debug, PartialEq)]
pub(crate) struct GridBin {
    pub lower: f64,
    pub upper: f64,
    pub count: u64,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum Number {
    Signed(i64),
    Unsigned(u64),
    Float(f64),
}

/* === Traits === */

/* === Implementations === */

impl Default for Moments {
    fn default() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            squares: 0.0,
            mean: 0.0,
            lowest: f64::INFINITY,
            highest: f64::NEG_INFINITY,
            scale: 1.0,
            deviations: 0.0,
        }
    }
}

impl Moments {
    fn push(&mut self, value: f64) {
        let maximum = value.abs().max(if self.count == 0 { 0.0 } else { self.lowest.abs().max(self.highest.abs()) });
        let count = self.count + 1;
        let safe = (f64::MAX / count as f64).sqrt() / 2.0;
        let tiny = maximum > 0.0 && maximum < f64::MIN_POSITIVE.sqrt();
        let scale = if maximum > safe || tiny { maximum } else { self.scale.max(1.0) };
        let ratio = self.scale / scale;
        if self.mean != 0.0 {
            self.mean *= ratio;
        }
        if self.deviations != 0.0 {
            self.deviations *= ratio * ratio;
        }
        self.scale = scale;
        let scaled = value / scale;
        let before = self.mean;
        self.count = count;
        self.sum += value;
        self.squares += value * value;
        self.lowest = self.lowest.min(value);
        self.highest = self.highest.max(value);
        self.mean += (scaled - before) / count as f64;
        self.deviations += (scaled - before) * (scaled - self.mean);
    }

    fn mean(&self) -> f64 {
        self.mean * self.scale
    }

    fn spread(&self) -> f64 {
        (self.deviations / self.count as f64).max(0.0).sqrt()
    }

    fn bound(&self) -> f64 {
        self.lowest.abs().max(self.highest.abs()) / self.scale
    }

    fn std(&self) -> f64 {
        if self.count == 0 { 0.0 } else { self.scale * self.spread().min(self.bound()) }
    }

    fn rms(&self) -> f64 {
        if self.count == 0 { 0.0 } else { self.scale * self.mean.hypot(self.spread()).min(self.bound()) }
    }
}

impl Number {
    fn float(self) -> f64 {
        match self {
            Self::Signed(value) => value as f64,
            Self::Unsigned(value) => value as f64,
            Self::Float(value) => value,
        }
    }

    fn exact(self) -> Option<i128> {
        match self {
            Self::Signed(value) => Some(value.into()),
            Self::Unsigned(value) => Some(value.into()),
            Self::Float(_) => None,
        }
    }

    fn scalar(self) -> ScalarValue {
        match self {
            Self::Signed(value) => ScalarValue::Signed(value),
            Self::Unsigned(value) => ScalarValue::Unsigned(value),
            Self::Float(value) => ScalarValue::Float(value),
        }
    }

    fn below(self, other: Self) -> bool {
        match (self.exact(), other.exact()) {
            (Some(left), Some(right)) => left < right,
            _ => self.float() < other.float(),
        }
    }
}

impl Running {
    /// Push One Column's Valid Finite Values in Row Order; Null and Nonfinite Values Are Missing Measurements.
    pub(crate) fn extend(&mut self, values: &Values<'_>, valid: impl Fn(usize) -> bool) {
        match values {
            Values::Signed(values) => (0..values.len()).filter(|row| valid(*row)).for_each(|row| self.push(Number::Signed(values[row]))),
            Values::Unsigned(values) => {
                (0..values.len()).filter(|row| valid(*row)).for_each(|row| self.push(Number::Unsigned(values[row])));
            }
            Values::Float(values) => {
                (0..values.len())
                    .filter(|row| valid(*row) && values[*row].is_finite())
                    .for_each(|row| self.push(Number::Float(values[row])));
            }
        }
    }

    fn push(&mut self, value: Number) {
        self.count += 1;
        if self.min.is_none_or(|min| value.below(min)) {
            self.min = Some(value);
        }
        if self.max.is_none_or(|max| max.below(value)) {
            self.max = Some(value);
        }
        self.first.get_or_insert(value);
        self.last = Some(value);
        self.float |= matches!(value, Number::Float(_));
        let exact = value.exact().unwrap_or_default();
        self.total = if self.count == 1 { Some(exact) } else { self.total.and_then(|total| total.checked_add(exact)) };
        self.moments.push(value.float());
    }

    /// The Summary Every Plot, Document and Statistics Read Reports for These Values.
    pub(crate) fn finish(&self) -> Result<NumericSummary> {
        let scalar = |value: Option<Number>| value.map_or(ScalarValue::Missing, Number::scalar);
        let (sum, average) = match self.count {
            0 => (ScalarValue::Unsigned(0), ScalarValue::Missing),
            _ if self.float => (ScalarValue::Float(finite(self.moments.sum)?), ScalarValue::Float(finite(self.moments.mean())?)),
            _ => {
                let total = self.total.ok_or(Error::Data("narrow the selection: its exact integer total overflows"))?;
                let total = ExactNumber::new(total < 0, total.unsigned_abs(), NonZeroU128::MIN);
                let mean =
                    total.divide(u128::from(self.count)).map_err(|_| Error::Data("narrow the selection: its exact mean overflows"))?;
                (integral(total, self.first), integral(mean, self.first))
            }
        };
        Ok(NumericSummary {
            count: self.count,
            min: scalar(self.min),
            max: scalar(self.max),
            first: scalar(self.first),
            last: scalar(self.last),
            sum,
            average,
            standard_deviation: self.moments.std(),
            rms: self.moments.rms(),
        })
    }
}

impl Grid {
    pub(crate) fn new(resolution: usize) -> Self {
        Self {
            // Bins Align to Zero, So Values of Both Signs Need Two.
            resolution: resolution.max(2),
            count: 0,
            low: 0.0,
            high: 0.0,
            exponent: 0,
            origin: 0,
            counts: VecDeque::new(),
        }
    }

    pub(crate) fn extend(&mut self, values: &Values<'_>, valid: impl Fn(usize) -> bool) {
        let mut scale = 2f64.powi(-self.exponent);
        match values {
            Values::Signed(values) => (0..values.len()).filter(|row| valid(*row)).for_each(|row| self.add(values[row] as f64, &mut scale)),
            Values::Unsigned(values) => {
                (0..values.len()).filter(|row| valid(*row)).for_each(|row| self.add(values[row] as f64, &mut scale))
            }
            Values::Float(values) => (0..values.len()).filter(|row| valid(*row)).for_each(|row| self.add(values[row], &mut scale)),
        }
    }

    /// Count a Value Inside the Extremes Directly; Any Other Value Takes the Full Push, Which May Widen the Bins.
    fn add(&mut self, value: f64, scale: &mut f64) {
        if !self.counts.is_empty() && value >= self.low && value <= self.high {
            self.count += 1;
            self.counts[((value * *scale).floor() as i128 - self.origin) as usize] += 1;
            return;
        }
        self.push(value);
        *scale = 2f64.powi(-self.exponent);
    }

    pub(crate) fn push(&mut self, value: f64) {
        if !value.is_finite() {
            return;
        }
        self.count += 1;
        if self.count == 1 {
            (self.low, self.high) = (value, value);
            return;
        }
        if self.counts.is_empty() {
            if value == self.low {
                return;
            }
            let (low, count) = (self.low, self.count - 1);
            (self.low, self.high) = (low.min(value), low.max(value));
            self.exponent = narrowest(self.low, self.high, self.resolution);
            self.origin = key(self.low, self.exponent);
            self.counts = VecDeque::from(vec![0; (key(self.high, self.exponent) - self.origin + 1) as usize]);
            self.counts[(key(low, self.exponent) - self.origin) as usize] = count;
        } else if value < self.low || value > self.high {
            // Only Wider Extremes Can Need Wider Bins; While They Still Fit, the Counts Only Grow at an End.
            (self.low, self.high) = (self.low.min(value), self.high.max(value));
            let (lower, upper) = (key(self.low, self.exponent), key(self.high, self.exponent));
            if upper - lower >= self.resolution as i128 || lower.abs().max(upper.abs()) >= 1 << KEY_BITS {
                self.coarsen(narrowest(self.low, self.high, self.resolution).max(self.exponent));
            } else {
                while self.origin > lower {
                    self.counts.push_front(0);
                    self.origin -= 1;
                }
                self.counts.resize((upper - self.origin + 1) as usize, 0);
            }
        }
        self.counts[(key(value, self.exponent) - self.origin) as usize] += 1;
    }

    /// Counts in `bins` Equal Bins From the Lowest to the Highest Value. A Value Lands in Its Own Bin, or the One Below
    /// When It Lies Within One Fine Width Above an Edge, So Integers Spanning at Most the Resolution Land Exactly.
    pub(crate) fn bins(&self, bins: usize) -> Vec<GridBin> {
        if self.count == 0 || bins == 0 {
            return Vec::new();
        }
        let width = if self.counts.is_empty() { 1.0 } else { (self.high - self.low) / bins as f64 };
        let mut output = (0..bins)
            .map(|index| GridBin {
                lower: self.low + index as f64 * width,
                upper: if index + 1 == bins && !self.counts.is_empty() { self.high } else { self.low + (index + 1) as f64 * width },
                count: 0,
            })
            .collect::<Vec<_>>();
        if self.counts.is_empty() {
            output[0].count = self.count;
            return output;
        }
        let fine = 2f64.powi(self.exponent);
        for (offset, count) in self.counts.iter().enumerate().filter(|(_, count)| **count > 0) {
            let lower = ((self.origin + offset as i128) as f64 * fine).max(self.low);
            output[(((lower - self.low) / width) as usize).min(bins - 1)].count += count;
        }
        output
    }

    fn coarsen(&mut self, exponent: i32) {
        let shift = (exponent - self.exponent) as u32;
        let rekey = |index: i128| index.checked_shr(shift).unwrap_or(if index < 0 { -1 } else { 0 });
        // Floor Division by a Power of Two Composes, So Old Bins Rekey Inside the New Span.
        let origin = key(self.low, exponent);
        let mut counts = VecDeque::from(vec![0; (key(self.high, exponent) - origin + 1) as usize]);
        for (offset, count) in self.counts.iter().enumerate().filter(|(_, count)| **count > 0) {
            counts[(rekey(self.origin + offset as i128) - origin) as usize] += count;
        }
        (self.exponent, self.origin, self.counts) = (exponent, origin, counts);
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn finite(value: f64) -> Result<f64> {
    if value.is_finite() { Ok(value) } else { Err(Error::Data("narrow the selection: its aggregate exceeds the finite range")) }
}

/// Integral Exact Results Keep the Column's Integer Kind.
fn integral(value: ExactNumber, kind: Option<Number>) -> ScalarValue {
    if value.denominator.get() == 1 {
        if let Some(Number::Signed(_)) = kind {
            let signed = i128::try_from(value.numerator).ok().and_then(|n| i64::try_from(if value.negative { -n } else { n }).ok());
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

/// The Index of a Value's Bin of Width 2^exponent; Scaling by a Power of Two Is Exact.
fn key(value: f64, exponent: i32) -> i128 {
    (value * 2f64.powi(-exponent)).floor() as i128
}

/// The Narrowest Power-of-Two Width Whose Bins From `low` to `high` Number at Most `resolution` and Have Representable Indices.
fn narrowest(low: f64, high: f64, resolution: usize) -> i32 {
    let span = |exponent: i32| {
        let scale = 2f64.powi(-exponent);
        let (lower, upper) = ((low * scale).floor(), (high * scale).floor());
        let limit = 2f64.powi(KEY_BITS);
        (lower.abs() < limit && upper.abs() < limit).then_some(upper - lower + 1.0)
    };
    let magnitude = low.abs().max(high.abs());
    let representable = if magnitude > 0.0 { (magnitude.log2().ceil() as i32).saturating_sub(KEY_BITS) } else { -1074 };
    let fitted = if high > low { (((high - low) / resolution as f64).log2().floor() as i32).saturating_sub(1) } else { 0 };
    let mut exponent = fitted.max(representable).max(-1074);
    while span(exponent).is_none_or(|bins| bins > resolution as f64) {
        exponent += 1;
    }
    exponent
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Grid, Running};
    use crate::telemetry::aggregates::Values;

    fn values() -> Vec<f64> {
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        (0..5000)
            .map(|index| {
                state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
                match index % 17 {
                    0 => f64::NAN,
                    1 => f64::NEG_INFINITY,
                    2 => 3.0,
                    _ => (state >> 11) as f64 / (1u64 << 53) as f64 * 200.0 - 50.0,
                }
            })
            .collect()
    }

    fn grid(resolution: usize, values: &mut dyn Iterator<Item = f64>) -> Grid {
        let mut grid = Grid::new(resolution);
        values.for_each(|value| grid.push(value));
        grid
    }

    #[test]
    fn histograms_span_the_data_in_the_requested_bins_within_one_fine_width_in_any_order() {
        let values = values();
        let finite = values.iter().copied().filter(|value| value.is_finite()).collect::<Vec<_>>();
        let (low, high) = (finite.iter().copied().fold(f64::INFINITY, f64::min), finite.iter().copied().fold(f64::NEG_INFINITY, f64::max));
        let forward = grid(1024, &mut values.iter().copied());
        let fine = 2f64.powi(forward.exponent);
        assert!((high - low) / fine > 500.0, "Use About Half the Resolution or More");
        for bins in [1, 7, 24, 500] {
            let histogram = forward.bins(bins);
            assert_eq!(histogram, grid(1024, &mut values.iter().rev().copied()).bins(bins));
            assert_eq!(histogram.len(), bins);
            assert_eq!((histogram[0].lower, histogram[bins - 1].upper), (low, high));
            assert_eq!(histogram.iter().map(|bin| bin.count).sum::<u64>(), finite.len() as u64);
            // Cumulative Counts Differ Only by Values Within One Fine Width Above an Edge.
            let mut below = 0;
            for bin in &histogram[..bins - 1] {
                below += bin.count;
                let exact = finite.iter().filter(|value| **value < bin.upper).count() as u64;
                let near = finite.iter().filter(|value| (bin.upper..bin.upper + fine).contains(*value)).count() as u64;
                assert!(exact <= below && below <= exact + near, "{below} Outside [{exact}, {}]", exact + near);
            }
        }
    }

    #[test]
    fn integer_histograms_within_the_resolution_are_exact() {
        let values = (0..3000).map(|index| ((index * 37) % 701) as f64 - 200.0).collect::<Vec<_>>();
        let histogram = grid(1024, &mut values.iter().copied()).bins(13);
        for bin in &histogram {
            let last = std::ptr::eq(bin, histogram.last().unwrap());
            let exact = values.iter().filter(|value| **value >= bin.lower && (**value < bin.upper || last)).count() as u64;
            assert_eq!(bin.count, exact);
        }
        assert_eq!(grid(8, &mut [5.0, 5.0, 5.0].into_iter()).bins(4)[0].count, 3);
        // A Ramp Widens Its Extremes With Every Value Yet Matches the Same Values Reversed.
        let ramp = (0..20_000).map(|index| index as f64 * 0.37).collect::<Vec<_>>();
        assert_eq!(grid(4096, &mut ramp.iter().copied()), grid(4096, &mut ramp.iter().rev().copied()));
    }

    #[test]
    fn running_statistics_across_chunks_equal_one_pass() {
        let values = values();
        let valid = |row: usize| row % 5 != 4;
        let mut whole = Running::default();
        whole.extend(&Values::Float(&values), valid);
        let mut chunked = Running::default();
        for (start, end) in [(0, 1), (1, 1700), (1700, 1700), (1700, 5000)] {
            chunked.extend(&Values::Float(&values[start..end]), |row| valid(start + row));
        }
        assert_eq!(chunked, whole);
        assert_eq!(chunked.finish().unwrap(), whole.finish().unwrap());
        let expected = (0..values.len()).filter(|row| valid(*row) && values[*row].is_finite()).count();
        assert_eq!(whole.finish().unwrap().count, expected as u64);
    }
}
