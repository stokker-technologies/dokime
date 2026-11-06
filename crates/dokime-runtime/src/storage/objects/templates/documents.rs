/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{Map, Value, layout};
use crate::expressions::{DataExpression, DataSource as DomainSource, DataTransform};
use crate::storage::objects::metadata;
use crate::storage::{Error, Result};
use dokime_domain::model::values::ValueFormat;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    #[serde(default)]
    document_id: String,
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    cards: Vec<CardDefinition>,
    #[serde(default)]
    layout: layout::Layout,
    #[serde(default)]
    sections: Vec<Section>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CardDefinition {
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    filters: Vec<Filter>,
    #[serde(rename = "type")]
    #[serde(default)]
    kind: String,
    #[serde(default)]
    data: Option<CardData>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Card {
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    layout: CardLayout,
    #[serde(default)]
    filters: Vec<Filter>,
    #[serde(rename = "type")]
    #[serde(default)]
    kind: String,
    #[serde(default)]
    data: Option<CardData>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CardData {
    #[serde(default)]
    markdown: Option<String>,
    #[serde(default)]
    summary: Option<Summary>,
    #[serde(default)]
    plot: Option<Plot>,
    #[serde(default)]
    rows: Vec<Map>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Section {
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    cards: Vec<Card>,
    #[serde(default)]
    repeat: Option<Repeat>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Repeat {
    value: Data,
    #[serde(default)]
    label_template: String,
    #[serde(default)]
    id_template: Option<String>,
    #[serde(default)]
    order_by: Option<Data>,
    #[serde(default)]
    filters: Vec<Filter>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Placement {
    #[serde(default)]
    card_id: String,
    #[serde(default)]
    layout: CardLayout,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardLayout {
    #[serde(default)]
    col_span: Span,
    #[serde(default)]
    row_span: u32,
    #[serde(default)]
    order: i32,
    #[serde(default)]
    density: String,
    #[serde(default)]
    rail: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Span {
    #[serde(default)]
    base: u32,
    #[serde(default)]
    sm: Option<u32>,
    #[serde(default)]
    md: Option<u32>,
    #[serde(default)]
    lg: Option<u32>,
    #[serde(default)]
    xl: Option<u32>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Summary {
    #[serde(default)]
    groups: Vec<SummaryGroup>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SummaryGroup {
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    tiles: Vec<Tile>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Tile {
    #[serde(default)]
    metadata: Option<metadata::Row>,
    value: Data,
    #[serde(default)]
    filters: Vec<Filter>,
    #[serde(default)]
    stat: String,
    #[serde(default)]
    format: String,
    #[serde(default)]
    presentation: String,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    conditions: Vec<TileCondition>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TileCondition {
    #[serde(default)]
    operator: String,
    #[serde(default)]
    value: Value,
    #[serde(default)]
    state: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Data {
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    source: DataSource,
    #[serde(default)]
    transforms: Vec<Transform>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DataSource {
    #[serde(rename = "type")]
    #[serde(default)]
    kind: String,
    #[serde(default)]
    stream: Option<String>,
    #[serde(default)]
    field: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    record_type: Option<String>,
    #[serde(default)]
    artifact: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Transform {
    #[serde(rename = "type")]
    #[serde(default)]
    kind: String,
    #[serde(default)]
    previous_record_type: Option<String>,
    #[serde(default)]
    name: Option<String>,
    // The Harmos vocabulary's wire name (`mean`, `p95`) of an `aggregate`.
    #[serde(default)]
    aggregate: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filter {
    expression: Data,
    #[serde(default)]
    operator: String,
    #[serde(default)]
    value: Option<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Plot {
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    plot_type: String,
    #[serde(default)]
    stream: Option<String>,
    #[serde(default)]
    x: Option<String>,
    #[serde(default)]
    y: Vec<String>,
    #[serde(default)]
    z_stream: Option<String>,
    #[serde(default)]
    z_field: Option<String>,
    #[serde(default)]
    series_by: Option<String>,
    #[serde(default)]
    x_expression: Option<Data>,
    #[serde(default)]
    y_expressions: Vec<Data>,
    #[serde(default)]
    z_expression: Option<Data>,
    #[serde(default)]
    series_by_expression: Option<Data>,
    #[serde(default)]
    filters: Vec<Filter>,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    style: String,
    #[serde(default)]
    x_scale: String,
    #[serde(default)]
    y_scale: String,
    #[serde(default)]
    fft: Fft,
    #[serde(default)]
    interval: Interval,
    #[serde(default)]
    histogram: Histogram,
    #[serde(default)]
    marker_record_types: Vec<String>,
    #[serde(default)]
    reference_lines: Vec<Line>,
    #[serde(default)]
    reference_bands: Vec<Band>,
    #[serde(default)]
    display_points: Option<u32>,
    #[serde(default)]
    downsample: crate::document::DownsampleTechnique,
    #[serde(default)]
    x_format: Option<Format>,
    #[serde(default)]
    y_format: Option<Format>,
    #[serde(default)]
    x_axis: Option<Axis>,
    #[serde(default)]
    analysis: Option<super::analysis::Selection>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fft {
    #[serde(default)]
    sample_rate_hz: Option<f64>,
    #[serde(default)]
    window: String,
    #[serde(default)]
    detrend: String,
    #[serde(default)]
    level_unit: String,
    #[serde(default)]
    include_dc: bool,
    #[serde(default)]
    min_frequency_hz: Option<f64>,
    #[serde(default)]
    max_frequency_hz: Option<f64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interval {
    #[serde(default)]
    bin_width: f64,
    #[serde(default)]
    sigma: f64,
    #[serde(default)]
    clip_to_observed: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Histogram {
    #[serde(default)]
    bins: u64,
    #[serde(default)]
    unit: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    #[serde(default)]
    axis: String,
    #[serde(default)]
    value: Option<f64>,
    #[serde(default)]
    source: Option<Data>,
    #[serde(default)]
    source_scale: f64,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    color: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Band {
    #[serde(default)]
    axis: String,
    #[serde(default)]
    min: Option<f64>,
    #[serde(default)]
    max: Option<f64>,
    #[serde(default)]
    center: Option<Data>,
    #[serde(default)]
    tolerance: Option<Data>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    color: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Format {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    max_fraction_digits: Option<u32>,
    #[serde(default)]
    decimals: Option<u32>,
    #[serde(default)]
    significant_digits: Option<u32>,
    #[serde(default)]
    pattern: Option<String>,
    #[serde(default)]
    unit: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Axis {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    unit: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl TryFrom<&crate::document::DocumentDefinition> for Document {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentDefinition) -> Result<Self> {
        let mut cards = Vec::new();
        cards.try_reserve_exact(value.cards.len()).map_err(|_| Error::Invalid("document cards allocation failed"))?;
        for card in &value.cards {
            cards.push(CardDefinition::try_from(card)?);
        }
        let mut sections = Vec::new();
        sections.try_reserve_exact(value.sections.len()).map_err(|_| Error::Invalid("document sections allocation failed"))?;
        for section in &value.sections {
            sections.push(Section::try_from(section)?);
        }
        Ok(Self {
            document_id: value.document_id.as_str().to_owned(),
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            cards,
            layout: layout::Layout::try_from(&value.layout)?,
            sections,
        })
    }
}

impl TryFrom<Document> for crate::document::DocumentDefinition {
    type Error = Error;

    fn try_from(row: Document) -> Result<Self> {
        let mut cards = Vec::new();
        cards.try_reserve_exact(row.cards.len()).map_err(|_| Error::Invalid("document cards allocation failed"))?;
        for card in row.cards {
            cards.push(card.try_into()?);
        }
        let mut sections = Vec::new();
        sections.try_reserve_exact(row.sections.len()).map_err(|_| Error::Invalid("document sections allocation failed"))?;
        for section in row.sections {
            sections.push(section.try_into()?);
        }
        Ok(Self {
            document_id: dokime_domain::model::identity::DocumentId::from_wire(row.document_id)?,
            metadata: row.metadata.ok_or(Error::Invalid("document metadata is missing"))?.try_into()?,
            cards,
            layout: row.layout.try_into()?,
            sections,
        })
    }
}

impl TryFrom<&crate::document::DocumentCardDefinition> for CardDefinition {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentCardDefinition) -> Result<Self> {
        let mut filters = Vec::new();
        filters.try_reserve_exact(value.filters.len()).map_err(|_| Error::Invalid("document filters allocation failed"))?;
        for filter in &value.filters {
            filters.push(Filter::try_from(filter)?);
        }
        let (kind, data) = encode_body(&value.body)?;
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            filters,
            kind,
            data,
        })
    }
}

impl TryFrom<CardDefinition> for crate::document::DocumentCardDefinition {
    type Error = Error;

    fn try_from(row: CardDefinition) -> Result<Self> {
        let mut filters = Vec::new();
        filters.try_reserve_exact(row.filters.len()).map_err(|_| Error::Invalid("document card filters allocation failed"))?;
        for filter in row.filters {
            filters.push(filter.try_into()?);
        }
        Ok(Self {
            metadata: row.metadata.ok_or(Error::Invalid("document card metadata is missing"))?.try_into()?,
            filters,
            body: consume_body(row.kind, row.data)?,
        })
    }
}

impl TryFrom<&crate::document::DocumentCard> for Card {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentCard) -> Result<Self> {
        CardLayout::admit(&value.layout)?;
        let mut filters = Vec::new();
        filters.try_reserve_exact(value.filters.len()).map_err(|_| Error::Invalid("document filters allocation failed"))?;
        for filter in &value.filters {
            filters.push(Filter::try_from(filter)?);
        }
        let (kind, data) = encode_body(&value.body)?;
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            layout: value.layout.clone().into(),
            filters,
            kind,
            data,
        })
    }
}

impl TryFrom<Card> for crate::document::DocumentCard {
    type Error = Error;

    fn try_from(row: Card) -> Result<Self> {
        let mut filters = Vec::new();
        filters.try_reserve_exact(row.filters.len()).map_err(|_| Error::Invalid("document card filters allocation failed"))?;
        for filter in row.filters {
            filters.push(filter.try_into()?);
        }
        Ok(Self {
            metadata: row.metadata.ok_or(Error::Invalid("document card metadata is missing"))?.try_into()?,
            layout: row.layout.try_into()?,
            filters,
            body: consume_body(row.kind, row.data)?,
        })
    }
}

impl TryFrom<&crate::document::DocumentSection> for Section {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentSection) -> Result<Self> {
        let mut cards = Vec::new();
        cards.try_reserve_exact(value.cards.len()).map_err(|_| Error::Invalid("document cards allocation failed"))?;
        for card in &value.cards {
            cards.push(Card::try_from(card)?);
        }
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            cards,
            repeat: value.repeat.as_ref().map(Repeat::try_from).transpose()?,
        })
    }
}

impl TryFrom<Section> for crate::document::DocumentSection {
    type Error = Error;

    fn try_from(row: Section) -> Result<Self> {
        let mut cards = Vec::new();
        cards.try_reserve_exact(row.cards.len()).map_err(|_| Error::Invalid("section cards allocation failed"))?;
        for card in row.cards {
            cards.push(card.try_into()?);
        }
        Ok(Self {
            metadata: row.metadata.ok_or(Error::Invalid("document section metadata is missing"))?.try_into()?,
            cards,
            repeat: row.repeat.map(TryInto::try_into).transpose()?,
        })
    }
}

impl TryFrom<&crate::document::PlotDefinition> for Plot {
    type Error = Error;

    fn try_from(value: &crate::document::PlotDefinition) -> Result<Self> {
        let mut y_expressions = Vec::new();
        y_expressions.try_reserve_exact(value.y_expressions.len()).map_err(|_| Error::Invalid("plot expressions allocation failed"))?;
        for expression in &value.y_expressions {
            y_expressions.push(Data::try_from(expression)?);
        }
        let mut filters = Vec::new();
        filters.try_reserve_exact(value.filters.len()).map_err(|_| Error::Invalid("plot filters allocation failed"))?;
        for filter in &value.filters {
            filters.push(Filter::try_from(filter)?);
        }
        let mut reference_lines = Vec::new();
        reference_lines
            .try_reserve_exact(value.reference_lines.len())
            .map_err(|_| Error::Invalid("plot reference lines allocation failed"))?;
        for line in &value.reference_lines {
            reference_lines.push(Line::try_from(line)?);
        }
        let mut reference_bands = Vec::new();
        reference_bands
            .try_reserve_exact(value.reference_bands.len())
            .map_err(|_| Error::Invalid("plot reference bands allocation failed"))?;
        for band in &value.reference_bands {
            reference_bands.push(Band::try_from(band)?);
        }
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            plot_type: name(&value.plot_type),
            stream: value.stream.as_deref().map(str::to_owned),
            x: value.x.as_deref().map(str::to_owned),
            y: super::copied_strings(&value.y)?,
            z_stream: value.z_stream.as_deref().map(str::to_owned),
            z_field: value.z_field.as_deref().map(str::to_owned),
            series_by: value.series_by.as_deref().map(str::to_owned),
            x_expression: value.x_expression.as_ref().map(Data::try_from).transpose()?,
            y_expressions,
            z_expression: value.z_expression.as_ref().map(Data::try_from).transpose()?,
            series_by_expression: value.series_by_expression.as_ref().map(Data::try_from).transpose()?,
            filters,
            unit: value.unit.as_deref().map(str::to_owned),
            style: name(&value.style),
            x_scale: name(&value.x_scale),
            y_scale: name(&value.y_scale),
            fft: Fft::try_from(&value.fft)?,
            analysis: value.analysis.as_ref().map(super::analysis::Selection::try_from).transpose()?,
            interval: Interval::try_from(&value.interval)?,
            histogram: Histogram::try_from(&value.histogram)?,
            marker_record_types: super::copied_strings(&value.marker_record_types)?,
            reference_lines,
            reference_bands,
            display_points: value.display_points,
            downsample: value.downsample,
            x_format: value.x_format.as_ref().map(Format::try_from).transpose()?,
            y_format: value.y_format.as_ref().map(Format::try_from).transpose()?,
            x_axis: value.x_axis.as_ref().map(Axis::try_from).transpose()?,
        })
    }
}

impl TryFrom<Plot> for crate::document::PlotDefinition {
    type Error = Error;

    fn try_from(row: Plot) -> Result<Self> {
        let mut y_expressions = Vec::new();
        y_expressions.try_reserve_exact(row.y_expressions.len()).map_err(|_| Error::Invalid("plot expressions allocation failed"))?;
        for expression in row.y_expressions {
            y_expressions.push(expression.try_into()?);
        }
        let mut filters = Vec::new();
        filters.try_reserve_exact(row.filters.len()).map_err(|_| Error::Invalid("plot filters allocation failed"))?;
        for filter in row.filters {
            filters.push(filter.try_into()?);
        }
        let mut reference_lines = Vec::new();
        reference_lines
            .try_reserve_exact(row.reference_lines.len())
            .map_err(|_| Error::Invalid("plot reference lines allocation failed"))?;
        for line in row.reference_lines {
            reference_lines.push(line.try_into()?);
        }
        let mut reference_bands = Vec::new();
        reference_bands
            .try_reserve_exact(row.reference_bands.len())
            .map_err(|_| Error::Invalid("plot reference bands allocation failed"))?;
        for band in row.reference_bands {
            reference_bands.push(band.try_into()?);
        }
        Ok(Self {
            automatic_id: false,
            metadata: row.metadata.ok_or(Error::Invalid("plot metadata is missing"))?.try_into()?,
            plot_type: parse(&row.plot_type, "invalid plot type")?,
            stream: row.stream,
            x: row.x,
            y: row.y,
            z_stream: row.z_stream,
            z_field: row.z_field,
            series_by: row.series_by,
            x_expression: row.x_expression.map(TryInto::try_into).transpose()?,
            y_expressions,
            z_expression: row.z_expression.map(TryInto::try_into).transpose()?,
            series_by_expression: row.series_by_expression.map(TryInto::try_into).transpose()?,
            filters,
            unit: row.unit,
            style: parse(&row.style, "invalid plot style")?,
            x_scale: parse(&row.x_scale, "invalid axis scale")?,
            y_scale: parse(&row.y_scale, "invalid axis scale")?,
            fft: row.fft.try_into()?,
            analysis: row.analysis.map(TryInto::try_into).transpose()?,
            interval: row.interval.try_into()?,
            histogram: row.histogram.try_into()?,
            marker_record_types: row.marker_record_types,
            reference_lines,
            reference_bands,
            display_points: row.display_points,
            downsample: row.downsample,
            x_format: row.x_format.map(TryInto::try_into).transpose()?,
            y_format: row.y_format.map(TryInto::try_into).transpose()?,
            x_axis: row.x_axis.map(TryInto::try_into).transpose()?,
        })
    }
}

impl Fft {
    pub fn validate(&self) -> Result<()> {
        for value in [self.sample_rate_hz, self.min_frequency_hz, self.max_frequency_hz].into_iter().flatten() {
            finite(value)?;
        }
        parse::<crate::document::FftWindow>(&self.window, "invalid fft window")?;
        parse::<crate::document::FftDetrend>(&self.detrend, "invalid fft detrend")?;
        parse::<crate::document::FrequencyLevelUnit>(&self.level_unit, "invalid level unit")?;
        Ok(())
    }
}

impl TryFrom<&crate::document::FftOptions> for Fft {
    type Error = Error;

    fn try_from(value: &crate::document::FftOptions) -> Result<Self> {
        for number in [value.sample_rate_hz, value.min_frequency_hz, value.max_frequency_hz].into_iter().flatten() {
            finite(number)?;
        }
        Ok(Self {
            sample_rate_hz: value.sample_rate_hz,
            window: name(&value.window),
            detrend: name(&value.detrend),
            level_unit: name(&value.level_unit),
            include_dc: value.include_dc,
            min_frequency_hz: value.min_frequency_hz,
            max_frequency_hz: value.max_frequency_hz,
        })
    }
}

impl TryFrom<Fft> for crate::document::FftOptions {
    type Error = Error;

    fn try_from(row: Fft) -> Result<Self> {
        row.validate()?;
        Ok(Self {
            sample_rate_hz: row.sample_rate_hz,
            window: parse(&row.window, "invalid fft window")?,
            detrend: parse(&row.detrend, "invalid fft detrend")?,
            level_unit: parse(&row.level_unit, "invalid level unit")?,
            include_dc: row.include_dc,
            min_frequency_hz: row.min_frequency_hz,
            max_frequency_hz: row.max_frequency_hz,
        })
    }
}

impl Interval {
    pub fn validate(&self) -> Result<()> {
        finite(self.bin_width)?;
        finite(self.sigma)?;
        Ok(())
    }
}

impl TryFrom<&crate::document::SigmaIntervalOptions> for Interval {
    type Error = Error;

    fn try_from(value: &crate::document::SigmaIntervalOptions) -> Result<Self> {
        finite(value.bin_width)?;
        finite(value.sigma)?;
        Ok(Self {
            bin_width: value.bin_width,
            sigma: value.sigma,
            clip_to_observed: value.clip_to_observed,
        })
    }
}

impl TryFrom<Interval> for crate::document::SigmaIntervalOptions {
    type Error = Error;

    fn try_from(row: Interval) -> Result<Self> {
        row.validate()?;
        Ok(Self {
            bin_width: row.bin_width,
            sigma: row.sigma,
            clip_to_observed: row.clip_to_observed,
        })
    }
}

impl TryFrom<&crate::document::HistogramOptions> for Histogram {
    type Error = Error;

    fn try_from(value: &crate::document::HistogramOptions) -> Result<Self> {
        Ok(Self {
            bins: u64::try_from(value.bins).map_err(|_| Error::Invalid("histogram bins overflow"))?,
            unit: value.unit.as_deref().map(str::to_owned),
        })
    }
}

impl TryFrom<Histogram> for crate::document::HistogramOptions {
    type Error = Error;

    fn try_from(row: Histogram) -> Result<Self> {
        Ok(Self {
            bins: usize::try_from(row.bins).map_err(|_| Error::Invalid("histogram bins overflow"))?,
            unit: row.unit,
        })
    }
}

impl TryFrom<&crate::document::ReferenceLine> for Line {
    type Error = Error;

    fn try_from(value: &crate::document::ReferenceLine) -> Result<Self> {
        for number in value.value.into_iter().chain([value.source_scale]) {
            finite(number)?;
        }
        Ok(Self {
            axis: name(&value.axis),
            value: value.value,
            source: value.source.as_ref().map(Data::try_from).transpose()?,
            source_scale: value.source_scale,
            label: value.label.as_deref().map(str::to_owned),
            color: value.color.as_str().to_owned(),
        })
    }
}

impl TryFrom<Line> for crate::document::ReferenceLine {
    type Error = Error;

    fn try_from(row: Line) -> Result<Self> {
        for number in row.value.into_iter().chain([row.source_scale]) {
            finite(number)?;
        }
        Ok(Self {
            axis: parse(&row.axis, "invalid reference axis")?,
            value: row.value,
            source: row.source.map(TryInto::try_into).transpose()?,
            source_scale: row.source_scale,
            label: row.label,
            color: row.color,
        })
    }
}

impl TryFrom<&crate::document::ReferenceBand> for Band {
    type Error = Error;

    fn try_from(value: &crate::document::ReferenceBand) -> Result<Self> {
        for number in [value.min, value.max].into_iter().flatten() {
            finite(number)?;
        }
        Ok(Self {
            axis: name(&value.axis),
            min: value.min,
            max: value.max,
            center: value.center.as_ref().map(Data::try_from).transpose()?,
            tolerance: value.tolerance.as_ref().map(Data::try_from).transpose()?,
            label: value.label.as_deref().map(str::to_owned),
            color: value.color.as_str().to_owned(),
        })
    }
}

impl TryFrom<Band> for crate::document::ReferenceBand {
    type Error = Error;

    fn try_from(row: Band) -> Result<Self> {
        for number in [row.min, row.max].into_iter().flatten() {
            finite(number)?;
        }
        Ok(Self {
            axis: parse(&row.axis, "invalid reference axis")?,
            min: row.min,
            max: row.max,
            center: row.center.map(TryInto::try_into).transpose()?,
            tolerance: row.tolerance.map(TryInto::try_into).transpose()?,
            label: row.label,
            color: row.color,
        })
    }
}

impl Format {
    pub fn validate(&self) -> Result<()> {
        let kind = self.kind.as_str();
        let required = [
            (self.decimals.is_some(), kind == "fixed"),
            (self.significant_digits.is_some(), matches!(kind, "scientific" | "engineering" | "si_prefix")),
            (self.pattern.is_some(), kind == "absolute_time"),
            (self.unit.is_some(), matches!(kind, "elapsed_since_run" | "elapsed_since_cycle")),
        ];
        if !matches!(
            kind,
            "normal" | "fixed" | "scientific" | "engineering" | "si_prefix" | "absolute_time" | "elapsed_since_run" | "elapsed_since_cycle"
        ) || required.iter().any(|(present, expected)| present != expected)
            || (kind != "normal" && self.max_fraction_digits.is_some())
        {
            return Err(Error::Invalid("value format fields differ"));
        }
        for value in [self.max_fraction_digits, self.decimals, self.significant_digits].into_iter().flatten() {
            format_digits(value)?;
        }
        if let Some(unit) = &self.unit {
            parse::<dokime_domain::model::values::ElapsedUnit>(unit, "invalid elapsed unit")?;
        }
        Ok(())
    }
}

impl TryFrom<&dokime_domain::model::values::ValueFormat> for Format {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::values::ValueFormat) -> Result<Self> {
        use dokime_domain::model::values::ValueFormat;
        let mut format = Self {
            kind: format_name(value).to_owned(),
            ..Default::default()
        };
        match value {
            ValueFormat::Normal { max_fraction_digits } => format.max_fraction_digits = max_fraction_digits.map(u32::from),
            ValueFormat::Fixed { decimals } => format.decimals = Some(u32::from(*decimals)),
            ValueFormat::Scientific { significant_digits }
            | ValueFormat::Engineering { significant_digits }
            | ValueFormat::SiPrefix { significant_digits } => format.significant_digits = Some(u32::from(*significant_digits)),
            ValueFormat::AbsoluteTime { pattern } => format.pattern = Some(pattern.as_str().to_owned()),
            ValueFormat::ElapsedSinceRun { unit } | ValueFormat::ElapsedSinceCycle { unit } => format.unit = Some(name(unit)),
        }
        Ok(format)
    }
}

impl TryFrom<Format> for dokime_domain::model::values::ValueFormat {
    type Error = Error;

    fn try_from(row: Format) -> Result<Self> {
        row.validate()?;
        match row.kind.as_str() {
            "normal" => Ok(Self::Normal {
                max_fraction_digits: row.max_fraction_digits.map(format_digits).transpose()?,
            }),
            "fixed" => Ok(Self::Fixed {
                decimals: format_digits(row.decimals.unwrap())?,
            }),
            "scientific" => Ok(Self::Scientific {
                significant_digits: format_digits(row.significant_digits.unwrap())?,
            }),
            "engineering" => Ok(Self::Engineering {
                significant_digits: format_digits(row.significant_digits.unwrap())?,
            }),
            "si_prefix" => Ok(Self::SiPrefix {
                significant_digits: format_digits(row.significant_digits.unwrap())?,
            }),
            "absolute_time" => Ok(Self::AbsoluteTime { pattern: row.pattern.unwrap() }),
            "elapsed_since_run" => Ok(Self::ElapsedSinceRun {
                unit: parse(row.unit.as_deref().unwrap(), "invalid elapsed unit")?,
            }),
            "elapsed_since_cycle" => Ok(Self::ElapsedSinceCycle {
                unit: parse(row.unit.as_deref().unwrap(), "invalid elapsed unit")?,
            }),
            _ => Err(Error::Invalid("invalid value format")),
        }
    }
}

impl Axis {
    pub fn validate(&self) -> Result<()> {
        match (self.kind.as_str(), &self.unit) {
            ("wall_time", None) | ("elapsed", Some(_)) => Ok(()),
            _ => Err(Error::Invalid("plot axis fields differ")),
        }
    }
}

impl TryFrom<&crate::document::PlotAxisKind> for Axis {
    type Error = Error;

    fn try_from(value: &crate::document::PlotAxisKind) -> Result<Self> {
        use crate::document::PlotAxisKind;
        Ok(match value {
            PlotAxisKind::WallTime => Self { kind: "wall_time".into(), unit: None },
            PlotAxisKind::Elapsed { unit } => Self {
                kind: "elapsed".into(),
                unit: Some(unit.as_str().to_owned()),
            },
        })
    }
}

impl TryFrom<Axis> for crate::document::PlotAxisKind {
    type Error = Error;

    fn try_from(row: Axis) -> Result<Self> {
        row.validate()?;
        match row.kind.as_str() {
            "wall_time" => Ok(Self::WallTime),
            "elapsed" => Ok(Self::Elapsed { unit: row.unit.unwrap() }),
            _ => Err(Error::Invalid("invalid plot axis")),
        }
    }
}

impl TryFrom<&crate::document::TelemetrySummaryDefinition> for Summary {
    type Error = Error;

    fn try_from(value: &crate::document::TelemetrySummaryDefinition) -> Result<Self> {
        let mut groups = Vec::new();
        groups.try_reserve_exact(value.groups.len()).map_err(|_| Error::Invalid("summary groups allocation failed"))?;
        for group in &value.groups {
            groups.push(SummaryGroup::try_from(group)?);
        }
        Ok(Self { groups })
    }
}

impl TryFrom<Summary> for crate::document::TelemetrySummaryDefinition {
    type Error = Error;

    fn try_from(row: Summary) -> Result<Self> {
        let mut groups = Vec::new();
        groups.try_reserve_exact(row.groups.len()).map_err(|_| Error::Invalid("summary groups allocation failed"))?;
        for group in row.groups {
            groups.push(group.try_into()?);
        }
        Ok(Self { groups })
    }
}

impl TryFrom<&crate::document::TelemetrySummaryGroupDefinition> for SummaryGroup {
    type Error = Error;

    fn try_from(value: &crate::document::TelemetrySummaryGroupDefinition) -> Result<Self> {
        let mut tiles = Vec::new();
        tiles.try_reserve_exact(value.tiles.len()).map_err(|_| Error::Invalid("summary tiles allocation failed"))?;
        for tile in &value.tiles {
            tiles.push(Tile::try_from(tile)?);
        }
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            tiles,
        })
    }
}

impl TryFrom<SummaryGroup> for crate::document::TelemetrySummaryGroupDefinition {
    type Error = Error;

    fn try_from(row: SummaryGroup) -> Result<Self> {
        let mut tiles = Vec::new();
        tiles.try_reserve_exact(row.tiles.len()).map_err(|_| Error::Invalid("summary tiles allocation failed"))?;
        for tile in row.tiles {
            tiles.push(tile.try_into()?);
        }
        Ok(Self {
            metadata: row.metadata.ok_or(Error::Invalid("summary group metadata is missing"))?.try_into()?,
            tiles,
        })
    }
}

impl TryFrom<&crate::document::TelemetrySummaryTileDefinition> for Tile {
    type Error = Error;

    fn try_from(value: &crate::document::TelemetrySummaryTileDefinition) -> Result<Self> {
        let mut filters = Vec::new();
        filters.try_reserve_exact(value.filters.len()).map_err(|_| Error::Invalid("summary filters allocation failed"))?;
        for filter in &value.filters {
            filters.push(Filter::try_from(filter)?);
        }
        let mut conditions = Vec::new();
        conditions.try_reserve_exact(value.conditions.len()).map_err(|_| Error::Invalid("summary conditions allocation failed"))?;
        for condition in &value.conditions {
            conditions.push(TileCondition::try_from(condition)?);
        }
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            value: Data::try_from(&value.value)?,
            filters,
            stat: name(&value.stat),
            format: name(&value.format),
            presentation: name(&value.presentation),
            unit: value.unit.as_deref().map(str::to_owned),
            conditions,
        })
    }
}

impl TryFrom<Tile> for crate::document::TelemetrySummaryTileDefinition {
    type Error = Error;

    fn try_from(row: Tile) -> Result<Self> {
        if row.stat != "latest" {
            return Err(Error::Invalid("invalid summary statistic"));
        }
        let mut filters = Vec::new();
        filters.try_reserve_exact(row.filters.len()).map_err(|_| Error::Invalid("summary filters allocation failed"))?;
        for filter in row.filters {
            filters.push(filter.try_into()?);
        }
        let mut conditions = Vec::new();
        conditions.try_reserve_exact(row.conditions.len()).map_err(|_| Error::Invalid("summary conditions allocation failed"))?;
        for condition in row.conditions {
            conditions.push(condition.try_into()?);
        }
        Ok(Self {
            metadata: row.metadata.ok_or(Error::Invalid("summary tile metadata is missing"))?.try_into()?,
            value: row.value.try_into()?,
            filters,
            stat: crate::document::TelemetrySummaryStat::Latest,
            format: parse(&row.format, "invalid summary format")?,
            presentation: parse(&row.presentation, "invalid summary presentation")?,
            unit: row.unit,
            conditions,
        })
    }
}

impl TryFrom<&crate::document::TelemetrySummaryCondition> for TileCondition {
    type Error = Error;

    fn try_from(value: &crate::document::TelemetrySummaryCondition) -> Result<Self> {
        Ok(Self {
            operator: name(&value.operator),
            value: Value::try_from(&value.value)?,
            state: name(&value.state),
        })
    }
}

impl TryFrom<TileCondition> for crate::document::TelemetrySummaryCondition {
    type Error = Error;

    fn try_from(row: TileCondition) -> Result<Self> {
        Ok(Self {
            operator: parse(&row.operator, "invalid document filter operator")?,
            value: row.value.try_into()?,
            state: parse(&row.state, "invalid summary state")?,
        })
    }
}

impl TryFrom<&DataExpression> for Data {
    type Error = Error;

    fn try_from(value: &DataExpression) -> Result<Self> {
        let mut transforms = Vec::new();
        transforms.try_reserve_exact(value.transforms.len()).map_err(|_| Error::Invalid("data transforms allocation failed"))?;
        for transform in &value.transforms {
            transforms.push(Transform::try_from(transform)?);
        }
        Ok(Self {
            label: value.label.as_deref().map(str::to_owned),
            source: DataSource::try_from(&value.source)?,
            transforms,
        })
    }
}

impl TryFrom<Data> for DataExpression {
    type Error = Error;

    fn try_from(row: Data) -> Result<Self> {
        let mut transforms = Vec::new();
        transforms.try_reserve_exact(row.transforms.len()).map_err(|_| Error::Invalid("data transforms allocation failed"))?;
        for transform in row.transforms {
            transforms.push(transform.try_into()?);
        }
        Ok(Self {
            label: row.label,
            source: row.source.try_into()?,
            transforms,
        })
    }
}

impl DataSource {
    fn validate(&self) -> Result<()> {
        let kind = self.kind.as_str();
        let fields = [
            (self.stream.is_some(), kind == "telemetry"),
            (self.field.is_some(), kind == "telemetry"),
            (self.name.is_some(), matches!(kind, "parameter" | "dimension")),
            (self.record_type.is_some(), kind == "record"),
            (self.artifact.is_some(), kind == "artifact"),
        ];
        if !matches!(kind, "telemetry" | "parameter" | "dimension" | "record" | "artifact")
            || fields.iter().any(|(present, required)| present != required)
        {
            return Err(Error::Invalid("data source fields differ"));
        }
        Ok(())
    }
}

impl TryFrom<&DomainSource> for DataSource {
    type Error = Error;

    fn try_from(value: &DomainSource) -> Result<Self> {
        let mut source = Self::default();
        let kind = match value {
            DomainSource::Telemetry { stream, field } => {
                source.stream = Some(stream.as_str().to_owned());
                source.field = Some(field.as_str().to_owned());
                "telemetry"
            }
            DomainSource::Parameter { name } => {
                source.name = Some(name.as_str().to_owned());
                "parameter"
            }
            DomainSource::Dimension { name } => {
                source.name = Some(name.as_str().to_owned());
                "dimension"
            }
            DomainSource::Record { record_type } => {
                source.record_type = Some(record_type.as_str().to_owned());
                "record"
            }
            DomainSource::Artifact { artifact } => {
                source.artifact = Some(artifact.as_str().to_owned());
                "artifact"
            }
        };
        source.kind = kind.to_owned();
        Ok(source)
    }
}

impl TryFrom<DataSource> for DomainSource {
    type Error = Error;

    fn try_from(row: DataSource) -> Result<Self> {
        row.validate()?;
        match row.kind.as_str() {
            "telemetry" => Ok(Self::Telemetry {
                stream: row.stream.unwrap(),
                field: row.field.unwrap(),
            }),
            "parameter" => Ok(Self::Parameter { name: row.name.unwrap() }),
            "dimension" => Ok(Self::Dimension { name: row.name.unwrap() }),
            "record" => Ok(Self::Record { record_type: row.record_type.unwrap() }),
            "artifact" => Ok(Self::Artifact { artifact: row.artifact.unwrap() }),
            _ => Err(Error::Invalid("invalid data source")),
        }
    }
}

impl Transform {
    fn validate(&self) -> Result<()> {
        let kind = self.kind.as_str();
        let fields = [
            (self.previous_record_type.is_some(), kind == "time_between"),
            (self.name.is_some(), kind == "detail"),
            (self.aggregate.is_some(), kind == "aggregate"),
        ];
        if !matches!(kind, "aggregate" | "time_between" | "detail" | "per_case")
            || fields.iter().any(|(present, required)| present != required)
        {
            return Err(Error::Invalid("data transform fields differ"));
        }
        Ok(())
    }
}

impl TryFrom<&DataTransform> for Transform {
    type Error = Error;

    fn try_from(value: &DataTransform) -> Result<Self> {
        let mut transform = Self {
            kind: transform_name(value).to_owned(),
            ..Default::default()
        };
        match value {
            DataTransform::TimeBetween { previous_record_type } => {
                transform.previous_record_type = Some(previous_record_type.as_str().to_owned())
            }
            DataTransform::Detail { name } => transform.name = Some(name.as_str().to_owned()),
            DataTransform::Aggregate(aggregate) => transform.aggregate = Some(aggregate.name()),
            _ => {}
        }
        Ok(transform)
    }
}

impl TryFrom<Transform> for DataTransform {
    type Error = Error;

    fn try_from(row: Transform) -> Result<Self> {
        row.validate()?;
        match row.kind.as_str() {
            "aggregate" => row
                .aggregate
                .as_deref()
                .and_then(harmos::Aggregate::parse)
                .map(Self::Aggregate)
                .ok_or(Error::Invalid("invalid data transform aggregate")),
            "time_between" => Ok(Self::TimeBetween {
                previous_record_type: row.previous_record_type.unwrap(),
            }),
            "detail" => Ok(Self::Detail { name: row.name.unwrap() }),
            "per_case" => Ok(Self::PerCase),
            _ => Err(Error::Invalid("invalid data transform")),
        }
    }
}

impl TryFrom<&crate::document::DocumentFilter> for Filter {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentFilter) -> Result<Self> {
        Ok(Self {
            expression: Data::try_from(&value.expression)?,
            operator: name(&value.operator),
            value: value.value.as_ref().map(Value::try_from).transpose()?,
        })
    }
}

impl TryFrom<Filter> for crate::document::DocumentFilter {
    type Error = Error;

    fn try_from(row: Filter) -> Result<Self> {
        Ok(Self {
            expression: row.expression.try_into()?,
            operator: parse(&row.operator, "invalid document filter operator")?,
            value: row.value.map(TryInto::try_into).transpose()?,
        })
    }
}

impl TryFrom<&crate::document::DocumentSectionRepeat> for Repeat {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentSectionRepeat) -> Result<Self> {
        let mut filters = Vec::new();
        filters.try_reserve_exact(value.filters.len()).map_err(|_| Error::Invalid("document filters allocation failed"))?;
        for filter in &value.filters {
            filters.push(Filter::try_from(filter)?);
        }
        Ok(Self {
            value: Data::try_from(&value.value)?,
            label_template: value.label_template.as_str().to_owned(),
            id_template: value.id_template.as_deref().map(str::to_owned),
            order_by: value.order_by.as_ref().map(Data::try_from).transpose()?,
            filters,
        })
    }
}

impl TryFrom<Repeat> for crate::document::DocumentSectionRepeat {
    type Error = Error;

    fn try_from(row: Repeat) -> Result<Self> {
        let mut filters = Vec::new();
        filters.try_reserve_exact(row.filters.len()).map_err(|_| Error::Invalid("document filters allocation failed"))?;
        for filter in row.filters {
            filters.push(filter.try_into()?);
        }
        Ok(Self {
            value: row.value.try_into()?,
            label_template: row.label_template,
            id_template: row.id_template,
            order_by: row.order_by.map(TryInto::try_into).transpose()?,
            filters,
        })
    }
}

impl TryFrom<Span> for crate::document::ResponsiveSpan {
    type Error = Error;

    fn try_from(row: Span) -> Result<Self> {
        Ok(Self {
            base: span(row.base)?,
            sm: row.sm.map(span).transpose()?,
            md: row.md.map(span).transpose()?,
            lg: row.lg.map(span).transpose()?,
            xl: row.xl.map(span).transpose()?,
        })
    }
}

impl CardLayout {
    pub fn admit(value: &crate::document::DocumentCardLayout) -> Result<()> {
        for value in
            [Some(value.col_span.base), value.col_span.sm, value.col_span.md, value.col_span.lg, value.col_span.xl, Some(value.row_span)]
                .into_iter()
                .flatten()
        {
            span(u32::from(value))?;
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        for value in [Some(self.col_span.base), self.col_span.sm, self.col_span.md, self.col_span.lg, self.col_span.xl, Some(self.row_span)]
            .into_iter()
            .flatten()
        {
            span(value)?;
        }
        parse::<crate::document::DocumentCardDensity>(&self.density, "invalid document density")?;
        parse::<crate::document::DocumentCardRail>(&self.rail, "invalid document rail")?;
        Ok(())
    }
}

impl TryFrom<CardLayout> for crate::document::DocumentCardLayout {
    type Error = Error;

    fn try_from(row: CardLayout) -> Result<Self> {
        row.validate()?;
        Ok(Self {
            col_span: row.col_span.try_into()?,
            row_span: span(row.row_span)?,
            order: row.order,
            density: parse(&row.density, "invalid document density")?,
            rail: parse(&row.rail, "invalid document rail")?,
        })
    }
}

impl TryFrom<&crate::document::DocumentCardPlacement> for Placement {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentCardPlacement) -> Result<Self> {
        CardLayout::admit(&value.layout)?;
        Ok(Self {
            card_id: value.card_id.as_str().to_owned(),
            layout: value.layout.clone().into(),
        })
    }
}

impl TryFrom<Placement> for crate::document::DocumentCardPlacement {
    type Error = Error;

    fn try_from(row: Placement) -> Result<Self> {
        Ok(Self {
            card_id: dokime_domain::model::identity::Identifier::new(row.card_id)?,
            layout: row.layout.try_into()?,
        })
    }
}

impl Default for CardLayout {
    fn default() -> Self {
        crate::document::DocumentCardLayout::default().into()
    }
}

impl From<crate::document::DocumentCardLayout> for CardLayout {
    fn from(layout: crate::document::DocumentCardLayout) -> Self {
        Self {
            col_span: Span {
                base: u32::from(layout.col_span.base),
                sm: layout.col_span.sm.map(u32::from),
                md: layout.col_span.md.map(u32::from),
                lg: layout.col_span.lg.map(u32::from),
                xl: layout.col_span.xl.map(u32::from),
            },
            row_span: u32::from(layout.row_span),
            order: layout.order,
            density: name(&layout.density),
            rail: name(&layout.rail),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn encode_body(value: &crate::document::DocumentCardBody) -> Result<(String, Option<CardData>)> {
    use crate::document::DocumentCardBody;
    let data = match value {
        DocumentCardBody::Text { markdown } => Some(CardData {
            markdown: Some(markdown.as_str().to_owned()),
            ..Default::default()
        }),
        DocumentCardBody::TelemetrySummary { summary } => Some(CardData {
            summary: Some(Summary::try_from(summary)?),
            ..Default::default()
        }),
        DocumentCardBody::Plot { plot } => Some(CardData {
            plot: Some(Plot::try_from(plot.as_ref())?),
            ..Default::default()
        }),
        DocumentCardBody::Table { rows } => {
            let mut native = Vec::new();
            native.try_reserve_exact(rows.len()).map_err(|_| Error::Invalid("document table rows allocation failed"))?;
            for row in rows {
                native.push(Map::try_from(row)?);
            }
            Some(CardData { rows: native, ..Default::default() })
        }
        _ => None,
    };
    Ok((body_name(value).to_owned(), data))
}

fn body_name(value: &crate::document::DocumentCardBody) -> &'static str {
    use crate::document::DocumentCardBody;
    match value {
        DocumentCardBody::Text { .. } => "text",
        DocumentCardBody::TelemetrySummary { .. } => "telemetry_summary",
        DocumentCardBody::Plot { .. } => "plot",
        DocumentCardBody::Table { .. } => "table",
        DocumentCardBody::Steps => "steps",
        DocumentCardBody::Criteria => "criteria",
        DocumentCardBody::Traceability => "traceability",
        DocumentCardBody::Notes => "notes",
        DocumentCardBody::Records => "records",
        DocumentCardBody::Artifacts => "artifacts",
        DocumentCardBody::Media => "media",
        DocumentCardBody::Health => "health",
        DocumentCardBody::TelemetryContext => "telemetry_context",
    }
}

fn format_name(value: &dokime_domain::model::values::ValueFormat) -> &'static str {
    match value {
        ValueFormat::Normal { .. } => "normal",
        ValueFormat::Fixed { .. } => "fixed",
        ValueFormat::Scientific { .. } => "scientific",
        ValueFormat::Engineering { .. } => "engineering",
        ValueFormat::SiPrefix { .. } => "si_prefix",
        ValueFormat::AbsoluteTime { .. } => "absolute_time",
        ValueFormat::ElapsedSinceRun { .. } => "elapsed_since_run",
        ValueFormat::ElapsedSinceCycle { .. } => "elapsed_since_cycle",
    }
}

fn transform_name(value: &DataTransform) -> &'static str {
    match value {
        DataTransform::Aggregate(_) => "aggregate",
        DataTransform::TimeBetween { .. } => "time_between",
        DataTransform::Detail { .. } => "detail",
        DataTransform::PerCase => "per_case",
    }
}

fn name<T: serde::Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(name)) => name,
        _ => unreachable!("document enums serialize as plain strings"),
    }
}

fn parse<T: serde::de::DeserializeOwned>(value: &str, invalid: &'static str) -> Result<T> {
    T::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value)).map_err(|_| Error::Invalid(invalid))
}

