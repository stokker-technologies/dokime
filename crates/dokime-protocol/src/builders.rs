/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Builders

/* === Modules === */

/* === Re-Exports === */

pub use crate::model::{AxisScale, BarArrangement, ContourRender, GridAxisEntry, GridHighlight, PlotStyle, Reduction, XyInterval};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::model::{self, DataExpression, PlotKind, ReferenceBand, ReferenceLine, metadata};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// A Plot Authoring Value Separate from Prost's Generated Enum and Optional-Field Getters.
///
/// Generated getters occupy names such as `style` and `hole`; the authoring value
/// uses those names for composition and transfers its message into a document.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plot {
    pub message: model::Plot,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl From<model::Plot> for Plot {
    fn from(message: model::Plot) -> Self {
        Self { message }
    }
}

impl From<Plot> for model::Plot {
    fn from(plot: Plot) -> Self {
        plot.message
    }
}

impl Plot {
    pub fn time_series(value: DataExpression) -> Self {
        Self::from(model::Plot {
            kind: PlotKind::TimeSeries as i32,
            y: vec![value],
            ..Default::default()
        })
    }

    pub fn frequency_series(value: DataExpression) -> Self {
        Self::from(model::Plot {
            kind: PlotKind::FrequencySeries as i32,
            y: vec![value],
            ..Default::default()
        })
    }

    pub fn xy(value: DataExpression) -> Self {
        Self::from(model::Plot {
            kind: PlotKind::Xy as i32,
            y: vec![value],
            ..Default::default()
        })
    }

    pub fn trajectory(value: DataExpression) -> Self {
        Self::from(model::Plot {
            kind: PlotKind::Trajectory as i32,
            y: vec![value],
            ..Default::default()
        })
    }

    pub fn contour(value: DataExpression) -> Self {
        Self::from(model::Plot {
            kind: PlotKind::Contour as i32,
            z: Some(value),
            ..Default::default()
        })
    }

    pub fn grid_chart(value: DataExpression) -> Self {
        Self::from(model::Plot {
            kind: PlotKind::GridChart as i32,
            y: vec![value],
            ..Default::default()
        })
    }

    pub fn bar(value: DataExpression) -> Self {
        Self::from(model::Plot {
            kind: PlotKind::Bar as i32,
            y: vec![value],
            ..Default::default()
        })
    }

    pub fn pie(value: DataExpression) -> Self {
        Self::from(model::Plot {
            kind: PlotKind::Pie as i32,
            y: vec![value],
            ..Default::default()
        })
    }

    pub fn histogram(value: DataExpression) -> Self {
        Self::from(model::Plot {
            kind: PlotKind::Histogram as i32,
            x: Some(value),
            ..Default::default()
        })
    }

    pub fn box_plot(value: DataExpression) -> Self {
        Self::from(model::Plot {
            kind: PlotKind::BoxPlot as i32,
            y: vec![value],
            ..Default::default()
        })
    }

    pub fn x(mut self, value: DataExpression) -> Self {
        self.message.x = Some(value);
        self
    }

    pub fn y(mut self, value: DataExpression) -> Self {
        self.message.y.push(value);
        self
    }

    pub fn series_by(mut self, value: DataExpression) -> Self {
        self.message.series_by = Some(value);
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        let identifier =
            self.message.metadata.as_ref().map(|metadata| metadata.identifier.clone()).filter(|identifier| !identifier.is_empty());
        let mut value = metadata::from_label(label.into());
        if let Some(identifier) = identifier {
            value.identifier = identifier;
        }
        self.message.metadata = Some(value);
        self
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.message.metadata.get_or_insert_default().identifier = id.into();
        self
    }

    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.message.unit = unit.into();
        self
    }

    pub fn style(mut self, style: PlotStyle) -> Self {
        self.message.style = style as i32;
        self
    }

    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.message.x_scale = scale as i32;
        self
    }

    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.message.y_scale = scale as i32;
        self
    }

    pub fn reduce(mut self, reduction: Reduction) -> Self {
        self.message.reduction = Some(reduction as i32);
        self
    }

    pub fn render(mut self, render: ContourRender) -> Self {
        self.message.render = render as i32;
        self
    }

    pub fn arrangement(mut self, arrangement: BarArrangement) -> Self {
        self.message.arrangement = Some(arrangement as i32);
        self
    }

    pub fn interval(mut self, interval: XyInterval) -> Self {
        self.message.interval = Some(interval as i32);
        self
    }

    pub fn pareto(mut self, pareto: bool) -> Self {
        self.message.pareto = pareto;
        self
    }

    pub fn hole(mut self, fraction: f64) -> Self {
        self.message.hole = Some(fraction);
        self
    }

    pub fn sample_rate_hz(mut self, rate: f64) -> Self {
        self.message.sample_rate_hz = Some(rate);
        self
    }

    pub fn input_samples(mut self, count: u32) -> Self {
        self.message.input_samples = Some(count);
        self
    }

    pub fn bins(mut self, count: u32) -> Self {
        self.message.histogram_bins = Some(count);
        self
    }

    pub fn display_points(mut self, count: u32) -> Self {
        self.message.display_points = Some(count);
        self
    }

    pub fn row(mut self, value: DataExpression) -> Self {
        self.message.grid.get_or_insert_default().row = Some(value);
        self
    }

    pub fn column(mut self, value: DataExpression) -> Self {
        self.message.grid.get_or_insert_default().column = Some(value);
        self
    }

    pub fn frame(mut self, value: DataExpression) -> Self {
        self.message.grid.get_or_insert_default().frame = Some(value);
        self
    }

    pub fn rows(mut self, entries: impl IntoIterator<Item = GridAxisEntry>) -> Self {
        self.message.grid.get_or_insert_default().rows = entries.into_iter().collect();
        self
    }

    pub fn columns(mut self, entries: impl IntoIterator<Item = GridAxisEntry>) -> Self {
        self.message.grid.get_or_insert_default().columns = entries.into_iter().collect();
        self
    }

    pub fn highlight(mut self, highlight: GridHighlight) -> Self {
        self.message.grid.get_or_insert_default().highlights.push(highlight);
        self
    }

    pub fn summary_label(mut self, label: impl Into<String>) -> Self {
        self.message.grid.get_or_insert_default().summary_label = label.into();
        self
    }

    pub fn reference_line(mut self, line: ReferenceLine) -> Self {
        self.message.reference_lines.push(line);
        self
    }

    pub fn reference_band(mut self, band: ReferenceBand) -> Self {
        self.message.reference_bands.push(band);
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
