/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Plot Kind Reductions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::scalar::{self, ScalarValue as Value};
use crate::{Error, NumericWindow, Result, Store};
use dokime_protocol::v1;
use std::collections::BTreeMap;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type CartesianRead = (dokime_protocol::client::plots::PlotFrame, u64, Vec<(String, u64)>);

/* === Structs === */

pub struct KindWindow<'a> {
    pub window: NumericWindow<'a>,
    pub x: Option<&'a str>,
    pub z: Option<&'a str>,
    pub group: Option<&'a str>,
    pub limit: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn cartesian_window(&self, request: KindWindow<'_>) -> Result<CartesianRead> {
        let window = &request.window;
        let width = ((i128::from(window.end_ns) - i128::from(window.start_ns) + 1 + window.bins.get() as i128 - 1)
            / window.bins.get() as i128)
            .max(1);
        let mut kept = BTreeMap::new();
        let mut raw_counts = BTreeMap::<String, u64>::new();
        let mut count = 0u64;
        self.kind_values(&request, |row| {
            if missing(&row.x) {
                return Ok(());
            }
            count += 1;
            *raw_counts.entry(text(row.g.clone())).or_default() += 1;
            let key = ((i128::from(row.time) - i128::from(window.start_ns)) / width, text(row.g.clone()));
            if !kept.contains_key(&key) && kept.len() >= request.limit {
                return Err(Error::Data("narrow the cartesian window or reduce groups"));
            }
            if kept.get(&key).is_none_or(|old: &Input| old.position < row.position) {
                kept.insert(key, row);
            }
            Ok(())
        })?;
        let mut rows = kept.into_values().collect::<Vec<_>>();
        rows.sort_by_key(|row| row.position);
        let rows = rows.into_iter().map(|r| Ok::<_, Error>((r.time, r.x, r.y, scalar::number(&r.z), r.g)));
        let mut columns: BTreeMap<String, dokime_protocol::client::plots::PlotColumn> = BTreeMap::new();
        let mut points = 0;
        for row in rows {
            let (time, x, y, z, group) = row?;
            points += 1;
            if points > request.limit {
                return Err(Error::Invalid("Read Cartesian Plot: Narrow the Window or Reduce the Number of Groups".into()));
            }
            let column = columns.entry(text(group)).or_default();
            column.instants.push(instant(time));
            column.x.push(value(x));
            column.y.push(dokime_protocol::client::plots::NullableNumber { value: Some(y) });
            column.z.push(dokime_protocol::client::plots::NullableNumber { value: z });
        }
        let keys = columns.keys().map(|key| (key.clone(), raw_counts.get(key).copied().unwrap_or(0))).collect();
        Ok((
            dokime_protocol::client::plots::PlotFrame {
                mode: Some(dokime_protocol::client::plots::PlotCartesianMode::Faceted as i32),
                series: columns.into_values().collect(),
                point_count: points as u64,
                ..Default::default()
            },
            count as u64,
            keys,
        ))
    }
    pub fn category_window(&self, request: KindWindow<'_>, aggregate: Option<v1::Aggregate>, unique: bool) -> Result<CartesianRead> {
        let mut grouped = BTreeMap::<(String, String), (Input, u64, f64, f64, f64)>::new();
        self.kind_values(&request, |row| {
            if missing(&row.x) {
                return Ok(());
            }
            let key = (serde_json::to_string(&row.x)?, serde_json::to_string(&row.g)?);
            if !grouped.contains_key(&key) && grouped.len() >= request.limit {
                return Err(Error::Data("narrow categories or select fewer groups"));
            }
            let entry = grouped.entry(key).or_insert((row.clone(), 0, 0.0, row.y, row.y));
            entry.1 += 1;
            entry.2 += row.y;
            entry.3 = entry.3.min(row.y);
            entry.4 = entry.4.max(row.y);
            if row.position > entry.0.position {
                entry.0 = row;
            }
            Ok(())
        })?;
        let rows = grouped.into_values().map(|(row, n, sum, min, max)| {
            let y = match aggregate {
                Some(v1::Aggregate::Count) => n as f64,
                Some(v1::Aggregate::Sum) => sum,
                Some(v1::Aggregate::Mean) => sum / n as f64,
                Some(v1::Aggregate::Min) => min,
                Some(v1::Aggregate::Max) => max,
                _ => row.y,
            };
            Ok::<_, Error>((row.x, y, row.g, n as i64))
        });
        let mut columns: BTreeMap<String, dokime_protocol::client::plots::PlotColumn> = BTreeMap::new();
        let mut points = 0;
        let mut count = 0;
        let mut raw_counts = BTreeMap::<String, u64>::new();
        for row in rows {
            let (x, y, group, n) = row?;
            points += 1;
            count += n as u64;
            if points > request.limit {
                return Err(Error::Invalid("Read Category Plot: Narrow the Window or Aggregate Fewer Categories".into()));
            }
            if unique && aggregate.is_none() && n > 1 {
                return Err(Error::Invalid("Read Pie or Pareto Plot: Aggregate Duplicate Category Totals Explicitly".into()));
            }
            if unique && (!y.is_finite() || y < 0.0) {
                return Err(Error::Invalid("Read Pie or Pareto Plot: Supply Finite Nonnegative Category Totals".into()));
            }
            let group = text(group);
            *raw_counts.entry(group.clone()).or_default() += n as u64;
            let column = columns.entry(group).or_default();
            column.x.push(value(x));
            column.y.push(dokime_protocol::client::plots::NullableNumber { value: Some(y) });
        }
        let keys = columns.keys().map(|key| (key.clone(), raw_counts.get(key).copied().unwrap_or(0))).collect();
        Ok((
            dokime_protocol::client::plots::PlotFrame {
                mode: Some(dokime_protocol::client::plots::PlotCartesianMode::Faceted as i32),
                series: columns.into_values().collect(),
                point_count: points as u64,
                ..Default::default()
            },
            count,
            keys,
        ))
    }
    pub fn sigma_window(
        &self, request: KindWindow<'_>, options: v1::PlotIntervalOptions,
    ) -> Result<(dokime_protocol::client::plots::PlotSigmaIntervalAnalysis, u64)> {
        if !options.bin_width.is_finite() || options.bin_width <= 0.0 || !options.sigma.is_finite() || options.sigma <= 0.0 {
            return Err(Error::Invalid("Query XY Sigma: Supply Positive Finite Bin Width and Sigma".into()));
        }
        let width = options.bin_width;
        let mut groups = BTreeMap::<(String, i64), (Value, u64, f64, f64, f64, f64)>::new();
        self.kind_values(&request, |row| {
            let Some(x) = scalar::number(&row.x) else {
                return Ok(());
            };
            let bin = (x / width).floor() as i64;
            let key = (text(row.g.clone()), bin);
            if !groups.contains_key(&key) && groups.len() >= request.limit {
                return Err(Error::Data("increase sigma bin width or narrow the window"));
            }
            let entry = groups.entry(key).or_insert((row.g, 0, 0.0, 0.0, row.y, row.y));
            entry.1 += 1;
            let delta = row.y - entry.2;
            entry.2 += delta / entry.1 as f64;
            entry.3 += delta * (row.y - entry.2);
            entry.4 = entry.4.min(row.y);
            entry.5 = entry.5.max(row.y);
            Ok(())
        })?;
        let reduced = groups
            .into_iter()
            .map(|((_, bin), (g, n, mean, m2, min, max))| Ok::<_, Error>((bin, g, n as i64, mean, m2 / n as f64, min, max)));
        let mut series = BTreeMap::<String, dokime_protocol::client::plots::PlotSigmaSeries>::new();
        let mut count = 0;
        let mut points = 0;
        for row in reduced {
            let (bin, group, n, mean, variance, min, max) = row?;
            points += 1;
            if points > request.limit {
                return Err(Error::Invalid("Query XY Sigma: Increase Bin Width or Narrow the Window".into()));
            }
            let spread = options.sigma * variance.max(0.0).sqrt();
            let mut lower = mean - spread;
            let mut upper = mean + spread;
            if options.clip_to_observed {
                lower = lower.max(min);
                upper = upper.min(max);
            }
            let group = text(group);
            let output = series.entry(group.clone()).or_insert_with(|| dokime_protocol::client::plots::PlotSigmaSeries {
                key: if group.is_empty() { request.window.field.into() } else { format!("{}/{group}", request.window.field) },
                ..Default::default()
            });
            output.bins.push(dokime_protocol::client::plots::PlotSigmaBin {
                x: (bin as f64 + 0.5) * width,
                count: n as u64,
                mean,
                lower,
                upper,
            });
            output.raw_points += n as u64;
            count += n as u64;
        }
        Ok((dokime_protocol::client::plots::PlotSigmaIntervalAnalysis { series: series.into_values().collect() }, count))
    }
    pub fn contour_window(
        &self, request: KindWindow<'_>, x_view: Option<&v1::PlotWindow>, y_view: Option<&v1::PlotWindow>, reduction: v1::Reduction,
    ) -> Result<(dokime_protocol::client::plots::ContourGrid, u64)> {
        self.contour_scaled_window(request, x_view, y_view, reduction, false, false)
    }
    pub fn contour_scaled_window(
        &self, request: KindWindow<'_>, x_view: Option<&v1::PlotWindow>, y_view: Option<&v1::PlotWindow>, reduction: v1::Reduction,
        x_log: bool, y_log: bool,
    ) -> Result<(dokime_protocol::client::plots::ContourGrid, u64)> {
        let mut bounds = (None::<f64>, None::<f64>, None::<f64>, None::<f64>);
        self.kind_values(&request, |row| {
            if let (Some(x), Some(_)) = (scalar::number(&row.x), scalar::number(&row.z)) {
                if (!x_log || x > 0.0) && (!y_log || row.y > 0.0) {
                    bounds.0 = Some(bounds.0.map_or(x, |v| v.min(x)));
                    bounds.1 = Some(bounds.1.map_or(x, |v| v.max(x)));
                    bounds.2 = Some(bounds.2.map_or(row.y, |v| v.min(row.y)));
                    bounds.3 = Some(bounds.3.map_or(row.y, |v| v.max(row.y)));
                }
            }
            Ok(())
        })?;
        let (Some(x_min), Some(x_max), Some(y_min), Some(y_max)) = bounds else {
            return Ok((dokime_protocol::client::plots::ContourGrid::default(), 0));
        };
        let (x_min, mut x_max) = x_view.map_or((x_min, x_max), |view| (view.from, view.to));
        let (y_min, mut y_max) = y_view.map_or((y_min, y_max), |view| (view.from, view.to));
        if x_min == x_max {
            x_max = if x_log { x_min * std::f64::consts::E } else { x_min + 1.0 };
        }
        if y_min == y_max {
            y_max = if y_log { y_min * std::f64::consts::E } else { y_min + 1.0 };
        }
        if !x_min.is_finite()
            || !x_max.is_finite()
            || !y_min.is_finite()
            || !y_max.is_finite()
            || x_min >= x_max
            || y_min >= y_max
            || x_log && x_min <= 0.0
            || y_log && y_min <= 0.0
        {
            return Err(Error::Invalid("Query Contour: Select Finite Increasing Bounds and Positive Logarithmic Axes".into()));
        }
        let columns = request.window.bins.get().min(request.limit).max(1);
        let rows = (request.limit / columns).max(1);
        let x_start = if x_log { x_min.ln() } else { x_min };
        let x_end = if x_log { x_max.ln() } else { x_max };
        let y_start = if y_log { y_min.ln() } else { y_min };
        let y_end = if y_log { y_max.ln() } else { y_max };
        let width = (x_end - x_start) / columns as f64;
        let height = (y_end - y_start) / rows as f64;
        let mut grid = dokime_protocol::client::plots::ContourGrid {
            columns: columns as u32,
            rows: rows as u32,
            x_edges: (0..=columns)
                .map(|index| {
                    let value = x_start + index as f64 * width;
                    if x_log { value.exp() } else { value }
                })
                .collect(),
            y_edges: (0..=rows)
                .map(|index| {
                    let value = y_start + index as f64 * height;
                    if y_log { value.exp() } else { value }
                })
                .collect(),
            values: vec![0.0; rows * columns],
            valid: vec![false; rows * columns],
        };
        let mut counts = vec![0_u64; rows * columns];
        let mut count = 0;
        self.kind_values(&request, |input| {
            let (Some(x), Some(z)) = (scalar::number(&input.x), scalar::number(&input.z)) else {
                return Ok(());
            };
            let y = input.y;
            if !x.is_finite()
                || !y.is_finite()
                || !z.is_finite()
                || x < x_min
                || x > x_max
                || y < y_min
                || y > y_max
                || x_log && x <= 0.0
                || y_log && y <= 0.0
            {
                return Ok(());
            }
            let x = if x_log { x.ln() } else { x };
            let y = if y_log { y.ln() } else { y };
            let column = (((x - x_start) / width) as usize).min(columns - 1);
            let row = (((y - y_start) / height) as usize).min(rows - 1);
            let index = row * columns + column;
            counts[index] += 1;
            count += 1;
            grid.values[index] = if !grid.valid[index] {
                z
            } else {
                match reduction {
                    v1::Reduction::Min => grid.values[index].min(z),
                    v1::Reduction::Max => grid.values[index].max(z),
                    _ => grid.values[index] + (z - grid.values[index]) / counts[index] as f64,
                }
            };
            grid.valid[index] = true;
            Ok(())
        })?;
        Ok((grid, count))
    }

    pub fn grid_window(
        &self, request: KindWindow<'_>, definition: &v1::GridDefinition, reduction: v1::Reduction,
    ) -> Result<(dokime_protocol::client::plots::GridFrames, u64)> {
        let mut grouped = BTreeMap::<(String, String, String), (Value, Value, Value, f64, u64)>::new();
        self.kind_values(&request, |row| {
            if missing(&row.x) || missing(&row.z) {
                return Ok(());
            }
            let frame = if request.group.is_none() { Value::Signed(row.time) } else { row.g };
            let key = (text(row.x.clone()), text(row.z.clone()), text(frame.clone()));
            if !grouped.contains_key(&key) && grouped.len() >= request.limit {
                return Err(Error::Data("narrow grid frames and cells"));
            }
            let entry = grouped.entry(key).or_insert((row.x, row.z, frame, row.y, 0));
            entry.4 += 1;
            entry.3 = match reduction {
                v1::Reduction::Min => entry.3.min(row.y),
                v1::Reduction::Max => entry.3.max(row.y),
                _ => entry.3 + (row.y - entry.3) / entry.4 as f64,
            };
            Ok(())
        })?;
        let records = grouped.into_values().collect::<Vec<_>>();
        if records.len() > request.limit {
            return Err(Error::Invalid("Read Grid Chart: Narrow the Window or Reduce the Number of Frames and Cells".into()));
        }
        let axes = |declared: &[v1::GridAxisEntry], index: usize| -> Vec<v1::GridAxisEntry> {
            if !declared.is_empty() {
                let mut values = declared.to_vec();
                values.sort_by_key(|entry| entry.order);
                return values;
            }
            let values = records
                .iter()
                .map(|record| text(if index == 0 { record.0.clone() } else { record.1.clone() }))
                .collect::<std::collections::BTreeSet<_>>();
            values
                .into_iter()
                .enumerate()
                .map(|(index, key)| v1::GridAxisEntry {
                    label: key.clone(),
                    key,
                    order: index as i32,
                    ..Default::default()
                })
                .collect()
        };
        let rows = axes(&definition.rows, 0);
        let columns = axes(&definition.columns, 1);
        let frame_count = records.iter().map(|record| text(record.2.clone())).collect::<std::collections::BTreeSet<_>>().len();
        if rows.len().saturating_mul(columns.len()).saturating_mul(frame_count) > request.limit {
            return Err(Error::Invalid(
                "Read Grid Chart: Narrow the Window or Reduce Declared Cells and Frames to Fit the Resolution".into(),
            ));
        }
        let mut frames: BTreeMap<String, dokime_protocol::client::plots::GridFrame> = BTreeMap::new();
        let mut count = 0;
        for (row, column, frame, value, n) in records {
            let row = text(row);
            let column = text(column);
            let row = rows
                .iter()
                .position(|axis| axis.key == row)
                .ok_or_else(|| Error::Invalid("Read Grid Chart: Declare Every Observed Row".into()))?;
            let column = columns
                .iter()
                .position(|axis| axis.key == column)
                .ok_or_else(|| Error::Invalid("Read Grid Chart: Declare Every Observed Column".into()))?;
            let frame = match frame {
                Value::Signed(time) if request.group.is_none() => instant(time),
                value => text(value),
            };
            let output = frames.entry(frame.clone()).or_insert_with(|| dokime_protocol::client::plots::GridFrame {
                instant: frame,
                values: vec![0.0; rows.len() * columns.len()],
                valid: vec![false; rows.len() * columns.len()],
                ..Default::default()
            });
            let index = row * columns.len() + column;
            output.values[index] = value;
            output.valid[index] = true;
            count += n as u64;
        }
        for highlight in &definition.highlights {
            let field = |expression: Option<&v1::DataExpression>| -> Result<String> {
                match expression.and_then(|expression| expression.kind.as_ref()) {
                    Some(v1::data_expression::Kind::Stream(reference)) if reference.identifier == request.window.stream => {
                        Ok(reference.field.clone())
                    }
                    _ => Err(Error::Invalid("Query Grid Highlight: Bind Row and Column Expressions to the Selected Stream".into())),
                }
            };
            let row = field(highlight.row.as_ref())?;
            let column = field(highlight.column.as_ref())?;
            let binding = KindWindow {
                window: request.window,
                x: Some(&row),
                z: Some(&column),
                group: request.group,
                limit: request.limit,
            };
            let mut highlights = BTreeMap::<String, Input>::new();
            self.kind_values(&binding, |row| {
                let key = if request.group.is_none() { instant(row.time) } else { text(row.g.clone()) };
                if !highlights.contains_key(&key) && highlights.len() >= request.limit {
                    return Err(Error::Data("narrow grid highlights"));
                }
                if highlights.get(&key).is_none_or(|old| old.position < row.position) {
                    highlights.insert(key, row);
                }
                Ok(())
            })?;
            for (_, input) in highlights {
                let (row, column, frame) = (input.x, input.z, if request.group.is_none() { Value::Signed(input.time) } else { input.g });
                let frame = match frame {
                    Value::Signed(time) if request.group.is_none() => instant(time),
                    value => text(value),
                };
                let row = rows.iter().position(|axis| axis.key == text(row.clone()));
                let column = columns.iter().position(|axis| axis.key == text(column.clone()));
                if let (Some(row), Some(column), Some(frame)) = (row, column, frames.get_mut(&frame)) {
                    frame.highlights.push(dokime_protocol::client::plots::GridCell {
                        row: row as u32,
                        column: column as u32,
                        label: highlight.label.clone(),
                    });
                }
            }
        }
        Ok((
            dokime_protocol::client::plots::GridFrames {
                rows,
                columns,
                frames: frames.into_values().collect(),
                summary_label: definition.summary_label.clone(),
            },
            count,
        ))
    }
    pub fn spectrum_window(
        &self, request: KindWindow<'_>, options: &v1::PlotFftOptions,
    ) -> Result<(dokime_protocol::client::plots::PlotColumn, u64)> {
        let window = &request.window;
        let mut rows = Vec::new();
        self.numeric_values(*window, |_, time, value| {
            if rows.len() > request.limit {
                return Err(Error::Data("narrow the frequency source window to its configured input bound"));
            }
            rows.push((time, value));
            Ok(())
        })?;
        rows.sort_by_key(|row| row.0);
        if rows.len() > request.limit {
            return Err(Error::Invalid(
                "Read Frequency Plot: Narrow the Source-Time Window to the Configured Transform Input Bound".into(),
            ));
        }
        if rows.len() < 2 {
            return Ok((dokime_protocol::client::plots::PlotColumn::default(), rows.len() as u64));
        }
        let sample_rate = options.sample_rate_hz.unwrap_or(1e9 / (rows[1].0 - rows[0].0) as f64);
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            return Err(Error::Invalid("Read Frequency Plot: Supply a Positive Sample Rate and Distinct Sample Times".into()));
        }
        let count = rows.len();
        let mean = if options.detrend == v1::FftDetrend::None as i32 {
            0.0
        } else {
            rows.iter().map(|(_, value)| value).sum::<f64>() / count as f64
        };
        let weighted = rows
            .iter()
            .enumerate()
            .map(|(index, (_, value))| {
                let weight = if options.window == v1::FftWindow::Rectangular as i32 {
                    1.0
                } else {
                    0.5 * (1.0 - (std::f64::consts::TAU * index as f64 / (count - 1) as f64).cos())
                };
                ((*value - mean) * weight, weight)
            })
            .collect::<Vec<_>>();
        let gain = weighted.iter().map(|(_, weight)| weight).sum::<f64>();
        if gain <= 0.0 {
            return Err(Error::Invalid("Read Frequency Plot: Select a Rectangular Window or Include More Samples".into()));
        }
        let mut column = dokime_protocol::client::plots::PlotColumn::default();
        for frequency in 0..=count / 2 {
            if frequency == 0 && !options.include_dc {
                continue;
            }
            let hz = frequency as f64 * sample_rate / count as f64;
            if options.min_frequency_hz.is_some_and(|min| hz < min) || options.max_frequency_hz.is_some_and(|max| hz > max) {
                continue;
            }
            let (mut real, mut imag) = (0.0, 0.0);
            for (index, (value, _)) in weighted.iter().enumerate() {
                let angle = std::f64::consts::TAU * frequency as f64 * index as f64 / count as f64;
                real += value * angle.cos();
                imag -= value * angle.sin();
            }
            let scale = if frequency == 0 || frequency == count / 2 && count.is_multiple_of(2) { 1.0 } else { 2.0 };
            let amplitude = real.hypot(imag) * scale / gain;
            let level = match v1::FrequencyLevelUnit::try_from(options.level_unit).unwrap_or(v1::FrequencyLevelUnit::Linear) {
                v1::FrequencyLevelUnit::Db => 20.0 * amplitude.max(f64::MIN_POSITIVE).log10(),
                v1::FrequencyLevelUnit::Dbu => 20.0 * (amplitude / std::f64::consts::SQRT_2 / 0.775).max(f64::MIN_POSITIVE).log10(),
                v1::FrequencyLevelUnit::Dbm => 10.0 * (amplitude * amplitude / 100.0 / 0.001).max(f64::MIN_POSITIVE).log10(),
                _ => amplitude,
            };
            column.x.push(value(Value::Float(hz)));
            column.y.push(dokime_protocol::client::plots::NullableNumber { value: Some(level) });
        }
        if column.y.len() > window.bins.get() {
            let width = column.y.len().div_ceil(window.bins.get());
            let mut reduced = dokime_protocol::client::plots::PlotColumn::default();
            for (index, chunk) in column.y.chunks(width).enumerate() {
                let peak = chunk
                    .iter()
                    .enumerate()
                    .max_by(|(_, left), (_, right)| left.value.unwrap_or_default().total_cmp(&right.value.unwrap_or_default()))
                    .unwrap()
                    .0;
                reduced.x.push(column.x[index * width + peak].clone());
                reduced.y.push(chunk[peak]);
            }
            column = reduced;
        }
        Ok((column, count as u64))
    }
}

