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
    stream: Option<String>,
    #[serde(default)]
    field: Option<String>,
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
    live_window_s: Option<f64>,
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
impl Document {
    pub fn encoding_memory(value: &crate::document::DocumentDefinition) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let layout = layout::Layout::encoding_memory(&value.layout)?;
        if [metadata, layout].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(value.document_id.as_str().len())
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>()))
            .saturating_add(layout.saturating_sub(std::mem::size_of::<layout::Layout>()));
        for card in &value.cards {
            bytes = bytes.saturating_add(CardDefinition::encoding_memory(card)?);
        }
        for section in &value.sections {
            bytes = bytes.saturating_add(Section::encoding_memory(section)?);
        }
        Ok(bytes)
    }

    pub fn projection_memory(&self) -> Result<usize> {
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("document metadata is missing"))?.projection_memory()?;
        let mut bytes = std::mem::size_of::<crate::document::DocumentDefinition>()
            .saturating_add(self.document_id.capacity())
            .saturating_add(metadata)
            .saturating_add(self.layout.projection_memory()?);
        for card in &self.cards {
            bytes = bytes.saturating_add(card.projection_memory()?);
        }
        for section in &self.sections {
            bytes = bytes.saturating_add(section.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::document::DocumentDefinition> for Document {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentDefinition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("document encoding memory overflows"));
        }
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
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("document projection memory overflows"));
        }
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

impl CardDefinition {
    fn encoding_memory(value: &crate::document::DocumentCardDefinition) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let body = body_encoding_memory(&value.body)?;
        if [metadata, body].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let mut bytes =
            std::mem::size_of::<Self>().saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>())).saturating_add(body);
        for filter in &value.filters {
            bytes = bytes.saturating_add(Filter::encoding_memory(filter)?);
        }
        Ok(bytes)
    }

    fn projection_memory(&self) -> Result<usize> {
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("document card metadata is missing"))?.projection_memory()?;
        let mut bytes = std::mem::size_of::<crate::document::DocumentCardDefinition>()
            .saturating_add(metadata)
            .saturating_add(body_memory(&self.kind, &self.data)?);
        for filter in &self.filters {
            bytes = bytes.saturating_add(filter.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::document::DocumentCardDefinition> for CardDefinition {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentCardDefinition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("document card encoding memory overflows"));
        }
        let mut filters = Vec::new();
        filters.try_reserve_exact(value.filters.len()).map_err(|_| Error::Invalid("document filters allocation failed"))?;
        for filter in &value.filters {
            filters.push(Filter::try_from(filter)?);
        }
        let (kind, data) = encode_body(&value.body)?;
        Ok(Self { metadata: Some(metadata::Row::try_from(&value.metadata)?), filters, kind, data })
    }
}

impl TryFrom<CardDefinition> for crate::document::DocumentCardDefinition {
    type Error = Error;

    fn try_from(row: CardDefinition) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("document card definition projection memory overflows"));
        }
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

impl Card {
    fn encoding_memory(value: &crate::document::DocumentCard) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let layout = CardLayout::encoding_memory(&value.layout)?;
        let body = body_encoding_memory(&value.body)?;
        if [metadata, layout, body].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>()))
            .saturating_add(layout.saturating_sub(std::mem::size_of::<CardLayout>()))
            .saturating_add(body);
        for filter in &value.filters {
            bytes = bytes.saturating_add(Filter::encoding_memory(filter)?);
        }
        Ok(bytes)
    }

    fn projection_memory(&self) -> Result<usize> {
        self.layout.validate()?;
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("document card metadata is missing"))?.projection_memory()?;
        let mut bytes = std::mem::size_of::<crate::document::DocumentCard>()
            .saturating_add(metadata)
            .saturating_add(body_memory(&self.kind, &self.data)?);
        for filter in &self.filters {
            bytes = bytes.saturating_add(filter.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::document::DocumentCard> for Card {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentCard) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("document card encoding memory overflows"));
        }
        let mut filters = Vec::new();
        filters.try_reserve_exact(value.filters.len()).map_err(|_| Error::Invalid("document filters allocation failed"))?;
        for filter in &value.filters {
            filters.push(Filter::try_from(filter)?);
        }
        let (kind, data) = encode_body(&value.body)?;
        Ok(Self { metadata: Some(metadata::Row::try_from(&value.metadata)?), layout: value.layout.clone().into(), filters, kind, data })
    }
}

impl TryFrom<Card> for crate::document::DocumentCard {
    type Error = Error;

    fn try_from(row: Card) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("document card projection memory overflows"));
        }
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

impl Section {
    fn encoding_memory(value: &crate::document::DocumentSection) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let repeat = value.repeat.as_ref().map(Repeat::encoding_memory).transpose()?.unwrap_or(0);
        if [metadata, repeat].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>()))
            .saturating_add(repeat.saturating_sub(std::mem::size_of::<Repeat>()));
        for card in &value.cards {
            bytes = bytes.saturating_add(Card::encoding_memory(card)?);
        }
        Ok(bytes)
    }

    fn projection_memory(&self) -> Result<usize> {
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("document section metadata is missing"))?.projection_memory()?;
        let repeat = self.repeat.as_ref().map(Repeat::projection_memory).transpose()?.unwrap_or(0);
        let mut bytes = std::mem::size_of::<crate::document::DocumentSection>().saturating_add(metadata).saturating_add(repeat);
        for card in &self.cards {
            bytes = bytes.saturating_add(card.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::document::DocumentSection> for Section {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentSection) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("document section encoding memory overflows"));
        }
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
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("document section projection memory overflows"));
        }
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

