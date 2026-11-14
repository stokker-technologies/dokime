/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Thin Conveniences Over The Generated Document Messages.

/* === Modules === */

/* === Re-Exports === */

pub use crate::v1::{AxisScale, BarArrangement, ContourRender, GridAxisEntry, GridHighlight, PlotStyle, Reduction, XyInterval};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::v1::{self, *};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plot {
    pub message: v1::Plot,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl From<v1::Plot> for Plot {
    fn from(message: v1::Plot) -> Self {
        Self { message }
    }
}
impl From<Plot> for v1::Plot {
    fn from(plot: Plot) -> Self {
        plot.message
    }
}
impl std::ops::Deref for Plot {
    type Target = v1::Plot;
    fn deref(&self) -> &Self::Target {
        &self.message
    }
}
impl std::ops::DerefMut for Plot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.message
    }
}

impl FieldReference {
    pub fn new(identifier: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            field: String::new(),
        }
    }
    pub fn field(&self, field: impl Into<String>) -> DataExpression {
        DataExpression {
            label: String::new(),
            kind: Some(data_expression::Kind::Stream(Self {
                identifier: self.identifier.clone(),
                field: field.into(),
            })),
        }
    }
}

impl DataExpression {
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    pub fn scale(self, factor: f64) -> Self {
        self.transform(transform_expression::Kind::Scale(factor))
    }
    pub fn offset(self, value: f64) -> Self {
        self.transform(transform_expression::Kind::Offset(value))
    }
    pub fn detail(self, field: impl Into<String>) -> Self {
        self.transform(transform_expression::Kind::Detail(field.into()))
    }
    pub fn time_between(self, record: impl Into<String>) -> Self {
        self.transform(transform_expression::Kind::TimeBetween(record.into()))
    }
    pub fn per_case(self) -> Self {
        self.transform(transform_expression::Kind::PerCase(true))
    }
    pub fn aggregate(self, kind: Aggregate) -> Self {
        self.aggregate_with_percentile(kind, 0.0)
    }
    pub fn percentile(self, rank: f64) -> Self {
        self.aggregate_with_percentile(Aggregate::Percentile, rank)
    }
    fn aggregate_with_percentile(self, kind: Aggregate, percentile: f64) -> Self {
        Self {
            label: self.label.clone(),
            kind: Some(data_expression::Kind::Aggregate(Box::new(AggregateExpression {
                input: Some(Box::new(self)),
                kind: kind as i32,
                percentile,
            }))),
        }
    }
    fn transform(self, kind: transform_expression::Kind) -> Self {
        Self {
            label: self.label.clone(),
            kind: Some(data_expression::Kind::Transform(Box::new(TransformExpression {
                input: Some(Box::new(self)),
                kind: Some(kind),
            }))),
        }
    }
    pub fn count(self) -> Self {
        self.aggregate(Aggregate::Count)
    }
    pub fn sum(self) -> Self {
        self.aggregate(Aggregate::Sum)
    }
    pub fn min(self) -> Self {
        self.aggregate(Aggregate::Min)
    }
    pub fn max(self) -> Self {
        self.aggregate(Aggregate::Max)
    }
    pub fn mean(self) -> Self {
        self.aggregate(Aggregate::Mean)
    }
    pub fn std(self) -> Self {
        self.aggregate(Aggregate::Stddev)
    }
    pub fn median(self) -> Self {
        self.aggregate(Aggregate::Median)
    }
    pub fn rms(self) -> Self {
        self.aggregate(Aggregate::Rms)
    }
}

