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
use crate::{Error, NumericWindow, Result, Store};
use dokime_protocol::v1;
use rusqlite::{params, types::Value};
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
        let connection = self.lock()?;
        let width = ((i128::from(window.end_ns) - i128::from(window.start_ns) + 1 + window.bins.get() as i128 - 1)
            / window.bins.get() as i128)
            .max(1)
            .min(i64::MAX as i128) as i64;
        let input = input();
        let count: i64 = connection.query_row(
            &format!("{input} SELECT COUNT(*) FROM input WHERE x IS NOT NULL AND y IS NOT NULL"),
            bind(&request),
            |row| row.get(0),
        )?;
        let query = format!(
            "{input}, kept AS (SELECT MAX(position) AS position FROM input WHERE x IS NOT NULL AND y IS NOT NULL GROUP BY (t-?4)/{width},g) SELECT t,x,y,z,g FROM input JOIN kept USING(position) ORDER BY position LIMIT {}",
            request.limit + 1
        );
        let mut statement = connection.prepare(&query)?;
        let rows = statement.query_map(bind(&request), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Value>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, Option<f64>>(3)?,
                row.get::<_, Value>(4)?,
            ))
        })?;
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
        let mut raw_counts = BTreeMap::new();
        for row in connection
            .prepare(&format!(
                "{input} SELECT g,COUNT(*) FROM input WHERE x IS NOT NULL AND y IS NOT NULL GROUP BY g LIMIT {}",
                request.limit + 1
            ))?
            .query_map(bind(&request), |row| Ok((row.get::<_, Value>(0)?, row.get::<_, i64>(1)?)))?
        {
            let (group, count) = row?;
            raw_counts.insert(text(group), count as u64);
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
        let connection = self.lock()?;
        let input = input();
        let reducer = match aggregate {
            Some(v1::Aggregate::Count) => "COUNT(*)",
            Some(v1::Aggregate::Sum) => "SUM(y)",
            Some(v1::Aggregate::Mean) => "AVG(y)",
            Some(v1::Aggregate::Min) => "MIN(y)",
            Some(v1::Aggregate::Max) => "MAX(y)",
            _ => "MAX(position)",
        };
        let expression = if aggregate.is_some() { "reduced" } else { "input.y" };
        let query = format!(
            "{input}, groups AS (SELECT x,g,COUNT(*) AS n,{reducer} AS reduced,MAX(position) AS last FROM input WHERE x IS NOT NULL AND y IS NOT NULL GROUP BY x,g) SELECT groups.x,{expression},groups.g,n FROM groups JOIN input ON input.position=groups.last ORDER BY groups.last LIMIT {}",
            request.limit + 1
        );
        let mut statement = connection.prepare(&query)?;
        let rows = statement.query_map(bind(&request), |row| {
            Ok((row.get::<_, Value>(0)?, row.get::<_, f64>(1)?, row.get::<_, Value>(2)?, row.get::<_, i64>(3)?))
        })?;
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
        let connection = self.lock()?;
        let input = input();
        let width = options.bin_width;
        let ratio = format!("x/{width}");
        let bin = format!("CAST({ratio} AS INTEGER)-CASE WHEN ({ratio})<CAST({ratio} AS INTEGER) THEN 1 ELSE 0 END");
        let query = format!(
            "{input}, bucketed AS(SELECT *,{bin} AS b FROM input WHERE typeof(x) IN ('real','integer') AND y IS NOT NULL), means AS(SELECT b,g,AVG(y) AS mean FROM bucketed GROUP BY b,g) SELECT b,g,COUNT(*),mean,AVG((y-mean)*(y-mean)),MIN(y),MAX(y) FROM bucketed JOIN means USING(b,g) GROUP BY b,g ORDER BY g,b LIMIT {}",
            request.limit + 1
        );
        let mut statement = connection.prepare(&query)?;
        let mut series = BTreeMap::<String, dokime_protocol::client::plots::PlotSigmaSeries>::new();
        let mut count = 0;
        let mut points = 0;
        for row in statement.query_map(bind(&request), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Value>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, f64>(6)?,
            ))
        })? {
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
        let connection = self.lock()?;
        let input = input();
        let valid = format!(
            "typeof(x) IN ('real','integer') AND y IS NOT NULL AND z IS NOT NULL{}{}",
            if x_log { " AND x>0" } else { "" },
            if y_log { " AND y>0" } else { "" }
        );
        let bounds = connection.query_row(
            &format!("{input} SELECT MIN(x),MAX(x),MIN(y),MAX(y) FROM input WHERE {valid}"),
            bind(&request),
            |row| {
                Ok((
                    row.get::<_, Option<f64>>(0)?,
                    row.get::<_, Option<f64>>(1)?,
                    row.get::<_, Option<f64>>(2)?,
                    row.get::<_, Option<f64>>(3)?,
                ))
            },
        )?;
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
        let mut query = connection.prepare(&format!(
            "{input} SELECT x,y,z FROM input WHERE {valid} AND x>={x_min} AND x<={x_max} AND y>={y_min} AND y<={y_max}"
        ))?;
        for row in query.query_map(bind(&request), |row| Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?, row.get::<_, f64>(2)?)))? {
            let (x, y, z) = row?;
            if !x.is_finite() || !y.is_finite() || !z.is_finite() {
                continue;
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
        }
        Ok((grid, count))
    }

    pub fn grid_window(
        &self, request: KindWindow<'_>, definition: &v1::GridDefinition, reduction: v1::Reduction,
    ) -> Result<(dokime_protocol::client::plots::GridFrames, u64)> {
        let connection = self.lock()?;
        let input = input();
        let reduce = match reduction {
            v1::Reduction::Min => "MIN(y)",
            v1::Reduction::Max => "MAX(y)",
            _ => "AVG(y)",
        };
        let query = format!(
            "{input} SELECT x,z,CASE WHEN ?10 IS NULL THEN t ELSE g END AS frame,{reduce},COUNT(*) FROM input WHERE x IS NOT NULL AND z IS NOT NULL AND y IS NOT NULL GROUP BY x,z,frame ORDER BY frame,x,z LIMIT {}",
            request.limit + 1
        );
        let mut statement = connection.prepare(&query)?;
        let records = statement
            .query_map(bind(&request), |row| {
                Ok((
                    row.get::<_, Value>(0)?,
                    row.get::<_, Value>(1)?,
                    row.get::<_, Value>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
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
                Value::Integer(time) if request.group.is_none() => instant(time),
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
            let query = format!(
                "{} SELECT x,z,CASE WHEN ?10 IS NULL THEN t ELSE g END AS frame,MAX(position) FROM input WHERE x IS NOT NULL AND z IS NOT NULL GROUP BY frame ORDER BY frame LIMIT {}",
                input,
                request.limit + 1
            );
            let mut statement = connection.prepare(&query)?;
            for value in statement
                .query_map(bind(&binding), |row| Ok((row.get::<_, Value>(0)?, row.get::<_, Value>(1)?, row.get::<_, Value>(2)?)))?
            {
                let (row, column, frame) = value?;
                let frame = match frame {
                    Value::Integer(time) if request.group.is_none() => instant(time),
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
        let connection = self.lock()?;
        let window = &request.window;
        let mut statement=connection.prepare("SELECT source_ns,number FROM sample_fields WHERE execution=?1 AND stream=?2 AND field=?3 AND source_ns>=?4 AND source_ns<=?5 AND position>?6 AND position<=?7 AND number IS NOT NULL ORDER BY source_ns,position LIMIT ?8")?;
        let rows = statement
            .query_map(
                params![
                    window.execution,
                    window.stream,
                    window.field,
                    window.start_ns,
                    window.end_ns,
                    window.after as i64,
                    window.through as i64,
                    request.limit as i64 + 1
                ],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?)),
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
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
            column.x.push(value(Value::Real(hz)));
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

/* === Public Functions === */

/* === Private Functions === */

fn input() -> &'static str {
    "WITH input AS (SELECT y.position,y.source_ns AS t,COALESCE(x.number,x.label) AS x,y.number AS y,COALESCE(z.number,z.label) AS z,COALESCE(g.label,g.number,'') AS g FROM sample_fields AS y LEFT JOIN sample_fields x ON x.execution=y.execution AND x.stream=y.stream AND x.position=y.position AND x.field=?8 LEFT JOIN sample_fields z ON z.execution=y.execution AND z.stream=y.stream AND z.position=y.position AND z.field=?9 LEFT JOIN sample_fields g ON g.execution=y.execution AND g.stream=y.stream AND g.position=y.position AND g.field=?10 WHERE y.execution=?1 AND y.stream=?2 AND y.field=?3 AND y.source_ns>=?4 AND y.source_ns<=?5 AND y.position>?6 AND y.position<=?7)"
}
fn bind(request: &KindWindow<'_>) -> rusqlite::ParamsFromIter<Vec<Value>> {
    let window = &request.window;
    let optional = |value: Option<&str>| value.map(|value| Value::Text(value.into())).unwrap_or(Value::Null);
    rusqlite::params_from_iter(vec![
        Value::Text(window.execution.into()),
        Value::Text(window.stream.into()),
        Value::Text(window.field.into()),
        Value::Integer(window.start_ns),
        Value::Integer(window.end_ns),
        Value::Integer(window.after as i64),
        Value::Integer(window.through as i64),
        optional(request.x),
        optional(request.z),
        optional(request.group),
    ])
}
fn value(value: Value) -> v1::Value {
    let kind = match value {
        Value::Real(value) => v1::value::Kind::Number(value),
        Value::Integer(value) => v1::value::Kind::Integer(value),
        Value::Text(value) => v1::value::Kind::Text(value),
        _ => v1::value::Kind::Null(v1::Null {}),
    };
    v1::Value { kind: Some(kind) }
}
fn text(value: Value) -> String {
    match value {
        Value::Text(value) => value,
        Value::Integer(value) => value.to_string(),
        Value::Real(value) => value.to_string(),
        _ => String::new(),
    }
}
fn instant(at_ns: i64) -> String {
    prost_types::Timestamp {
        seconds: at_ns.div_euclid(1_000_000_000),
        nanos: at_ns.rem_euclid(1_000_000_000) as i32,
    }
    .to_string()
}

/* === Tests === */