impl Plot {
    fn encoding_memory(value: &crate::document::PlotDefinition) -> Result<usize> {
        if let Some(window) = value.live_window_s {
            finite(window)?;
        }
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let fft = Fft::encoding_memory(&value.fft)?;
        Interval::encoding_memory(&value.interval)?;
        let histogram = Histogram::encoding_memory(&value.histogram)?;
        if [metadata, fft, histogram].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            metadata.saturating_sub(std::mem::size_of::<metadata::Row>()),
            plot_name(&value.plot_type).len(),
            style_name(&value.style).len(),
            scale_name(&value.x_scale).len(),
            scale_name(&value.y_scale).len(),
            fft.saturating_sub(std::mem::size_of::<Fft>()),
            histogram.saturating_sub(std::mem::size_of::<Histogram>()),
            value.y.len().saturating_mul(std::mem::size_of::<String>()),
            value.marker_record_types.len().saturating_mul(std::mem::size_of::<String>()),
        ];
        let mut bytes = fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add);
        for text in [&value.stream, &value.x, &value.z_stream, &value.z_field, &value.series_by, &value.unit].into_iter().flatten() {
            bytes = bytes.saturating_add(text.len());
        }
        for text in value.y.iter().chain(&value.marker_record_types) {
            bytes = bytes.saturating_add(text.len());
        }
        for expression in [&value.x_expression, &value.z_expression, &value.series_by_expression].into_iter().flatten() {
            let memory = Data::encoding_memory(expression)?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(memory.saturating_sub(std::mem::size_of::<Data>()));
        }
        for expression in &value.y_expressions {
            bytes = bytes.saturating_add(Data::encoding_memory(expression)?);
        }
        for filter in &value.filters {
            bytes = bytes.saturating_add(Filter::encoding_memory(filter)?);
        }
        for line in &value.reference_lines {
            bytes = bytes.saturating_add(Line::encoding_memory(line)?);
        }
        for band in &value.reference_bands {
            bytes = bytes.saturating_add(Band::encoding_memory(band)?);
        }
        for format in [&value.x_format, &value.y_format].into_iter().flatten() {
            let memory = Format::encoding_memory(format);
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(memory.saturating_sub(std::mem::size_of::<Format>()));
        }
        if let Some(axis) = &value.x_axis {
            let memory = Axis::encoding_memory(axis);
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(memory.saturating_sub(std::mem::size_of::<Axis>()));
        }
        if let Some(analysis) = &value.analysis {
            bytes = bytes.saturating_add(super::analysis::Selection::encoding_memory(analysis)?);
        }
        Ok(bytes)
    }

    fn projection_memory(&self) -> Result<usize> {
        use harmos::MemorySize;
        plot_type(&self.plot_type)?;
        plot_style(&self.style)?;
        axis_scale(&self.x_scale)?;
        axis_scale(&self.y_scale)?;
        self.fft.validate()?;
        self.interval.validate()?;
        if let Some(window) = self.live_window_s {
            finite(window)?;
        }
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("plot metadata is missing"))?.projection_memory()?;
        let strings = [&self.stream, &self.x, &self.z_stream, &self.z_field, &self.series_by, &self.unit];
        let mut bytes = strings
            .into_iter()
            .fold(std::mem::size_of::<crate::document::PlotDefinition>(), |bytes, value| {
                bytes.saturating_add(value.as_ref().map_or(0, String::capacity))
            })
            .saturating_add(metadata);
        let y = self.y.memory_size();
        let markers = self.marker_record_types.memory_size();
        bytes = bytes.saturating_add(y).saturating_add(markers).saturating_add(self.histogram.projection_memory()?);
        for value in [&self.x_expression, &self.z_expression, &self.series_by_expression].into_iter().flatten() {
            bytes = bytes.saturating_add(value.projection_memory()?);
        }
        for value in &self.y_expressions {
            bytes = bytes.saturating_add(value.projection_memory()?);
        }
        for filter in &self.filters {
            bytes = bytes.saturating_add(filter.projection_memory()?);
        }
        for line in &self.reference_lines {
            bytes = bytes.saturating_add(line.projection_memory()?);
        }
        for band in &self.reference_bands {
            bytes = bytes.saturating_add(band.projection_memory()?);
        }
        for format in [&self.x_format, &self.y_format].into_iter().flatten() {
            bytes = bytes.saturating_add(format.projection_memory()?);
        }
        if let Some(axis) = &self.x_axis {
            bytes = bytes.saturating_add(axis.projection_memory()?);
        }
        if let Some(analysis) = &self.analysis {
            bytes = bytes.saturating_add(analysis.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::document::PlotDefinition> for Plot {
    type Error = Error;

    fn try_from(value: &crate::document::PlotDefinition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("plot encoding memory overflows"));
        }
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
            plot_type: plot_name(&value.plot_type).to_owned(),
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
            style: style_name(&value.style).to_owned(),
            x_scale: scale_name(&value.x_scale).to_owned(),
            y_scale: scale_name(&value.y_scale).to_owned(),
            fft: Fft::try_from(&value.fft)?,
            analysis: value.analysis.as_ref().map(super::analysis::Selection::try_from).transpose()?,
            interval: Interval::try_from(&value.interval)?,
            histogram: Histogram::try_from(&value.histogram)?,
            marker_record_types: super::copied_strings(&value.marker_record_types)?,
            reference_lines,
            reference_bands,
            live_window_s: value.live_window_s,
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
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("plot projection memory overflows"));
        }
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
            plot_type: plot_type(&row.plot_type)?,
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
            style: plot_style(&row.style)?,
            x_scale: axis_scale(&row.x_scale)?,
            y_scale: axis_scale(&row.y_scale)?,
            fft: row.fft.try_into()?,
            analysis: row.analysis.map(TryInto::try_into).transpose()?,
            interval: row.interval.try_into()?,
            histogram: row.histogram.try_into()?,
            marker_record_types: row.marker_record_types,
            reference_lines,
            reference_bands,
            live_window_s: row.live_window_s,
            display_points: row.display_points,
            downsample: row.downsample,
            x_format: row.x_format.map(TryInto::try_into).transpose()?,
            y_format: row.y_format.map(TryInto::try_into).transpose()?,
            x_axis: row.x_axis.map(TryInto::try_into).transpose()?,
        })
    }
}

impl Fft {
    pub fn encoding_memory(value: &crate::document::FftOptions) -> Result<usize> {
        for number in [value.sample_rate_hz, value.min_frequency_hz, value.max_frequency_hz].into_iter().flatten() {
            finite(number)?;
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(fft_window_name(&value.window).len())
            .saturating_add(fft_detrend_name(&value.detrend).len())
            .saturating_add(level_unit_name(&value.level_unit).len()))
    }

    pub fn validate(&self) -> Result<()> {
        for value in [self.sample_rate_hz, self.min_frequency_hz, self.max_frequency_hz].into_iter().flatten() {
            finite(value)?;
        }
        fft_window(&self.window)?;
        fft_detrend(&self.detrend)?;
        level_unit(&self.level_unit)?;
        Ok(())
    }
}

impl TryFrom<&crate::document::FftOptions> for Fft {
    type Error = Error;

    fn try_from(value: &crate::document::FftOptions) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("fft encoding memory overflows"));
        }
        Ok(Self {
            sample_rate_hz: value.sample_rate_hz,
            window: fft_window_name(&value.window).to_owned(),
            detrend: fft_detrend_name(&value.detrend).to_owned(),
            level_unit: level_unit_name(&value.level_unit).to_owned(),
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
            window: fft_window(&row.window)?,
            detrend: fft_detrend(&row.detrend)?,
            level_unit: level_unit(&row.level_unit)?,
            include_dc: row.include_dc,
            min_frequency_hz: row.min_frequency_hz,
            max_frequency_hz: row.max_frequency_hz,
        })
    }
}

impl Interval {
    pub fn encoding_memory(value: &crate::document::SigmaIntervalOptions) -> Result<usize> {
        finite(value.bin_width)?;
        finite(value.sigma)?;
        Ok(std::mem::size_of::<Self>())
    }

    pub fn validate(&self) -> Result<()> {
        finite(self.bin_width)?;
        finite(self.sigma)?;
        Ok(())
    }
}

impl TryFrom<&crate::document::SigmaIntervalOptions> for Interval {
    type Error = Error;

    fn try_from(value: &crate::document::SigmaIntervalOptions) -> Result<Self> {
        Self::encoding_memory(value)?;
        Ok(Self { bin_width: value.bin_width, sigma: value.sigma, clip_to_observed: value.clip_to_observed })
    }
}

impl TryFrom<Interval> for crate::document::SigmaIntervalOptions {
    type Error = Error;

    fn try_from(row: Interval) -> Result<Self> {
        row.validate()?;
        Ok(Self { bin_width: row.bin_width, sigma: row.sigma, clip_to_observed: row.clip_to_observed })
    }
}

impl Histogram {
    pub fn encoding_memory(value: &crate::document::HistogramOptions) -> Result<usize> {
        u64::try_from(value.bins).map_err(|_| Error::Invalid("histogram bins overflow"))?;
        Ok(std::mem::size_of::<Self>().saturating_add(value.unit.as_deref().map_or(0, str::len)))
    }

    pub fn projection_memory(&self) -> Result<usize> {
        usize::try_from(self.bins).map_err(|_| Error::Invalid("histogram bins overflow"))?;
        Ok(std::mem::size_of::<crate::document::HistogramOptions>().saturating_add(self.unit.as_ref().map_or(0, String::capacity)))
    }
}