impl Plot {
    pub fn time_series(value: DataExpression) -> Self {
        Self::from(v1::Plot {
            kind: PlotKind::TimeSeries as i32,
            y: vec![value],
            ..Default::default()
        })
    }
    pub fn frequency_series(value: DataExpression) -> Self {
        Self::from(v1::Plot {
            kind: PlotKind::FrequencySeries as i32,
            y: vec![value],
            ..Default::default()
        })
    }
    pub fn xy(value: DataExpression) -> Self {
        Self::from(v1::Plot {
            kind: PlotKind::Xy as i32,
            y: vec![value],
            ..Default::default()
        })
    }
    pub fn trajectory(value: DataExpression) -> Self {
        Self::from(v1::Plot {
            kind: PlotKind::Trajectory as i32,
            y: vec![value],
            ..Default::default()
        })
    }
    pub fn contour(value: DataExpression) -> Self {
        Self::from(v1::Plot {
            kind: PlotKind::Contour as i32,
            z: Some(value),
            ..Default::default()
        })
    }
    pub fn grid_chart(value: DataExpression) -> Self {
        Self::from(v1::Plot {
            kind: PlotKind::GridChart as i32,
            y: vec![value],
            ..Default::default()
        })
    }
    pub fn bar(value: DataExpression) -> Self {
        Self::from(v1::Plot {
            kind: PlotKind::Bar as i32,
            y: vec![value],
            ..Default::default()
        })
    }
    pub fn pie(value: DataExpression) -> Self {
        Self::from(v1::Plot {
            kind: PlotKind::Pie as i32,
            y: vec![value],
            ..Default::default()
        })
    }
    pub fn histogram(value: DataExpression) -> Self {
        Self::from(v1::Plot {
            kind: PlotKind::Histogram as i32,
            x: Some(value),
            ..Default::default()
        })
    }
    pub fn box_plot(value: DataExpression) -> Self {
        Self::from(v1::Plot {
            kind: PlotKind::BoxPlot as i32,
            y: vec![value],
            ..Default::default()
        })
    }
    pub fn x(mut self, value: DataExpression) -> Self {
        self.x = Some(value);
        self
    }
    pub fn y(mut self, value: DataExpression) -> Self {
        self.y.push(value);
        self
    }
    pub fn series_by(mut self, value: DataExpression) -> Self {
        self.series_by = Some(value);
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        let id = self.metadata.as_ref().map(|metadata| metadata.identifier.clone()).filter(|id| !id.is_empty());
        let mut value = metadata(label.into());
        if let Some(id) = id {
            value.identifier = id;
        }
        self.metadata = Some(value);
        self
    }
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.metadata.get_or_insert_default().identifier = id.into();
        self
    }
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }
    pub fn style(mut self, style: PlotStyle) -> Self {
        self.style = style as i32;
        self
    }
    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale as i32;
        self
    }
    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale as i32;
        self
    }
    pub fn reduce(mut self, reduction: Reduction) -> Self {
        self.reduction = Some(reduction as i32);
        self
    }
    pub fn render(mut self, render: ContourRender) -> Self {
        self.render = render as i32;
        self
    }
    pub fn arrangement(mut self, arrangement: BarArrangement) -> Self {
        self.arrangement = Some(arrangement as i32);
        self
    }
    pub fn interval(mut self, interval: XyInterval) -> Self {
        self.interval = Some(interval as i32);
        self
    }
    pub fn pareto(mut self, pareto: bool) -> Self {
        self.pareto = pareto;
        self
    }
    pub fn hole(mut self, fraction: f64) -> Self {
        self.hole = Some(fraction);
        self
    }
    pub fn sample_rate_hz(mut self, rate: f64) -> Self {
        self.sample_rate_hz = Some(rate);
        self
    }
    pub fn input_samples(mut self, count: u32) -> Self {
        self.input_samples = Some(count);
        self
    }
    pub fn bins(mut self, count: u32) -> Self {
        self.histogram_bins = Some(count);
        self
    }
    pub fn display_points(mut self, count: u32) -> Self {
        self.display_points = Some(count);
        self
    }
    pub fn row(mut self, value: DataExpression) -> Self {
        self.grid.get_or_insert_default().row = Some(value);
        self
    }
    pub fn column(mut self, value: DataExpression) -> Self {
        self.grid.get_or_insert_default().column = Some(value);
        self
    }
    pub fn frame(mut self, value: DataExpression) -> Self {
        self.grid.get_or_insert_default().frame = Some(value);
        self
    }
    pub fn rows(mut self, entries: impl IntoIterator<Item = GridAxisEntry>) -> Self {
        self.grid.get_or_insert_default().rows = entries.into_iter().collect();
        self
    }
    pub fn columns(mut self, entries: impl IntoIterator<Item = GridAxisEntry>) -> Self {
        self.grid.get_or_insert_default().columns = entries.into_iter().collect();
        self
    }
    pub fn highlight(mut self, highlight: GridHighlight) -> Self {
        self.grid.get_or_insert_default().highlights.push(highlight);
        self
    }
    pub fn summary_label(mut self, label: impl Into<String>) -> Self {
        self.grid.get_or_insert_default().summary_label = label.into();
        self
    }
    pub fn reference_line(mut self, line: ReferenceLine) -> Self {
        self.reference_lines.push(line);
        self
    }
    pub fn reference_band(mut self, band: ReferenceBand) -> Self {
        self.reference_bands.push(band);
        self
    }
}