#[derive(Clone)]
struct Input {
    position: u64,
    time: i64,
    x: Value,
    y: f64,
    z: Value,
    g: Value,
}
impl Store {
    fn kind_values(&self, request: &KindWindow<'_>, mut read: impl FnMut(Input) -> Result<()>) -> Result<()> {
        use arrow_array::{Int64Array, StructArray, UInt64Array};
        let fields = std::iter::once(request.window.field)
            .chain(request.x)
            .chain(request.z)
            .chain(request.group)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        self.evaluated_batches(request.window, &fields, |batch| {
            let metadata = batch.column(0).as_any().downcast_ref::<StructArray>().unwrap();
            let positions = metadata.column_by_name("position").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap();
            let times = metadata.column_by_name("source_ns").unwrap().as_any().downcast_ref::<Int64Array>().unwrap();
            for row in 0..batch.num_rows() {
                let Some(y) = scalar::number(&scalar::field(&batch, request.window.field, row)?).filter(|v| v.is_finite()) else {
                    continue;
                };
                let field = |field: Option<&str>| {
                    field.map(|field| scalar::field(&batch, field, row)).transpose().map(|v| v.unwrap_or(Value::Null))
                };
                read(Input {
                    position: positions.value(row),
                    time: times.value(row),
                    x: field(request.x)?,
                    y,
                    z: field(request.z)?,
                    g: request.group.map(|f| scalar::field(&batch, f, row)).transpose()?.unwrap_or(Value::Text(String::new())),
                })?;
            }
            Ok(true)
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn value(value: Value) -> v1::Value {
    scalar::wire(&value).expect("finite plot scalar")
}
fn text(value: Value) -> String {
    scalar::text(&value)
}
fn missing(value: &Value) -> bool {
    matches!(value, Value::Missing | Value::Null)
}
fn instant(at_ns: i64) -> String {
    prost_types::Timestamp {
        seconds: at_ns.div_euclid(1_000_000_000),
        nanos: at_ns.rem_euclid(1_000_000_000) as i32,
    }
    .to_string()
}

/* === Tests === */