impl TryFrom<&crate::document::HistogramOptions> for Histogram {
    type Error = Error;

    fn try_from(value: &crate::document::HistogramOptions) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("histogram encoding memory overflows"));
        }
        Ok(Self {
            bins: u64::try_from(value.bins).map_err(|_| Error::Invalid("histogram bins overflow"))?,
            unit: value.unit.as_deref().map(str::to_owned),
        })
    }
}

impl TryFrom<Histogram> for crate::document::HistogramOptions {
    type Error = Error;

    fn try_from(row: Histogram) -> Result<Self> {
        Ok(Self { bins: usize::try_from(row.bins).map_err(|_| Error::Invalid("histogram bins overflow"))?, unit: row.unit })
    }
}

impl Line {
    pub fn encoding_memory(value: &crate::document::ReferenceLine) -> Result<usize> {
        if let Some(number) = value.value {
            finite(number)?;
        }
        finite(value.source_scale)?;
        let source = value.source.as_ref().map(Data::encoding_memory).transpose()?.unwrap_or(0);
        if source == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(reference_axis_name(&value.axis).len())
            .saturating_add(source.saturating_sub(std::mem::size_of::<Data>()))
            .saturating_add(value.label.as_deref().map_or(0, str::len))
            .saturating_add(value.color.len()))
    }

    pub fn projection_memory(&self) -> Result<usize> {
        reference_axis(&self.axis)?;
        if let Some(value) = self.value {
            finite(value)?;
        }
        finite(self.source_scale)?;
        let source = self.source.as_ref().map(Data::projection_memory).transpose()?.unwrap_or(0);
        Ok(std::mem::size_of::<crate::document::ReferenceLine>()
            .saturating_add(source)
            .saturating_add(self.label.as_ref().map_or(0, String::capacity))
            .saturating_add(self.color.capacity()))
    }
}

impl TryFrom<&crate::document::ReferenceLine> for Line {
    type Error = Error;

    fn try_from(value: &crate::document::ReferenceLine) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("reference line encoding memory overflows"));
        }
        Ok(Self {
            axis: reference_axis_name(&value.axis).to_owned(),
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
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("reference line projection memory overflows"));
        }
        Ok(Self {
            axis: reference_axis(&row.axis)?,
            value: row.value,
            source: row.source.map(TryInto::try_into).transpose()?,
            source_scale: row.source_scale,
            label: row.label,
            color: row.color,
        })
    }
}

impl Band {
    pub fn encoding_memory(value: &crate::document::ReferenceBand) -> Result<usize> {
        for number in [value.min, value.max].into_iter().flatten() {
            finite(number)?;
        }
        let center = value.center.as_ref().map(Data::encoding_memory).transpose()?.unwrap_or(0);
        let tolerance = value.tolerance.as_ref().map(Data::encoding_memory).transpose()?.unwrap_or(0);
        if [center, tolerance].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(reference_axis_name(&value.axis).len())
            .saturating_add(center.saturating_sub(std::mem::size_of::<Data>()))
            .saturating_add(tolerance.saturating_sub(std::mem::size_of::<Data>()))
            .saturating_add(value.label.as_deref().map_or(0, str::len))
            .saturating_add(value.color.len()))
    }

    pub fn projection_memory(&self) -> Result<usize> {
        reference_axis(&self.axis)?;
        for value in [self.min, self.max].into_iter().flatten() {
            finite(value)?;
        }
        let center = self.center.as_ref().map(Data::projection_memory).transpose()?.unwrap_or(0);
        let tolerance = self.tolerance.as_ref().map(Data::projection_memory).transpose()?.unwrap_or(0);
        Ok(std::mem::size_of::<crate::document::ReferenceBand>()
            .saturating_add(center)
            .saturating_add(tolerance)
            .saturating_add(self.label.as_ref().map_or(0, String::capacity))
            .saturating_add(self.color.capacity()))
    }
}

impl TryFrom<&crate::document::ReferenceBand> for Band {
    type Error = Error;

    fn try_from(value: &crate::document::ReferenceBand) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("reference band encoding memory overflows"));
        }
        Ok(Self {
            axis: reference_axis_name(&value.axis).to_owned(),
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
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("reference band projection memory overflows"));
        }
        Ok(Self {
            axis: reference_axis(&row.axis)?,
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
    pub fn encoding_memory(value: &dokime_domain::model::values::ValueFormat) -> usize {
        use dokime_domain::model::values::ValueFormat;
        let fields = match value {
            ValueFormat::AbsoluteTime { pattern } => pattern.len(),
            ValueFormat::ElapsedSinceRun { unit } | ValueFormat::ElapsedSinceCycle { unit } => elapsed_unit_name(unit).len(),
            _ => 0,
        };
        std::mem::size_of::<Self>().saturating_add(format_name(value).len()).saturating_add(fields)
    }

    pub fn projection_memory(&self) -> Result<usize> {
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
            elapsed_unit(unit)?;
        }
        Ok(std::mem::size_of::<dokime_domain::model::values::ValueFormat>()
            .saturating_add(self.pattern.as_ref().map_or(0, String::capacity)))
    }
}

impl TryFrom<&dokime_domain::model::values::ValueFormat> for Format {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::values::ValueFormat) -> Result<Self> {
        use dokime_domain::model::values::ValueFormat;
        if Self::encoding_memory(value) == usize::MAX {
            return Err(Error::Invalid("value format encoding memory overflows"));
        }
        let mut format = Self { kind: format_name(value).to_owned(), ..Default::default() };
        match value {
            ValueFormat::Normal { max_fraction_digits } => format.max_fraction_digits = max_fraction_digits.map(u32::from),
            ValueFormat::Fixed { decimals } => format.decimals = Some(u32::from(*decimals)),
            ValueFormat::Scientific { significant_digits }
            | ValueFormat::Engineering { significant_digits }
            | ValueFormat::SiPrefix { significant_digits } => format.significant_digits = Some(u32::from(*significant_digits)),
            ValueFormat::AbsoluteTime { pattern } => format.pattern = Some(pattern.as_str().to_owned()),
            ValueFormat::ElapsedSinceRun { unit } | ValueFormat::ElapsedSinceCycle { unit } => {
                format.unit = Some(elapsed_unit_name(unit).to_owned())
            }
        }
        Ok(format)
    }
}

impl TryFrom<Format> for dokime_domain::model::values::ValueFormat {
    type Error = Error;

    fn try_from(row: Format) -> Result<Self> {
        row.projection_memory()?;
        match row.kind.as_str() {
            "normal" => Ok(Self::Normal { max_fraction_digits: row.max_fraction_digits.map(format_digits).transpose()? }),
            "fixed" => Ok(Self::Fixed { decimals: format_digits(row.decimals.unwrap())? }),
            "scientific" => Ok(Self::Scientific { significant_digits: format_digits(row.significant_digits.unwrap())? }),
            "engineering" => Ok(Self::Engineering { significant_digits: format_digits(row.significant_digits.unwrap())? }),
            "si_prefix" => Ok(Self::SiPrefix { significant_digits: format_digits(row.significant_digits.unwrap())? }),
            "absolute_time" => Ok(Self::AbsoluteTime { pattern: row.pattern.unwrap() }),
            "elapsed_since_run" => Ok(Self::ElapsedSinceRun { unit: elapsed_unit(row.unit.as_deref().unwrap())? }),
            "elapsed_since_cycle" => Ok(Self::ElapsedSinceCycle { unit: elapsed_unit(row.unit.as_deref().unwrap())? }),
            _ => Err(Error::Invalid("invalid value format")),
        }
    }
}