impl Document {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            metadata: Some(metadata(label.into())),
            ..Default::default()
        }
    }
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.metadata.get_or_insert_default().identifier = id.into();
        self
    }
    pub fn section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }
}

impl Section {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            metadata: Some(metadata(label.into())),
            ..Default::default()
        }
    }
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.metadata.get_or_insert_default().identifier = id.into();
        self
    }
    pub fn section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }
    pub fn card(mut self, card: Card) -> Self {
        self.cards.push(card);
        self
    }
    pub fn plot(self, plot: impl Into<v1::Plot>) -> Self {
        let mut plot = plot.into();
        if plot.metadata.as_ref().is_none_or(|metadata| metadata.label.is_empty()) {
            let label =
                plot.z.as_ref().or_else(|| plot.y.first()).or(plot.x.as_ref()).map(expression_label).unwrap_or_else(|| "Plot".into());
            let id = plot.metadata.as_ref().map(|metadata| metadata.identifier.clone()).filter(|id| !id.is_empty());
            plot.metadata = Some(metadata(label));
            if let Some(id) = id {
                plot.metadata.as_mut().expect("initialized metadata").identifier = id;
            }
        }
        self.card(Card {
            metadata: plot.metadata.clone(),
            body: Some(card::Body::Plot(plot)),
            ..Default::default()
        })
    }
    pub fn summary(self, label: impl Into<String>, summary: Summary) -> Self {
        self.card(Card {
            metadata: Some(metadata(label.into())),
            body: Some(card::Body::Summary(summary)),
            ..Default::default()
        })
    }
}

impl Summary {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn tile(mut self, label: impl Into<String>, value: DataExpression) -> Self {
        self.tiles.push(SummaryTile {
            metadata: Some(metadata(label.into())),
            value: Some(value),
            unit: String::new(),
        });
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn metadata(label: String) -> Metadata {
    let mut identifier = String::new();
    for word in label.split(|c: char| !c.is_alphanumeric()).filter(|word| !word.is_empty()) {
        if !identifier.is_empty() {
            identifier.push('.');
        }
        identifier.push_str(&word.to_lowercase());
    }
    Metadata {
        identifier,
        label,
        description: String::new(),
        ..Default::default()
    }
}

fn expression_label(value: &DataExpression) -> String {
    if !value.label.is_empty() {
        return value.label.clone();
    }
    match value.kind.as_ref() {
        Some(data_expression::Kind::Stream(field) | data_expression::Kind::Record(field)) => field.field.clone(),
        Some(data_expression::Kind::Transform(transform)) => {
            transform.input.as_deref().map(expression_label).unwrap_or_else(|| "Value".into())
        }
        Some(data_expression::Kind::Aggregate(aggregate)) => {
            aggregate.input.as_deref().map(expression_label).unwrap_or_else(|| "Value".into())
        }
        _ => "Value".into(),
    }
}

/* === Tests === */
