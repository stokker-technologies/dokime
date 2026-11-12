/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Fixed Snapshot Document Rendering

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Write;

// Third Party Imports

// Local Imports
use crate::{Error, Result};
use dokime_protocol::v1;

/* === Constants === */

const COLORS: [&str; 6] = ["#4477aa", "#ee6677", "#228833", "#ccbb44", "#66ccee", "#aa3377"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn render(
    output: &mut impl Write, plot: &v1::Plot, envelope: &v1::PlotEnvelope, presentation: &v1::PlotPresentation,
) -> Result<()> {
    let payload = envelope.payload.as_ref().ok_or_else(|| Error::Invalid("Render Plot: Query Its Fixed Snapshot First".into()))?;
    let count = envelope.diagnostics.as_ref().map_or(0, |diagnostics| diagnostics.raw_point_count);
    write!(
        output,
        "<figure><figcaption>{}</figcaption><svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 640 240\" role=\"img\" data-stream=\"{}\" data-count=\"{count}\">",
        escape(plot.metadata.as_ref().map_or("", |metadata| metadata.label.as_str())),
        escape(&payload.stream)
    )?;
    render_references(output, payload, presentation)?;
    if let Some(frame) = &payload.frame {
        render_frame(output, plot, frame)?;
    }
    if let Some(analysis) = payload.analysis.as_ref().and_then(|analysis| analysis.result.as_ref()).and_then(|result| result.kind.as_ref())
    {
        match analysis {
            v1::plot_analysis_result::Kind::Histogram(histogram) => {
                let max = histogram.series.iter().flat_map(|series| &series.bins).map(|bin| bin.count).max().unwrap_or(1).max(1) as f64;
                let total = histogram.series.iter().map(|series| series.bins.len()).sum::<usize>().max(1);
                let mut index = 0;
                for (series_index, series) in histogram.series.iter().enumerate() {
                    for bin in &series.bins {
                        let width = 600.0 / total as f64;
                        let height = 200.0 * bin.count as f64 / max;
                        write!(
                            output,
                            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{height}\" fill=\"{}\"><title>{}–{}: {}</title></rect>",
                            20.0 + index as f64 * width,
                            220.0 - height,
                            width,
                            COLORS[series_index % COLORS.len()],
                            bin.lower,
                            bin.upper,
                            bin.count
                        )?;
                        index += 1;
                    }
                }
            }
            v1::plot_analysis_result::Kind::BoxPlot(boxes) => {
                let min = boxes.series.iter().map(|series| series.min).reduce(f64::min).unwrap_or(0.0);
                let max = boxes.series.iter().map(|series| series.max).reduce(f64::max).unwrap_or(1.0);
                let y = |value| 220.0 - 200.0 * (value - min) / (max - min).max(f64::EPSILON);
                for (index, series) in boxes.series.iter().enumerate() {
                    let x = 40.0 + 560.0 * (index as f64 + 0.5) / boxes.series.len().max(1) as f64;
                    let color = COLORS[index % COLORS.len()];
                    write!(
                        output,
                        "<path d=\"M{x} {}V{}\" stroke=\"{color}\"/><rect x=\"{}\" y=\"{}\" width=\"24\" height=\"{}\" fill=\"none\" stroke=\"{color}\"/><path d=\"M{} {}h24\" stroke=\"{color}\"/>",
                        y(series.lower_whisker),
                        y(series.upper_whisker),
                        x - 12.0,
                        y(series.q3),
                        y(series.q1) - y(series.q3),
                        x - 12.0,
                        y(series.median)
                    )?;
                    for value in &series.outliers {
                        write!(output, "<circle cx=\"{x}\" cy=\"{}\" r=\"2\" fill=\"{color}\"/>", y(*value))?;
                    }
                }
            }
            v1::plot_analysis_result::Kind::SigmaInterval(interval) => {
                let bins = interval.series.iter().flat_map(|series| &series.bins).collect::<Vec<_>>();
                let min_x = bins.iter().map(|bin| bin.x).reduce(f64::min).unwrap_or(0.0);
                let max_x = bins.iter().map(|bin| bin.x).reduce(f64::max).unwrap_or(1.0);
                let min_y = bins.iter().map(|bin| bin.lower).reduce(f64::min).unwrap_or(0.0);
                let max_y = bins.iter().map(|bin| bin.upper).reduce(f64::max).unwrap_or(1.0);
                for (index, series) in interval.series.iter().enumerate() {
                    for bin in &series.bins {
                        let x = 20.0 + 600.0 * (bin.x - min_x) / (max_x - min_x).max(f64::EPSILON);
                        let y = |value| 220.0 - 200.0 * (value - min_y) / (max_y - min_y).max(f64::EPSILON);
                        let color = COLORS[index % COLORS.len()];
                        write!(
                            output,
                            "<path d=\"M{x} {}V{}\" stroke=\"{color}\"/><circle cx=\"{x}\" cy=\"{}\" r=\"3\" fill=\"{color}\"/>",
                            y(bin.lower),
                            y(bin.upper),
                            y(bin.mean)
                        )?;
                    }
                }
            }
            v1::plot_analysis_result::Kind::Contour(grid) => {
                for row in 0..grid.rows as usize {
                    for column in 0..grid.columns as usize {
                        let index = row * grid.columns as usize + column;
                        if !grid.valid.get(index).copied().unwrap_or(false) {
                            continue;
                        }
                        let width = 600.0 / grid.columns.max(1) as f64;
                        let height = 200.0 / grid.rows.max(1) as f64;
                        write!(
                            output,
                            "<rect x=\"{}\" y=\"{}\" width=\"{width}\" height=\"{height}\" fill=\"{}\"><title>{}</title></rect>",
                            20.0 + column as f64 * width,
                            20.0 + row as f64 * height,
                            color(grid.values[index], &grid.values),
                            grid.values[index]
                        )?;
                    }
                }
            }
            v1::plot_analysis_result::Kind::GridChart(grid) => {
                if let Some(frame) = grid.frames.first() {
                    grid_frame(output, grid, frame)?;
                }
            }
            v1::plot_analysis_result::Kind::Error(error) => return Err(Error::Invalid(format!("Render Plot: {}", error.message))),
        }
    }
    write!(output, "</svg>")?;
    if let Some(v1::plot_analysis_result::Kind::GridChart(grid)) =
        payload.analysis.as_ref().and_then(|analysis| analysis.result.as_ref()).and_then(|result| result.kind.as_ref())
    {
        for frame in grid.frames.iter().skip(1) {
            write!(
                output,
                "<details><summary>{}</summary><svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 640 240\">",
                escape(&frame.instant)
            )?;
            grid_frame(output, grid, frame)?;
            write!(output, "</svg></details>")?;
        }
    }
    write!(output, "</figure>")?;
    Ok(())
}

/* === Private Functions === */

fn render_references(output: &mut impl Write, payload: &v1::PlotPayload, presentation: &v1::PlotPresentation) -> Result<()> {
    if presentation.reference_lines.is_empty() && presentation.reference_bands.is_empty() {
        return Ok(());
    }
    let mut values = payload
        .frame
        .iter()
        .flat_map(|frame| &frame.series)
        .flat_map(|column| column.y.iter().filter_map(|value| value.value))
        .collect::<Vec<_>>();
    if let Some(analysis) = payload.analysis.as_ref().and_then(|analysis| analysis.result.as_ref()).and_then(|result| result.kind.as_ref())
    {
        match analysis {
            v1::plot_analysis_result::Kind::Histogram(histogram) => {
                values.extend(histogram.series.iter().flat_map(|series| series.bins.iter().map(|bin| bin.count as f64)))
            }
            v1::plot_analysis_result::Kind::BoxPlot(boxes) => {
                values.extend(boxes.series.iter().flat_map(|series| [series.min, series.max]))
            }
            v1::plot_analysis_result::Kind::SigmaInterval(interval) => {
                values.extend(interval.series.iter().flat_map(|series| series.bins.iter().flat_map(|bin| [bin.lower, bin.upper])))
            }
            v1::plot_analysis_result::Kind::Contour(grid) => values.extend(&grid.y_edges),
            _ => (),
        }
    }
    let min = values.iter().copied().reduce(f64::min).unwrap_or(0.0).min(0.0);
    let max = values.iter().copied().reduce(f64::max).unwrap_or(1.0);
    let y = |value| 220.0 - 200.0 * (value - min) / (max - min).max(f64::EPSILON);
    let constant = |value: Option<&v1::PlotReferenceValue>| value.and_then(|value| value.constant);
    for band in &presentation.reference_bands {
        let bounds = match (
            constant(band.min.as_ref()),
            constant(band.max.as_ref()),
            constant(band.center.as_ref()),
            constant(band.tolerance.as_ref()),
        ) {
            (Some(min), Some(max), _, _) => Some((min, max)),
            (_, _, Some(center), Some(tolerance)) if tolerance >= 0.0 => Some((center - tolerance, center + tolerance)),
            _ => None,
        };
        let Some((min, max)) = bounds.filter(|(min, max)| min <= max) else {
            return Err(Error::Invalid("Render Reference Band: Supply Increasing Bounds or a Center and Nonnegative Tolerance".into()));
        };
        let color = if band.color.is_empty() { "#4477aa" } else { &band.color };
        write!(
            output,
            "<rect x=\"20\" y=\"{}\" width=\"600\" height=\"{}\" fill=\"{}\" opacity=\"0.15\"><title>{}</title></rect>",
            y(max),
            y(min) - y(max),
            escape(color),
            escape(band.label.as_deref().unwrap_or_default())
        )?;
    }
    for line in &presentation.reference_lines {
        let value =
            constant(line.value.as_ref()).ok_or_else(|| Error::Invalid("Render Reference Line: Supply a Fixed Scalar Value".into()))?;
        let color = if line.color.is_empty() { "#444444" } else { &line.color };
        write!(
            output,
            "<path d=\"M20 {}H620\" stroke=\"{}\" stroke-dasharray=\"4 3\"><title>{}</title></path>",
            y(value),
            escape(color),
            escape(line.label.as_deref().unwrap_or_default())
        )?;
    }
    Ok(())
}
fn render_frame(output: &mut impl Write, plot: &v1::Plot, frame: &v1::PlotFrame) -> Result<()> {
    let kind = v1::PlotKind::try_from(plot.kind).unwrap_or(v1::PlotKind::TimeSeries);
    if kind == v1::PlotKind::Pie {
        let values = frame.series.iter().flat_map(|column| column.y.iter().filter_map(|value| value.value)).collect::<Vec<_>>();
        let total = values.iter().sum::<f64>();
        let mut angle = -std::f64::consts::FRAC_PI_2;
        if total > 0.0 {
            for (index, value) in values.iter().enumerate() {
                let next = angle + std::f64::consts::TAU * value / total;
                let start = (320.0 + 90.0 * angle.cos(), 120.0 + 90.0 * angle.sin());
                let end = (320.0 + 90.0 * next.cos(), 120.0 + 90.0 * next.sin());
                if *value == total {
                    write!(output, "<circle cx=\"320\" cy=\"120\" r=\"90\" fill=\"{}\"/>", COLORS[index % COLORS.len()])?;
                } else {
                    write!(
                        output,
                        "<path d=\"M320 120L{} {}A90 90 0 {} 1 {} {}Z\" fill=\"{}\"/>",
                        start.0,
                        start.1,
                        u8::from(next - angle > std::f64::consts::PI),
                        end.0,
                        end.1,
                        COLORS[index % COLORS.len()]
                    )?;
                }
                angle = next;
            }
        }
        if let Some(hole) = plot.hole {
            write!(output, "<circle cx=\"320\" cy=\"120\" r=\"{}\" fill=\"white\"/>", 90.0 * hole)?;
        }
        return Ok(());
    }
    let min_y =
        frame.series.iter().flat_map(|column| column.y.iter().filter_map(|value| value.value)).reduce(f64::min).unwrap_or(0.0).min(0.0);
    let max_y = frame.series.iter().flat_map(|column| column.y.iter().filter_map(|value| value.value)).reduce(f64::max).unwrap_or(1.0);
    if kind == v1::PlotKind::Bar {
        let categories = frame
            .series
            .iter()
            .flat_map(|column| column.x.iter().map(display))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let stacked = plot.arrangement == Some(v1::BarArrangement::Stacked as i32);
        let mut stacks = vec![0.0; categories.len()];
        let max = if stacked {
            categories
                .iter()
                .map(|category| {
                    frame
                        .series
                        .iter()
                        .map(|column| {
                            column
                                .x
                                .iter()
                                .zip(&column.y)
                                .filter(|(x, _)| display(x) == *category)
                                .filter_map(|(_, y)| y.value)
                                .sum::<f64>()
                        })
                        .sum::<f64>()
                })
                .reduce(f64::max)
                .unwrap_or(1.0)
        } else {
            max_y
        };
        let slot = 600.0 / categories.len().max(1) as f64;
        let total = frame.series.iter().flat_map(|column| column.y.iter().filter_map(|value| value.value)).sum::<f64>();
        let mut cumulative = 0.0;
        let mut line = Vec::new();
        for (series_index, column) in frame.series.iter().enumerate() {
            for (x, y) in column.x.iter().zip(&column.y) {
                let Some(value) = y.value else {
                    continue;
                };
                let index = categories.iter().position(|category| *category == display(x)).unwrap_or(0);
                let width = if stacked { slot } else { slot / frame.series.len().max(1) as f64 };
                let x = 20.0 + index as f64 * slot + if stacked { 0.0 } else { series_index as f64 * width };
                let base = if stacked { stacks[index] } else { 0.0 };
                let top = base + value;
                stacks[index] = top;
                let y = 220.0 - 200.0 * (top - min_y) / (max - min_y).max(f64::EPSILON);
                let height = 200.0 * value.abs() / (max - min_y).max(f64::EPSILON);
                write!(
                    output,
                    "<rect x=\"{x}\" y=\"{y}\" width=\"{}\" height=\"{height}\" fill=\"{}\"/>",
                    width * 0.9,
                    COLORS[series_index % COLORS.len()]
                )?;
                if plot.pareto && total > 0.0 {
                    cumulative += value;
                    line.push((x + width / 2.0, 220.0 - 200.0 * cumulative / total));
                }
            }
        }
        if !line.is_empty() {
            polyline(output, &line, "#222222")?;
        }
        return Ok(());
    }
    for (index, column) in frame.series.iter().enumerate() {
        let points = column.x.iter().zip(&column.y).filter_map(|(x, y)| Some((number(x)?, y.value?))).collect::<Vec<_>>();
        let min_x = points.iter().map(|point| point.0).reduce(f64::min).unwrap_or(0.0);
        let max_x = points.iter().map(|point| point.0).reduce(f64::max).unwrap_or(1.0);
        let points = points
            .into_iter()
            .map(|(x, y)| {
                (
                    20.0 + 600.0 * (x - min_x) / (max_x - min_x).max(f64::EPSILON),
                    220.0 - 200.0 * (y - min_y) / (max_y - min_y).max(f64::EPSILON),
                )
            })
            .collect::<Vec<_>>();
        let color = COLORS[index % COLORS.len()];
        if plot.style != v1::PlotStyle::Points as i32 {
            polyline(output, &points, color)?;
        }
        if matches!(v1::PlotStyle::try_from(plot.style), Ok(v1::PlotStyle::Points | v1::PlotStyle::LinePoints)) {
            for (x, y) in &points {
                write!(output, "<circle cx=\"{x}\" cy=\"{y}\" r=\"2\" fill=\"{color}\"/>")?;
            }
        }
    }
    Ok(())
}
fn grid_frame(output: &mut impl Write, grid: &v1::GridFrames, frame: &v1::GridFrame) -> Result<()> {
    let width = 600.0 / grid.columns.len().max(1) as f64;
    let height = 200.0 / grid.rows.len().max(1) as f64;
    for row in 0..grid.rows.len() {
        for column in 0..grid.columns.len() {
            let index = row * grid.columns.len() + column;
            if !frame.valid.get(index).copied().unwrap_or(false) {
                continue;
            }
            let x = 20.0 + column as f64 * width;
            let y = 20.0 + row as f64 * height;
            write!(
                output,
                "<rect x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\" fill=\"{}\"><title>{} / {}: {}</title></rect>",
                color(frame.values[index], &frame.values),
                escape(&grid.rows[row].label),
                escape(&grid.columns[column].label),
                frame.values[index]
            )?;
        }
    }
    for highlight in &frame.highlights {
        write!(
            output,
            "<rect x=\"{}\" y=\"{}\" width=\"{width}\" height=\"{height}\" fill=\"none\" stroke=\"#222222\" stroke-width=\"3\"><title>{}</title></rect>",
            20.0 + highlight.column as f64 * width,
            20.0 + highlight.row as f64 * height,
            escape(&highlight.label)
        )?;
    }
    Ok(())
}
fn polyline(output: &mut impl Write, points: &[(f64, f64)], color: &str) -> Result<()> {
    write!(output, "<polyline fill=\"none\" stroke=\"{color}\" points=\"")?;
    for (x, y) in points {
        write!(output, "{x:.3},{y:.3} ")?;
    }
    write!(output, "\"></polyline>")?;
    Ok(())
}
fn color(value: f64, values: &[f64]) -> String {
    let min = values.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max = values.iter().copied().reduce(f64::max).unwrap_or(1.0);
    let level = ((value - min) / (max - min).max(f64::EPSILON)).clamp(0.0, 1.0);
    format!("hsl({},60%,50%)", 240.0 * (1.0 - level))
}
fn number(value: &v1::Value) -> Option<f64> {
    match value.kind.as_ref()? {
        v1::value::Kind::Number(value) => Some(*value),
        v1::value::Kind::Integer(value) => Some(*value as f64),
        v1::value::Kind::UnsignedInteger(value) => Some(*value as f64),
        _ => None,
    }
}
pub(crate) fn display(value: &v1::Value) -> String {
    match value.kind.as_ref() {
        Some(v1::value::Kind::Text(value)) => value.clone(),
        Some(v1::value::Kind::Boolean(value)) => value.to_string(),
        Some(v1::value::Kind::Number(value)) => value.to_string(),
        Some(v1::value::Kind::Integer(value)) => value.to_string(),
        Some(v1::value::Kind::UnsignedInteger(value)) => value.to_string(),
        _ => String::new(),
    }
}
fn escape(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

/* === Tests === */