impl Axis {
    pub fn encoding_memory(value: &crate::document::PlotAxisKind) -> usize {
        use crate::document::PlotAxisKind;
        let (kind, unit) = match value {
            PlotAxisKind::WallTime => ("wall_time", 0),
            PlotAxisKind::Elapsed { unit } => ("elapsed", unit.len()),
        };
        std::mem::size_of::<Self>().saturating_add(kind.len()).saturating_add(unit)
    }

    pub fn projection_memory(&self) -> Result<usize> {
        match (self.kind.as_str(), &self.unit) {
            ("wall_time", None) => Ok(std::mem::size_of::<crate::document::PlotAxisKind>()),
            ("elapsed", Some(unit)) => Ok(std::mem::size_of::<crate::document::PlotAxisKind>().saturating_add(unit.capacity())),
            _ => Err(Error::Invalid("plot axis fields differ")),
        }
    }
}

impl TryFrom<&crate::document::PlotAxisKind> for Axis {
    type Error = Error;

    fn try_from(value: &crate::document::PlotAxisKind) -> Result<Self> {
        use crate::document::PlotAxisKind;
        if Self::encoding_memory(value) == usize::MAX {
            return Err(Error::Invalid("plot axis encoding memory overflows"));
        }
        Ok(match value {
            PlotAxisKind::WallTime => Self { kind: "wall_time".into(), unit: None },
            PlotAxisKind::Elapsed { unit } => Self { kind: "elapsed".into(), unit: Some(unit.as_str().to_owned()) },
        })
    }
}

impl TryFrom<Axis> for crate::document::PlotAxisKind {
    type Error = Error;

    fn try_from(row: Axis) -> Result<Self> {
        row.projection_memory()?;
        match row.kind.as_str() {
            "wall_time" => Ok(Self::WallTime),
            "elapsed" => Ok(Self::Elapsed { unit: row.unit.unwrap() }),
            _ => Err(Error::Invalid("invalid plot axis")),
        }
    }
}

impl Summary {
    fn encoding_memory(value: &crate::document::TelemetrySummaryDefinition) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>();
        for group in &value.groups {
            bytes = bytes.saturating_add(SummaryGroup::encoding_memory(group)?);
        }
        Ok(bytes)
    }

    fn projection_memory(&self) -> Result<usize> {
        let mut bytes = std::mem::size_of::<crate::document::TelemetrySummaryDefinition>();
        for group in &self.groups {
            bytes = bytes.saturating_add(group.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::document::TelemetrySummaryDefinition> for Summary {
    type Error = Error;

    fn try_from(value: &crate::document::TelemetrySummaryDefinition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("summary encoding memory overflows"));
        }
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
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("summary projection memory overflows"));
        }
        let mut groups = Vec::new();
        groups.try_reserve_exact(row.groups.len()).map_err(|_| Error::Invalid("summary groups allocation failed"))?;
        for group in row.groups {
            groups.push(group.try_into()?);
        }
        Ok(Self { groups })
    }
}

impl SummaryGroup {
    fn encoding_memory(value: &crate::document::TelemetrySummaryGroupDefinition) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        if metadata == usize::MAX {
            return Ok(usize::MAX);
        }
        let mut bytes = std::mem::size_of::<Self>().saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>()));
        for tile in &value.tiles {
            bytes = bytes.saturating_add(Tile::encoding_memory(tile)?);
        }
        Ok(bytes)
    }

    fn projection_memory(&self) -> Result<usize> {
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("summary group metadata is missing"))?.projection_memory()?;
        let mut bytes = std::mem::size_of::<crate::document::TelemetrySummaryGroupDefinition>().saturating_add(metadata);
        for tile in &self.tiles {
            bytes = bytes.saturating_add(tile.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::document::TelemetrySummaryGroupDefinition> for SummaryGroup {
    type Error = Error;

    fn try_from(value: &crate::document::TelemetrySummaryGroupDefinition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("summary group encoding memory overflows"));
        }
        let mut tiles = Vec::new();
        tiles.try_reserve_exact(value.tiles.len()).map_err(|_| Error::Invalid("summary tiles allocation failed"))?;
        for tile in &value.tiles {
            tiles.push(Tile::try_from(tile)?);
        }
        Ok(Self { metadata: Some(metadata::Row::try_from(&value.metadata)?), tiles })
    }
}

impl TryFrom<SummaryGroup> for crate::document::TelemetrySummaryGroupDefinition {
    type Error = Error;

    fn try_from(row: SummaryGroup) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("summary group projection memory overflows"));
        }
        let mut tiles = Vec::new();
        tiles.try_reserve_exact(row.tiles.len()).map_err(|_| Error::Invalid("summary tiles allocation failed"))?;
        for tile in row.tiles {
            tiles.push(tile.try_into()?);
        }
        Ok(Self { metadata: row.metadata.ok_or(Error::Invalid("summary group metadata is missing"))?.try_into()?, tiles })
    }
}

impl Tile {
    fn encoding_memory(value: &crate::document::TelemetrySummaryTileDefinition) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let scalar = Data::encoding_memory(&value.value)?;
        if [metadata, scalar].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            metadata.saturating_sub(std::mem::size_of::<metadata::Row>()),
            scalar.saturating_sub(std::mem::size_of::<Data>()),
            summary_stat_name(&value.stat).len(),
            summary_format_name(&value.format).len(),
            summary_presentation_name(&value.presentation).len(),
            value.unit.as_deref().map_or(0, str::len),
        ];
        let mut bytes = fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add);
        for filter in &value.filters {
            bytes = bytes.saturating_add(Filter::encoding_memory(filter)?);
        }
        for condition in &value.conditions {
            bytes = bytes.saturating_add(TileCondition::encoding_memory(condition)?);
        }
        Ok(bytes)
    }

    fn projection_memory(&self) -> Result<usize> {
        if self.stat != "latest" {
            return Err(Error::Invalid("invalid summary statistic"));
        }
        summary_format(&self.format)?;
        summary_presentation(&self.presentation)?;
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("summary tile metadata is missing"))?.projection_memory()?;
        let mut bytes = std::mem::size_of::<crate::document::TelemetrySummaryTileDefinition>()
            .saturating_add(metadata)
            .saturating_add(self.value.projection_memory()?)
            .saturating_add(self.unit.as_ref().map_or(0, String::capacity));
        for filter in &self.filters {
            bytes = bytes.saturating_add(filter.projection_memory()?);
        }
        for condition in &self.conditions {
            bytes = bytes.saturating_add(condition.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::document::TelemetrySummaryTileDefinition> for Tile {
    type Error = Error;

    fn try_from(value: &crate::document::TelemetrySummaryTileDefinition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("summary tile encoding memory overflows"));
        }
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
            stat: summary_stat_name(&value.stat).to_owned(),
            format: summary_format_name(&value.format).to_owned(),
            presentation: summary_presentation_name(&value.presentation).to_owned(),
            unit: value.unit.as_deref().map(str::to_owned),
            conditions,
        })
    }
}

impl TryFrom<Tile> for crate::document::TelemetrySummaryTileDefinition {
    type Error = Error;

    fn try_from(row: Tile) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("summary tile projection memory overflows"));
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
            format: summary_format(&row.format)?,
            presentation: summary_presentation(&row.presentation)?,
            unit: row.unit,
            conditions,
        })
    }
}

