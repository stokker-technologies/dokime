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
use std::ops::{Bound, RangeBounds};

// Third Party Imports

// Local Imports
use crate::model::{
    self, DataExpression, Metadata, ParameterExpansion, ParameterExpression, ParameterExpressions, PlotKind, ReferenceBand, ReferenceLine,
    Value, ValueList, ValueType, metadata, parameter_expansion, parameter_expression, value,
};

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

/// An Owned Parameter Value Under Construction.
pub struct ParameterBuilder {
    message: model::Parameter,
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

    pub fn heat_strip(value: DataExpression) -> Self {
        Self::from(model::Plot {
            kind: PlotKind::HeatStrip as i32,
            y: vec![value],
            reduction: Some(Reduction::Last as i32),
            ..Default::default()
        })
    }

    pub fn cells(mut self, value: DataExpression) -> Self {
        self.message.cell = Some(value);
        self
    }

    pub fn facet(mut self, value: DataExpression) -> Self {
        self.message.facet = Some(value);
        self
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

impl model::Parameter {
    pub fn number(identifier: impl Into<String>, label: impl Into<String>) -> ParameterBuilder {
        ParameterBuilder::new(identifier, label, ValueType::Number)
    }

    pub fn count(identifier: impl Into<String>, label: impl Into<String>) -> ParameterBuilder {
        ParameterBuilder::new(identifier, label, ValueType::UnsignedInteger)
    }

    pub fn flag(identifier: impl Into<String>, label: impl Into<String>) -> ParameterBuilder {
        ParameterBuilder::new(identifier, label, ValueType::Boolean)
    }

    pub fn sweep(identifier: impl Into<String>, label: impl Into<String>, values: impl IntoIterator<Item = f64>) -> ParameterBuilder {
        let mut builder = Self::number(identifier, label);
        builder.message.expression = Some(ParameterExpression {
            kind: Some(parameter_expression::Kind::Array(ParameterExpressions {
                values: values.into_iter().map(|number| literal(value::Kind::Number(number))).collect(),
            })),
        });
        builder.message.expansion = Some(ParameterExpansion {
            kind: Some(parameter_expansion::Kind::Cartesian(true)),
        });
        builder
    }

    pub fn numbers(identifier: impl Into<String>, label: impl Into<String>, values: impl IntoIterator<Item = f64>) -> ParameterBuilder {
        let mut builder = ParameterBuilder::new(identifier, label, ValueType::List);
        builder.message.element_type = ValueType::Number as i32;
        builder.message.expression = Some(literal(value::Kind::List(ValueList {
            values: values.into_iter().map(|number| Value { kind: Some(value::Kind::Number(number)) }).collect(),
        })));
        builder
    }
}

impl ParameterBuilder {
    fn new(identifier: impl Into<String>, label: impl Into<String>, kind: ValueType) -> Self {
        Self {
            message: model::Parameter {
                metadata: Some(Metadata::new(identifier, label)),
                r#type: kind as i32,
                ..Default::default()
            },
        }
    }

    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.message.unit = unit.into();
        self
    }

    pub fn default(mut self, value: impl Into<value::Kind>) -> Self {
        self.message.expression = Some(literal(value.into()));
        self
    }
    /// Inclusive Bounds Only; Open Bounds Are Unbounded.
    pub fn range(mut self, bounds: impl RangeBounds<f64>) -> Self {
        self.message.min = match bounds.start_bound() {
            Bound::Included(value) => Some(*value),
            Bound::Unbounded => None,
            Bound::Excluded(_) => panic!("Parameter Range: Use Inclusive Bounds"),
        };
        self.message.max = match bounds.end_bound() {
            Bound::Included(value) => Some(*value),
            Bound::Unbounded => None,
            Bound::Excluded(_) => panic!("Parameter Range: Use Inclusive Bounds"),
        };
        self
    }

    pub fn items(mut self, bounds: impl RangeBounds<u32>) -> Self {
        self.message.min_items = match bounds.start_bound() {
            Bound::Included(value) => Some(*value),
            Bound::Unbounded => None,
            Bound::Excluded(_) => panic!("Parameter Items: Use Inclusive Bounds"),
        };
        self.message.max_items = match bounds.end_bound() {
            Bound::Included(value) => Some(*value),
            Bound::Unbounded => None,
            Bound::Excluded(_) => panic!("Parameter Items: Use Inclusive Bounds"),
        };
        self
    }

    pub fn build(self) -> model::Parameter {
        self.message
    }
}

impl From<ParameterBuilder> for model::Parameter {
    fn from(builder: ParameterBuilder) -> Self {
        builder.message
    }
}

impl From<f64> for value::Kind {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<u64> for value::Kind {
    fn from(value: u64) -> Self {
        Self::UnsignedInteger(value)
    }
}

impl From<bool> for value::Kind {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn literal(kind: value::Kind) -> ParameterExpression {
    ParameterExpression {
        kind: Some(parameter_expression::Kind::Literal(Value { kind: Some(kind) })),
    }
}

/* === Tests === */
