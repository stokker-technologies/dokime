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
use std::{io::Write, num::NonZeroUsize, path::Path};

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, FieldDescriptor, MessageDescriptor, Value};
use tempfile::NamedTempFile;

// Local Imports
use crate::{Error, Host, Result};
use dokime_protocol::v1;
use dokime_storage::{Execution, ExecutionCut, ExecutionState, Report, Sample, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct FixedSnapshot {
    store: Store,
    cut: ExecutionCut,
}

#[derive(Clone, Copy)]
pub struct RenderSettings {
    pub page_size: NonZeroUsize,
    pub display_points: NonZeroUsize,
}

pub struct RenderedDocument {
    output: NamedTempFile,
    execution: String,
    execution_seal: Vec<u8>,
    document: v1::Document,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl FixedSnapshot {
    pub fn execution(&self) -> &Execution {
        &self.cut.execution
    }
    pub fn seal(&self) -> &[u8] {
        &self.cut.seal
    }
}
impl RenderedDocument {
    pub fn path(&self) -> &Path {
        self.output.path()
    }
    pub fn html(&self) -> Result<Vec<u8>> {
        Ok(std::fs::read(self.path())?)
    }
}
impl Host {
    pub fn snapshot(&self, execution: &str) -> Result<FixedSnapshot> {
        let cut = self.inner.store.cut(execution)?;
        if cut.state != ExecutionState::Sealed {
            return Err(Error::Invalid("Render Document: Select a Sealed Execution for Its Fixed Snapshot".into()));
        }
        Ok(FixedSnapshot { store: self.inner.store.clone(), cut })
    }
    pub fn render_document(&self, snapshot: &FixedSnapshot, document: v1::Document, settings: RenderSettings) -> Result<RenderedDocument> {
        let mut output = NamedTempFile::with_prefix("dokime-render-")?;
        let title = document.metadata.as_ref().map_or("", |metadata| metadata.label.as_str());
        write!(
            output,
            "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>{}</title><body><h1>{}</h1>",
            escape(title),
            escape(title)
        )?;
        let catalog = h::Catalog::decode(snapshot.cut.execution.catalog.as_slice())?;
        let pool = DescriptorPool::from_file_descriptor_set(
            catalog.types.clone().ok_or_else(|| Error::Invalid("Render Document: Supply the Frozen Row Descriptors".into()))?,
        )
        .map_err(|error| Error::Invalid(format!("Render Document: {error}")))?;
        for section in &document.sections {
            render_section(&mut output, snapshot, section, &catalog, &pool, settings)?;
        }
        write!(output, "</body></html>")?;
        output.flush()?;
        Ok(RenderedDocument {
            output,
            execution: snapshot.cut.execution.identifier.clone(),
            execution_seal: snapshot.cut.seal.clone(),
            document,
        })
    }
    pub fn publish_report(&self, output: &RenderedDocument) -> Result<Report> {
        Ok(self.inner.store.publish_report(&output.execution, &output.execution_seal, output.document.clone(), output.html()?)?)
    }
    pub fn reports(&self, execution: &str) -> Result<Vec<Report>> {
        Ok(self.inner.store.reports(execution)?)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn render_section(
    output: &mut NamedTempFile, snapshot: &FixedSnapshot, section: &v1::Section, catalog: &h::Catalog, pool: &DescriptorPool,
    settings: RenderSettings,
) -> Result<()> {
    if section.repeat.is_some() {
        return Err(Error::Invalid("Render Document: Expand Repeated Sections Before Rendering".into()));
    }
    let label = section.metadata.as_ref().map_or("", |metadata| metadata.label.as_str());
    write!(output, "<section><h2>{}</h2>", escape(label))?;
    for card in &section.cards {
        if !card.filters.is_empty() {
            return Err(Error::Invalid("Render Document: Use Unfiltered Cards in This Renderer".into()));
        }
        match card.body.as_ref() {
            Some(v1::card::Body::Markdown(text)) => write!(output, "<pre>{}</pre>", escape(text))?,
            Some(v1::card::Body::Plot(plot)) => render_plot(output, snapshot, plot, catalog, pool, settings)?,
            _ => return Err(Error::Invalid("Render Document: Select Markdown or Time Series Cards".into())),
        }
    }
    for child in &section.sections {
        render_section(output, snapshot, child, catalog, pool, settings)?;
    }
    write!(output, "</section>")?;
    Ok(())
}

fn render_plot(
    output: &mut NamedTempFile, snapshot: &FixedSnapshot, plot: &v1::Plot, catalog: &h::Catalog, pool: &DescriptorPool,
    settings: RenderSettings,
) -> Result<()> {
    if plot.kind != v1::PlotKind::TimeSeries as i32
        || plot.x.is_some()
        || plot.y.is_empty()
        || !plot.filters.is_empty()
        || plot.series_by.is_some()
    {
        return Err(Error::Invalid("Render Document Plot: Select an Unfiltered Time Series with Stream Value Columns".into()));
    }
    let label = plot.metadata.as_ref().map_or("", |metadata| metadata.label.as_str());
    write!(output, "<figure><figcaption>{}</figcaption>", escape(label))?;
    for expression in &plot.y {
        let Some(v1::data_expression::Kind::Stream(reference)) = expression.kind.as_ref() else {
            return Err(Error::Invalid("Render Document Plot: Select a Numeric Stored Stream Field".into()));
        };
        let stream = catalog
            .streams
            .iter()
            .find(|stream| stream.id == reference.identifier)
            .ok_or_else(|| Error::Invalid(format!("Render Document Plot: Declare Stream {}", reference.identifier)))?;
        let descriptor = pool
            .get_message_by_name(&stream.row)
            .ok_or_else(|| Error::Invalid("Render Document Plot: Restore the Stream Row Descriptor".into()))?;
        let field = descriptor.get_field_by_name(&reference.field).ok_or_else(|| {
            Error::Invalid(format!("Render Document Plot: Select a Declared Field {}.{}", reference.identifier, reference.field))
        })?;
        let mut count = 0_u64;
        let mut min_time = i64::MAX;
        let mut max_time = i64::MIN;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        scan(snapshot, &stream.id, &descriptor, &field, settings.page_size, |sample, value| {
            count += 1;
            min_time = min_time.min(sample.source_ns);
            max_time = max_time.max(sample.source_ns);
            min = min.min(value);
            max = max.max(value);
            Ok(())
        })?;
        write!(
            output,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 640 240\" role=\"img\" data-stream=\"{}\" data-count=\"{count}\"><path d=\"M20 10V220H630\" fill=\"none\" stroke=\"currentColor\"/><polyline fill=\"none\" stroke=\"#4477aa\" points=\"",
            escape(&stream.id)
        )?;
        let stride = count.div_ceil(settings.display_points.get() as u64).max(1);
        let mut index = 0_u64;
        scan(snapshot, &stream.id, &descriptor, &field, settings.page_size, |sample, value| {
            if index.is_multiple_of(stride) {
                let x = 20.0
                    + 610.0 * ((sample.source_ns as i128 - min_time as i128) as f64 / (max_time as i128 - min_time as i128).max(1) as f64);
                let y = if max == min { 115.0 } else { 220.0 - 210.0 * (value - min) / (max - min) };
                write!(output, "{x:.3},{y:.3} ")?;
            }
            index += 1;
            Ok(())
        })?;
        write!(output, "\"></polyline></svg>")?;
    }
    write!(output, "</figure>")?;
    Ok(())
}

fn scan(
    snapshot: &FixedSnapshot, stream: &str, descriptor: &MessageDescriptor, field: &FieldDescriptor, page_size: NonZeroUsize,
    mut visit: impl FnMut(&Sample, f64) -> Result<()>,
) -> Result<()> {
    let mut after = 0;
    loop {
        let page = snapshot.store.sample_page(&snapshot.cut.execution.identifier, stream, after, snapshot.cut.frontier, page_size)?;
        if page.is_empty() {
            return Ok(());
        }
        for sample in page {
            after = sample.position;
            let row = DynamicMessage::decode(descriptor.clone(), sample.row.as_slice())?;
            let value = match row.get_field(field).as_ref() {
                Value::F64(value) => *value,
                Value::F32(value) => *value as f64,
                Value::I64(value) => *value as f64,
                Value::U64(value) => *value as f64,
                Value::I32(value) => *value as f64,
                Value::U32(value) => *value as f64,
                _ => return Err(Error::Invalid("Render Document Plot: Select a Numeric Stream Field".into())),
            };
            if !value.is_finite() {
                return Err(Error::Invalid("Render Document Plot: Use Finite Stored Values".into()));
            }
            visit(&sample, value)?;
        }
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

/* === Tests === */