impl TileCondition {
    fn encoding_memory(value: &crate::document::TelemetrySummaryCondition) -> Result<usize> {
        let scalar = Value::encoding_memory(&value.value)?;
        if scalar == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(filter_name(&value.operator).len())
            .saturating_add(summary_state_name(&value.state).len())
            .saturating_add(scalar.saturating_sub(std::mem::size_of::<Value>())))
    }

    fn projection_memory(&self) -> Result<usize> {
        filter_operator(&self.operator)?;
        summary_state(&self.state)?;
        Ok(std::mem::size_of::<crate::document::TelemetrySummaryCondition>().saturating_add(self.value.projection_memory()?))
    }
}

impl TryFrom<&crate::document::TelemetrySummaryCondition> for TileCondition {
    type Error = Error;

    fn try_from(value: &crate::document::TelemetrySummaryCondition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("summary condition encoding memory overflows"));
        }
        Ok(Self {
            operator: filter_name(&value.operator).to_owned(),
            value: Value::try_from(&value.value)?,
            state: summary_state_name(&value.state).to_owned(),
        })
    }
}

impl TryFrom<TileCondition> for crate::document::TelemetrySummaryCondition {
    type Error = Error;

    fn try_from(row: TileCondition) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("summary condition projection memory overflows"));
        }
        Ok(Self { operator: filter_operator(&row.operator)?, value: row.value.try_into()?, state: summary_state(&row.state)? })
    }
}

impl Data {
    pub fn encoding_memory(value: &DataExpression) -> Result<usize> {
        let source = DataSource::encoding_memory(&value.source);
        if source == usize::MAX {
            return Ok(usize::MAX);
        }
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(value.label.as_deref().map_or(0, str::len))
            .saturating_add(source.saturating_sub(std::mem::size_of::<DataSource>()));
        for transform in &value.transforms {
            bytes = bytes.saturating_add(Transform::encoding_memory(transform));
        }
        Ok(bytes)
    }

    pub fn projection_memory(&self) -> Result<usize> {
        let mut bytes = std::mem::size_of::<DataExpression>()
            .saturating_add(self.label.as_ref().map_or(0, String::capacity))
            .saturating_add(self.source.projection_memory()?);
        for transform in &self.transforms {
            bytes = bytes.saturating_add(transform.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&DataExpression> for Data {
    type Error = Error;

    fn try_from(value: &DataExpression) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("data expression encoding memory overflows"));
        }
        let mut transforms = Vec::new();
        transforms.try_reserve_exact(value.transforms.len()).map_err(|_| Error::Invalid("data transforms allocation failed"))?;
        for transform in &value.transforms {
            transforms.push(Transform::try_from(transform)?);
        }
        Ok(Self { label: value.label.as_deref().map(str::to_owned), source: DataSource::try_from(&value.source)?, transforms })
    }
}

impl TryFrom<Data> for DataExpression {
    type Error = Error;

    fn try_from(row: Data) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("data expression projection memory overflows"));
        }
        let mut transforms = Vec::new();
        transforms.try_reserve_exact(row.transforms.len()).map_err(|_| Error::Invalid("data transforms allocation failed"))?;
        for transform in row.transforms {
            transforms.push(transform.try_into()?);
        }
        Ok(Self { label: row.label, source: row.source.try_into()?, transforms })
    }
}

impl DataSource {
    fn encoding_memory(value: &DomainSource) -> usize {
        let (kind, fields) = match value {
            DomainSource::Telemetry { stream, field } => ("telemetry", stream.len().saturating_add(field.len())),
            DomainSource::Parameter { name } => ("parameter", name.len()),
            DomainSource::Dimension { name } => ("dimension", name.len()),
            DomainSource::Record { record_type } => ("record", record_type.len()),
            DomainSource::Artifact { artifact } => ("artifact", artifact.len()),
        };
        std::mem::size_of::<Self>().saturating_add(kind.len()).saturating_add(fields)
    }

    fn projection_memory(&self) -> Result<usize> {
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
        let fields = [&self.stream, &self.field, &self.name, &self.record_type, &self.artifact];
        Ok(fields
            .into_iter()
            .fold(std::mem::size_of::<DomainSource>(), |bytes, field| bytes.saturating_add(field.as_ref().map_or(0, String::capacity))))
    }
}

impl TryFrom<&DomainSource> for DataSource {
    type Error = Error;

    fn try_from(value: &DomainSource) -> Result<Self> {
        if Self::encoding_memory(value) == usize::MAX {
            return Err(Error::Invalid("data source encoding memory overflows"));
        }
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
        row.projection_memory()?;
        match row.kind.as_str() {
            "telemetry" => Ok(Self::Telemetry { stream: row.stream.unwrap(), field: row.field.unwrap() }),
            "parameter" => Ok(Self::Parameter { name: row.name.unwrap() }),
            "dimension" => Ok(Self::Dimension { name: row.name.unwrap() }),
            "record" => Ok(Self::Record { record_type: row.record_type.unwrap() }),
            "artifact" => Ok(Self::Artifact { artifact: row.artifact.unwrap() }),
            _ => Err(Error::Invalid("invalid data source")),
        }
    }
}

impl Transform {
    fn encoding_memory(value: &DataTransform) -> usize {
        let fields = match value {
            DataTransform::TimeBetween { previous_record_type } => previous_record_type.len(),
            DataTransform::TelemetryAtOrBefore { stream, field } => stream.len().saturating_add(field.len()),
            DataTransform::Detail { name } => name.len(),
            DataTransform::Aggregate(aggregate) => aggregate.name().len(),
            _ => 0,
        };
        std::mem::size_of::<Self>().saturating_add(transform_name(value).len()).saturating_add(fields)
    }

    fn projection_memory(&self) -> Result<usize> {
        let kind = self.kind.as_str();
        let fields = [
            (self.previous_record_type.is_some(), kind == "time_between"),
            (self.stream.is_some(), kind == "telemetry_at_or_before"),
            (self.field.is_some(), kind == "telemetry_at_or_before"),
            (self.name.is_some(), kind == "detail"),
            (self.aggregate.is_some(), kind == "aggregate"),
        ];
        if !matches!(kind, "aggregate" | "time_between" | "telemetry_at_or_before" | "detail" | "per_case")
            || fields.iter().any(|(present, required)| present != required)
        {
            return Err(Error::Invalid("data transform fields differ"));
        }
        let fields = [&self.previous_record_type, &self.stream, &self.field, &self.name, &self.aggregate];
        Ok(fields
            .into_iter()
            .fold(std::mem::size_of::<DataTransform>(), |bytes, field| bytes.saturating_add(field.as_ref().map_or(0, String::capacity))))
    }
}

impl TryFrom<&DataTransform> for Transform {
    type Error = Error;