fn body_shape(kind: &str, data: &Option<CardData>) -> Result<()> {
    match (kind, data) {
        ("text" | "telemetry_summary" | "plot" | "table", Some(data)) => {
            let fields = [
                (data.markdown.is_some(), kind == "text"),
                (data.summary.is_some(), kind == "telemetry_summary"),
                (data.plot.is_some(), kind == "plot"),
            ];
            if fields.iter().any(|(present, expected)| present != expected) || (kind != "table" && !data.rows.is_empty()) {
                return Err(Error::Invalid("document card body fields differ"));
            }
        }
        ("steps" | "criteria" | "traceability" | "notes" | "records" | "artifacts" | "media" | "health" | "telemetry_context", None) => {}
        _ => return Err(Error::Invalid("document card body kind or data differs")),
    }
    Ok(())
}

fn consume_body(kind: String, data: Option<CardData>) -> Result<crate::document::DocumentCardBody> {
    use crate::document::DocumentCardBody;
    body_shape(&kind, &data)?;
    Ok(match kind.as_str() {
        "text" => DocumentCardBody::Text {
            markdown: data.unwrap().markdown.unwrap(),
        },
        "telemetry_summary" => DocumentCardBody::TelemetrySummary {
            summary: data.unwrap().summary.unwrap().try_into()?,
        },
        "plot" => DocumentCardBody::Plot {
            plot: Box::new(data.unwrap().plot.unwrap().try_into()?),
        },
        "table" => {
            let data = data.unwrap();
            let mut rows = Vec::new();
            rows.try_reserve_exact(data.rows.len()).map_err(|_| Error::Invalid("document table rows allocation failed"))?;
            for row in data.rows {
                rows.push(row.try_into()?);
            }
            DocumentCardBody::Table { rows }
        }
        "steps" => DocumentCardBody::Steps,
        "criteria" => DocumentCardBody::Criteria,
        "traceability" => DocumentCardBody::Traceability,
        "notes" => DocumentCardBody::Notes,
        "records" => DocumentCardBody::Records,
        "artifacts" => DocumentCardBody::Artifacts,
        "media" => DocumentCardBody::Media,
        "health" => DocumentCardBody::Health,
        "telemetry_context" => DocumentCardBody::TelemetryContext,
        _ => return Err(Error::Invalid("invalid document card body")),
    })
}

fn finite(value: f64) -> Result<()> {
    if value.is_finite() { Ok(()) } else { Err(Error::Invalid("plot number is nonfinite")) }
}

fn format_digits(value: u32) -> Result<u8> {
    u8::try_from(value).map_err(|_| Error::Invalid("value format digits exceed byte range"))
}

fn span(value: u32) -> Result<u8> {
    u8::try_from(value).map_err(|_| Error::Invalid("document span exceeds byte range"))
}

/* === Tests === */