    fn try_from(value: &DataTransform) -> Result<Self> {
        if Self::encoding_memory(value) == usize::MAX {
            return Err(Error::Invalid("data transform encoding memory overflows"));
        }
        let mut transform = Self { kind: transform_name(value).to_owned(), ..Default::default() };
        match value {
            DataTransform::TimeBetween { previous_record_type } => {
                transform.previous_record_type = Some(previous_record_type.as_str().to_owned())
            }
            DataTransform::TelemetryAtOrBefore { stream, field } => {
                transform.stream = Some(stream.as_str().to_owned());
                transform.field = Some(field.as_str().to_owned());
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
        row.projection_memory()?;
        match row.kind.as_str() {
            "aggregate" => row
                .aggregate
                .as_deref()
                .and_then(harmos::Aggregate::parse)
                .map(Self::Aggregate)
                .ok_or(Error::Invalid("invalid data transform aggregate")),
            "time_between" => Ok(Self::TimeBetween { previous_record_type: row.previous_record_type.unwrap() }),
            "telemetry_at_or_before" => Ok(Self::TelemetryAtOrBefore { stream: row.stream.unwrap(), field: row.field.unwrap() }),
            "detail" => Ok(Self::Detail { name: row.name.unwrap() }),
            "per_case" => Ok(Self::PerCase),
            _ => Err(Error::Invalid("invalid data transform")),
        }
    }
}

impl Filter {
    pub fn encoding_memory(value: &crate::document::DocumentFilter) -> Result<usize> {
        let expression = Data::encoding_memory(&value.expression)?;
        let scalar = value.value.as_ref().map(Value::encoding_memory).transpose()?.unwrap_or(0);
        if [expression, scalar].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(expression.saturating_sub(std::mem::size_of::<Data>()))
            .saturating_add(filter_name(&value.operator).len())
            .saturating_add(scalar.saturating_sub(std::mem::size_of::<Value>())))
    }

    pub fn projection_memory(&self) -> Result<usize> {
        filter_operator(&self.operator)?;
        let value = self.value.as_ref().map(Value::projection_memory).transpose()?.unwrap_or(0);
        Ok(std::mem::size_of::<crate::document::DocumentFilter>()
            .saturating_add(self.expression.projection_memory()?)
            .saturating_add(value))
    }
}

impl TryFrom<&crate::document::DocumentFilter> for Filter {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentFilter) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("document filter encoding memory overflows"));
        }
        Ok(Self {
            expression: Data::try_from(&value.expression)?,
            operator: filter_name(&value.operator).to_owned(),
            value: value.value.as_ref().map(Value::try_from).transpose()?,
        })
    }
}

impl TryFrom<Filter> for crate::document::DocumentFilter {
    type Error = Error;

    fn try_from(row: Filter) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("document filter projection memory overflows"));
        }
        Ok(Self {
            expression: row.expression.try_into()?,
            operator: filter_operator(&row.operator)?,
            value: row.value.map(TryInto::try_into).transpose()?,
        })
    }
}

impl Repeat {
    pub(super) fn encoding_memory(value: &crate::document::DocumentSectionRepeat) -> Result<usize> {
        let scalar = Data::encoding_memory(&value.value)?;
        let order = value.order_by.as_ref().map(Data::encoding_memory).transpose()?.unwrap_or(0);
        if [scalar, order].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(scalar.saturating_sub(std::mem::size_of::<Data>()))
            .saturating_add(order.saturating_sub(std::mem::size_of::<Data>()))
            .saturating_add(value.label_template.len())
            .saturating_add(value.id_template.as_deref().map_or(0, str::len));
        for filter in &value.filters {
            bytes = bytes.saturating_add(Filter::encoding_memory(filter)?);
        }
        Ok(bytes)
    }

    pub(super) fn projection_memory(&self) -> Result<usize> {
        let order = self.order_by.as_ref().map(Data::projection_memory).transpose()?.unwrap_or(0);
        let mut bytes = std::mem::size_of::<crate::document::DocumentSectionRepeat>()
            .saturating_add(self.value.projection_memory()?)
            .saturating_add(order)
            .saturating_add(self.label_template.capacity())
            .saturating_add(self.id_template.as_ref().map_or(0, String::capacity));
        for filter in &self.filters {
            bytes = bytes.saturating_add(filter.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::document::DocumentSectionRepeat> for Repeat {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentSectionRepeat) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("document repeat encoding memory overflows"));
        }
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
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("document repeat projection memory overflows"));
        }
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
    pub fn encoding_memory(value: &crate::document::DocumentCardLayout) -> Result<usize> {
        for value in
            [Some(value.col_span.base), value.col_span.sm, value.col_span.md, value.col_span.lg, value.col_span.xl, Some(value.row_span)]
                .into_iter()
                .flatten()
        {
            span(u32::from(value))?;
        }
        Ok(std::mem::size_of::<Self>().saturating_add(density_name(&value.density).len()).saturating_add(rail_name(&value.rail).len()))
    }

    pub fn validate(&self) -> Result<()> {
        for value in [Some(self.col_span.base), self.col_span.sm, self.col_span.md, self.col_span.lg, self.col_span.xl, Some(self.row_span)]
            .into_iter()
            .flatten()
        {
            span(value)?;
        }
        density(&self.density)?;
        rail(&self.rail)?;
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
            density: density(&row.density)?,
            rail: rail(&row.rail)?,
        })
    }
}

impl Placement {
    pub(super) fn encoding_memory(value: &crate::document::DocumentCardPlacement) -> Result<usize> {
        let layout = CardLayout::encoding_memory(&value.layout)?;
        if layout == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(value.card_id.as_str().len())
            .saturating_add(layout.saturating_sub(std::mem::size_of::<CardLayout>())))
    }

    pub(super) fn projection_memory(&self) -> Result<usize> {
        self.layout.validate()?;
        Ok(std::mem::size_of::<crate::document::DocumentCardPlacement>().saturating_add(self.card_id.capacity()))
    }
}

impl TryFrom<&crate::document::DocumentCardPlacement> for Placement {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentCardPlacement) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("document placement encoding memory overflows"));
        }
        Ok(Self { card_id: value.card_id.as_str().to_owned(), layout: value.layout.clone().into() })
    }
}

impl TryFrom<Placement> for crate::document::DocumentCardPlacement {
    type Error = Error;

    fn try_from(row: Placement) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("document placement projection memory overflows"));
        }
        Ok(Self { card_id: dokime_domain::model::identity::Identifier::new(row.card_id)?, layout: row.layout.try_into()? })
    }
}

impl Default for CardLayout {
    fn default() -> Self {
        crate::document::DocumentCardLayout::default().into()
    }
}

impl From<crate::document::DocumentCardLayout> for CardLayout {
    fn from(layout: crate::document::DocumentCardLayout) -> Self {
        use crate::document::{DocumentCardDensity, DocumentCardRail};

        let density = match layout.density {
            DocumentCardDensity::Compact => "compact",
            DocumentCardDensity::Normal => "normal",
            DocumentCardDensity::Spacious => "spacious",
        };
        let rail = match layout.rail {
            DocumentCardRail::Main => "main",
            DocumentCardRail::Right => "right",
            DocumentCardRail::Left => "left",
        };
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
            density: density.into(),
            rail: rail.into(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn body_encoding_memory(value: &crate::document::DocumentCardBody) -> Result<usize> {
    use crate::document::DocumentCardBody;
    let mut bytes = body_name(value).len();
    match value {
        DocumentCardBody::Text { markdown } => bytes = bytes.saturating_add(markdown.len()),
        DocumentCardBody::TelemetrySummary { summary } => {
            let memory = Summary::encoding_memory(summary)?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(memory.saturating_sub(std::mem::size_of::<Summary>()));
        }
        DocumentCardBody::Plot { plot } => {
            let memory = Plot::encoding_memory(plot)?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(memory.saturating_sub(std::mem::size_of::<Plot>()));
        }
        DocumentCardBody::Table { rows } => {
            for row in rows {
                bytes = bytes.saturating_add(Map::encoding_memory(row)?);
            }
        }
        _ => {}
    }
    Ok(bytes)
}

fn encode_body(value: &crate::document::DocumentCardBody) -> Result<(String, Option<CardData>)> {
    use crate::document::DocumentCardBody;
    if body_encoding_memory(value)? == usize::MAX {
        return Err(Error::Invalid("document body encoding memory overflows"));
    }
    let data = match value {
        DocumentCardBody::Text { markdown } => Some(CardData { markdown: Some(markdown.as_str().to_owned()), ..Default::default() }),
        DocumentCardBody::TelemetrySummary { summary } => {
            Some(CardData { summary: Some(Summary::try_from(summary)?), ..Default::default() })
        }
        DocumentCardBody::Plot { plot } => Some(CardData { plot: Some(Plot::try_from(plot.as_ref())?), ..Default::default() }),
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

fn summary_stat_name(value: &crate::document::TelemetrySummaryStat) -> &'static str {
    match value {
        crate::document::TelemetrySummaryStat::Latest => "latest",
    }
}

fn summary_format_name(value: &crate::document::TelemetrySummaryFormat) -> &'static str {
    use crate::document::TelemetrySummaryFormat;
    match value {
        TelemetrySummaryFormat::Auto => "auto",
        TelemetrySummaryFormat::Number => "number",
        TelemetrySummaryFormat::Percent => "percent",
        TelemetrySummaryFormat::Duration => "duration",
        TelemetrySummaryFormat::Distance => "distance",
        TelemetrySummaryFormat::Status => "status",
        TelemetrySummaryFormat::Text => "text",
    }
}

fn summary_presentation_name(value: &crate::document::TelemetrySummaryPresentation) -> &'static str {
    use crate::document::TelemetrySummaryPresentation;
    match value {
        TelemetrySummaryPresentation::Value => "value",
        TelemetrySummaryPresentation::Progress => "progress",
        TelemetrySummaryPresentation::Badge => "badge",
    }
}

fn summary_state_name(value: &crate::document::TelemetrySummaryState) -> &'static str {
    use crate::document::TelemetrySummaryState;
    match value {
        TelemetrySummaryState::Ok => "ok",
        TelemetrySummaryState::Warning => "warning",
        TelemetrySummaryState::Critical => "critical",
        TelemetrySummaryState::Muted => "muted",
    }
}

fn plot_name(value: &crate::document::PlotType) -> &'static str {
    use crate::document::PlotType;
    match value {
        PlotType::TimeSeries => "time_series",
        PlotType::Bar => "bar",
        PlotType::GroupedBar => "grouped_bar",
        PlotType::StackedBar => "stacked_bar",
        PlotType::Pie => "pie",
        PlotType::Donut => "donut",
        PlotType::Pareto => "pareto",
        PlotType::XyScatter => "xy_scatter",
        PlotType::XyLine => "xy_line",
        PlotType::XyInterval => "xy_interval",
        PlotType::Trajectory => "trajectory",
        PlotType::Heatmap => "heatmap",
        PlotType::XyValueHeatmap => "xy_value_heatmap",
        PlotType::WifiChannelOccupancy => "wifi_channel_occupancy",
        PlotType::FrequencySeries => "frequency_series",
        PlotType::Histogram => "histogram",
        PlotType::BoxSummary => "box_summary",
        PlotType::AggregateTable => "aggregate_table",
    }
}

fn style_name(value: &crate::document::PlotStyle) -> &'static str {
    use crate::document::PlotStyle;
    match value {
        PlotStyle::Line => "line",
        PlotStyle::Points => "points",
        PlotStyle::LinePoints => "line_points",
        PlotStyle::Step => "step",
    }
}

fn scale_name(value: &crate::document::AxisScale) -> &'static str {
    use crate::document::AxisScale;
    match value {
        AxisScale::Linear => "linear",
        AxisScale::Log => "log",
    }
}

fn fft_window_name(value: &crate::document::FftWindow) -> &'static str {
    use crate::document::FftWindow;
    match value {
        FftWindow::Hann => "hann",
        FftWindow::Rectangular => "rectangular",
    }
}

fn fft_detrend_name(value: &crate::document::FftDetrend) -> &'static str {
    use crate::document::FftDetrend;
    match value {
        FftDetrend::None => "none",
        FftDetrend::Mean => "mean",
    }
}

fn level_unit_name(value: &crate::document::FrequencyLevelUnit) -> &'static str {
    use crate::document::FrequencyLevelUnit;
    match value {
        FrequencyLevelUnit::Linear => "linear",
        FrequencyLevelUnit::Db => "dB",
        FrequencyLevelUnit::Dbu => "dBu",
        FrequencyLevelUnit::Dbm => "dBm",
    }
}

fn reference_axis_name(value: &crate::document::ReferenceAxis) -> &'static str {
    use crate::document::ReferenceAxis;
    match value {
        ReferenceAxis::X => "x",
        ReferenceAxis::Y => "y",
    }
}

fn elapsed_unit_name(value: &dokime_domain::model::values::ElapsedUnit) -> &'static str {
    use dokime_domain::model::values::ElapsedUnit;
    match value {
        ElapsedUnit::Seconds => "seconds",
        ElapsedUnit::Milliseconds => "milliseconds",
        ElapsedUnit::Minutes => "minutes",
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
        DataTransform::TelemetryAtOrBefore { .. } => "telemetry_at_or_before",
        DataTransform::Detail { .. } => "detail",
        DataTransform::PerCase => "per_case",
    }
}

fn density_name(value: &crate::document::DocumentCardDensity) -> &'static str {
    use crate::document::DocumentCardDensity;
    match value {
        DocumentCardDensity::Compact => "compact",
        DocumentCardDensity::Normal => "normal",
        DocumentCardDensity::Spacious => "spacious",
    }
}

fn rail_name(value: &crate::document::DocumentCardRail) -> &'static str {
    use crate::document::DocumentCardRail;
    match value {
        DocumentCardRail::Main => "main",
        DocumentCardRail::Right => "right",
        DocumentCardRail::Left => "left",
    }
}

fn filter_name(value: &crate::document::FilterOperator) -> &'static str {
    use crate::document::FilterOperator;
    match value {
        FilterOperator::Exists => "exists",
        FilterOperator::Eq => "eq",
        FilterOperator::Ne => "ne",
        FilterOperator::Gt => "gt",
        FilterOperator::Gte => "gte",
        FilterOperator::Lt => "lt",
        FilterOperator::Lte => "lte",
        FilterOperator::Contains => "contains",
    }
}

fn body_memory(kind: &str, data: &Option<CardData>) -> Result<usize> {
    let mut bytes = std::mem::size_of::<crate::document::DocumentCardBody>();
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
            if let Some(markdown) = &data.markdown {
                bytes = bytes.saturating_add(markdown.capacity());
            }
            if let Some(summary) = &data.summary {
                bytes = bytes.saturating_add(summary.projection_memory()?);
            }
            if let Some(plot) = &data.plot {
                bytes = bytes.saturating_add(plot.projection_memory()?);
            }
            for row in &data.rows {
                bytes = bytes.saturating_add(row.projection_memory()?);
            }
        }
        ("steps" | "criteria" | "traceability" | "notes" | "records" | "artifacts" | "media" | "health" | "telemetry_context", None) => {}
        _ => return Err(Error::Invalid("document card body kind or data differs")),
    }
    Ok(bytes)
}

fn consume_body(kind: String, data: Option<CardData>) -> Result<crate::document::DocumentCardBody> {
    use crate::document::DocumentCardBody;
    if body_memory(&kind, &data)? == usize::MAX {
        return Err(Error::Invalid("document card body projection memory overflows"));
    }
    Ok(match kind.as_str() {
        "text" => DocumentCardBody::Text { markdown: data.unwrap().markdown.unwrap() },
        "telemetry_summary" => DocumentCardBody::TelemetrySummary { summary: data.unwrap().summary.unwrap().try_into()? },
        "plot" => DocumentCardBody::Plot { plot: Box::new(data.unwrap().plot.unwrap().try_into()?) },
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

fn plot_type(value: &str) -> Result<crate::document::PlotType> {
    use crate::document::PlotType;
    match value {
        "time_series" => Ok(PlotType::TimeSeries),
        "bar" => Ok(PlotType::Bar),
        "grouped_bar" => Ok(PlotType::GroupedBar),
        "stacked_bar" => Ok(PlotType::StackedBar),
        "pie" => Ok(PlotType::Pie),
        "donut" => Ok(PlotType::Donut),
        "pareto" => Ok(PlotType::Pareto),
        "xy_scatter" => Ok(PlotType::XyScatter),
        "xy_line" => Ok(PlotType::XyLine),
        "xy_interval" => Ok(PlotType::XyInterval),
        "trajectory" => Ok(PlotType::Trajectory),
        "heatmap" => Ok(PlotType::Heatmap),
        "xy_value_heatmap" => Ok(PlotType::XyValueHeatmap),
        "wifi_channel_occupancy" => Ok(PlotType::WifiChannelOccupancy),
        "frequency_series" => Ok(PlotType::FrequencySeries),
        "histogram" => Ok(PlotType::Histogram),
        "box_summary" => Ok(PlotType::BoxSummary),
        "aggregate_table" => Ok(PlotType::AggregateTable),
        _ => Err(Error::Invalid("invalid plot type")),
    }
}

fn plot_style(value: &str) -> Result<crate::document::PlotStyle> {
    use crate::document::PlotStyle;
    match value {
        "line" => Ok(PlotStyle::Line),
        "points" => Ok(PlotStyle::Points),
        "line_points" => Ok(PlotStyle::LinePoints),
        "step" => Ok(PlotStyle::Step),
        _ => Err(Error::Invalid("invalid plot style")),
    }
}

fn axis_scale(value: &str) -> Result<crate::document::AxisScale> {
    use crate::document::AxisScale;
    match value {
        "linear" => Ok(AxisScale::Linear),
        "log" => Ok(AxisScale::Log),
        _ => Err(Error::Invalid("invalid axis scale")),
    }
}

fn finite(value: f64) -> Result<()> {
    if value.is_finite() { Ok(()) } else { Err(Error::Invalid("plot number is nonfinite")) }
}

fn format_digits(value: u32) -> Result<u8> {
    u8::try_from(value).map_err(|_| Error::Invalid("value format digits exceed byte range"))
}

fn fft_window(value: &str) -> Result<crate::document::FftWindow> {
    use crate::document::FftWindow;
    match value {
        "hann" => Ok(FftWindow::Hann),
        "rectangular" => Ok(FftWindow::Rectangular),
        _ => Err(Error::Invalid("invalid fft window")),
    }
}

fn fft_detrend(value: &str) -> Result<crate::document::FftDetrend> {
    use crate::document::FftDetrend;
    match value {
        "none" => Ok(FftDetrend::None),
        "mean" => Ok(FftDetrend::Mean),
        _ => Err(Error::Invalid("invalid fft detrend")),
    }
}

fn level_unit(value: &str) -> Result<crate::document::FrequencyLevelUnit> {
    use crate::document::FrequencyLevelUnit;
    match value {
        "linear" => Ok(FrequencyLevelUnit::Linear),
        "dB" => Ok(FrequencyLevelUnit::Db),
        "dBu" => Ok(FrequencyLevelUnit::Dbu),
        "dBm" => Ok(FrequencyLevelUnit::Dbm),
        _ => Err(Error::Invalid("invalid level unit")),
    }
}

fn reference_axis(value: &str) -> Result<crate::document::ReferenceAxis> {
    use crate::document::ReferenceAxis;
    match value {
        "x" => Ok(ReferenceAxis::X),
        "y" => Ok(ReferenceAxis::Y),
        _ => Err(Error::Invalid("invalid reference axis")),
    }
}

fn elapsed_unit(value: &str) -> Result<dokime_domain::model::values::ElapsedUnit> {
    use dokime_domain::model::values::ElapsedUnit;
    match value {
        "seconds" => Ok(ElapsedUnit::Seconds),
        "milliseconds" => Ok(ElapsedUnit::Milliseconds),
        "minutes" => Ok(ElapsedUnit::Minutes),
        _ => Err(Error::Invalid("invalid elapsed unit")),
    }
}

fn summary_format(value: &str) -> Result<crate::document::TelemetrySummaryFormat> {
    use crate::document::TelemetrySummaryFormat;
    match value {
        "auto" => Ok(TelemetrySummaryFormat::Auto),
        "number" => Ok(TelemetrySummaryFormat::Number),
        "percent" => Ok(TelemetrySummaryFormat::Percent),
        "duration" => Ok(TelemetrySummaryFormat::Duration),
        "distance" => Ok(TelemetrySummaryFormat::Distance),
        "status" => Ok(TelemetrySummaryFormat::Status),
        "text" => Ok(TelemetrySummaryFormat::Text),
        _ => Err(Error::Invalid("invalid summary format")),
    }
}

fn summary_presentation(value: &str) -> Result<crate::document::TelemetrySummaryPresentation> {
    use crate::document::TelemetrySummaryPresentation;
    match value {
        "value" => Ok(TelemetrySummaryPresentation::Value),
        "progress" => Ok(TelemetrySummaryPresentation::Progress),
        "badge" => Ok(TelemetrySummaryPresentation::Badge),
        _ => Err(Error::Invalid("invalid summary presentation")),
    }
}

fn summary_state(value: &str) -> Result<crate::document::TelemetrySummaryState> {
    use crate::document::TelemetrySummaryState;
    match value {
        "ok" => Ok(TelemetrySummaryState::Ok),
        "warning" => Ok(TelemetrySummaryState::Warning),
        "critical" => Ok(TelemetrySummaryState::Critical),
        "muted" => Ok(TelemetrySummaryState::Muted),
        _ => Err(Error::Invalid("invalid summary state")),
    }
}

fn span(value: u32) -> Result<u8> {
    u8::try_from(value).map_err(|_| Error::Invalid("document span exceeds byte range"))
}

fn density(value: &str) -> Result<crate::document::DocumentCardDensity> {
    use crate::document::DocumentCardDensity;
    match value {
        "compact" => Ok(DocumentCardDensity::Compact),
        "normal" => Ok(DocumentCardDensity::Normal),
        "spacious" => Ok(DocumentCardDensity::Spacious),
        _ => Err(Error::Invalid("invalid document density")),
    }
}

fn rail(value: &str) -> Result<crate::document::DocumentCardRail> {
    use crate::document::DocumentCardRail;
    match value {
        "main" => Ok(DocumentCardRail::Main),
        "right" => Ok(DocumentCardRail::Right),
        "left" => Ok(DocumentCardRail::Left),
        _ => Err(Error::Invalid("invalid document rail")),
    }
}

fn filter_operator(value: &str) -> Result<crate::document::FilterOperator> {
    use crate::document::FilterOperator;
    match value {
        "eq" => Ok(FilterOperator::Eq),
        "ne" => Ok(FilterOperator::Ne),
        "gt" => Ok(FilterOperator::Gt),
        "gte" => Ok(FilterOperator::Gte),
        "lt" => Ok(FilterOperator::Lt),
        "lte" => Ok(FilterOperator::Lte),
        "contains" => Ok(FilterOperator::Contains),
        "exists" => Ok(FilterOperator::Exists),
        _ => Err(Error::Invalid("invalid document filter operator")),
    }
}

/* === Tests === */
