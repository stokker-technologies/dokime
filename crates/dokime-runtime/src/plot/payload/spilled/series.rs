/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Paged Exact Plot Series Reduction

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use arrow_array::{Array, Float64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use harmos::storage::query::{
    Accumulator, Table, TableWriter,
    scalar::{Agg, ScalarValue},
    transform::{
        FieldPath, Transform,
        downsample::{Algorithm, Downsample},
    },
};

// Local Imports
use super::super::{DownsampleTechnique, PlotPoint, PlotSeries, PlotSeriesSummary, PlotWindow, analyses};
use super::input::failure;
use crate::runtime::Result;

/* === Constants === */

const PAGE_ROWS: usize = 1024;
const PAGE_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Points {
    writer: TableWriter,
    schema: Arc<Schema>,
    json: Vec<String>,
    axis: Vec<Option<f64>>,
    y: Vec<Option<f64>>,
    z: Vec<Option<f64>>,
    bytes: usize,
}

pub(super) struct Series {
    pub metadata: PlotSeries,
    pub table: Table,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Points {
    pub(super) fn new() -> Result<Self> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("x", DataType::Utf8, false),
            Field::new("axis", DataType::Float64, true),
            Field::new("y", DataType::Float64, true),
            Field::new("z", DataType::Float64, true),
        ]));
        let writer = TableWriter::new(schema.clone())?;
        Ok(Self { writer, schema, json: Vec::new(), axis: Vec::new(), y: Vec::new(), z: Vec::new(), bytes: 0 })
    }

    pub(super) fn push(&mut self, point: PlotPoint) -> Result<()> {
        let json = serde_json::to_string(&point.x).map_err(failure)?;
        let bytes = json.len().saturating_add(64);
        if !self.json.is_empty() && (self.json.len() == PAGE_ROWS || self.bytes.saturating_add(bytes) > PAGE_BYTES) {
            self.flush()?;
        }
        self.axis.push(analyses::point_x(&point));
        self.y.push(point.y);
        self.z.push(point.z);
        self.json.push(json);
        self.bytes = self.bytes.saturating_add(bytes);
        Ok(())
    }

    pub(super) fn finish(mut self, mut metadata: PlotSeries) -> Result<Series> {
        self.flush()?;
        let table = self.writer.finish()?;
        metadata.raw_points = table.len();
        metadata.points.clear();
        Ok(Series { metadata, table })
    }

    fn flush(&mut self) -> Result<()> {
        if self.json.is_empty() {
            return Ok(());
        }
        let json = StringArray::from(std::mem::take(&mut self.json));
        let axis = Float64Array::from(std::mem::take(&mut self.axis));
        let y = Float64Array::from(std::mem::take(&mut self.y));
        let z = Float64Array::from(std::mem::take(&mut self.z));
        let arrays: Vec<Arc<dyn Array>> = vec![Arc::new(json), Arc::new(axis), Arc::new(y), Arc::new(z)];
        let page = RecordBatch::try_new(self.schema.clone(), arrays).map_err(failure)?;
        self.writer.push(&page, &mut || Ok(()))?;
        self.bytes = 0;
        Ok(())
    }
}

impl Series {
    pub(super) fn visit(&self, visit: impl FnMut(PlotPoint) -> Result<()>) -> Result<()> {
        visit_table(&self.table, visit)
    }

    pub(super) fn summary(&self, window: Option<PlotWindow>) -> Result<PlotSeriesSummary> {
        let mut moments = harmos::signal::Summary::new();
        let mut median = Accumulator::new(Agg::Of(harmos::Aggregate::Median));
        let mut extent: Option<PlotWindow> = None;
        self.visit(|point| {
            let Some((x, y)) = analyses::point_x(&point).zip(point.y) else {
                return Ok(());
            };
            if window.is_some_and(|window| x < window.from || x > window.to) {
                return Ok(());
            }
            let range = extent.get_or_insert(PlotWindow { from: x, to: x });
            range.from = range.from.min(x);
            range.to = range.to.max(x);
            median.push(moments.count(), &ScalarValue::Float(y))?;
            moments.push(y);
            Ok(())
        })?;
        let median = scalar_number(median.finish(&mut || Ok(()))?);
        Ok(PlotSeriesSummary {
            series_id: self.metadata.key.clone(),
            count: moments.count() as usize,
            min: moments.min(),
            max: moments.max(),
            mean: moments.mean(),
            median,
            std: moments.std(),
            rms: moments.rms(),
            window: window.or(extent),
        })
    }

    pub(super) fn reduce(mut self, maximum: usize, technique: DownsampleTechnique) -> Result<PlotSeries> {
        let rows = self.table.len();
        if maximum == 0 || rows <= maximum {
            visit_table(&self.table, |point| {
                self.metadata.points.push(point);
                Ok(())
            })?;
            return Ok(self.metadata);
        }
        let technique = match technique {
            DownsampleTechnique::Auto | DownsampleTechnique::Unspecified if rows <= maximum.saturating_mul(4) => DownsampleTechnique::Lttb,
            DownsampleTechnique::Auto | DownsampleTechnique::Unspecified => DownsampleTechnique::MinMax,
            technique => technique,
        };
        let mut missing_y = false;
        let mut missing_x = false;
        self.visit(|point| {
            missing_y |= point.y.is_none_or(|value| !value.is_finite());
            missing_x |= analyses::point_x(&point).is_none();
            Ok(())
        })?;
        let decimate = maximum == 1
            || technique == DownsampleTechnique::Raw
            || missing_y
            || (technique == DownsampleTechnique::Lttb && (maximum < 3 || missing_x));
        let algorithm = if decimate {
            Algorithm::Decimate
        } else if technique == DownsampleTechnique::MinMax {
            Algorithm::Extrema
        } else {
            Algorithm::Lttb
        };
        let transform = Transform::Downsample(Downsample {
            max_rows: maximum,
            algorithm,
            value: Some(FieldPath { fields: vec!["y".into()] }),
            axis: Some(FieldPath { fields: vec!["axis".into()] }),
        });
        let reduced = self.table.transform(&transform, &mut || Ok(()))?;
        visit_table(&reduced, |point| {
            self.metadata.points.push(point);
            Ok(())
        })?;
        self.metadata.downsampled = true;
        Ok(self.metadata)
    }
}

/* === Public Functions === */

pub(super) fn scalar_number(value: ScalarValue) -> Option<f64> {
    match value {
        ScalarValue::Float(value) => Some(value),
        ScalarValue::Unsigned(value) => Some(value as f64),
        ScalarValue::Signed(value) => Some(value as f64),
        _ => None,
    }
}

/* === Private Functions === */

pub(super) fn visit_table(table: &Table, mut visit: impl FnMut(PlotPoint) -> Result<()>) -> Result<()> {
    let mut reader = table.reader()?;
    while let Some(page) = reader.next(&mut || Ok(()))? {
        let json = page.column(0).as_any().downcast_ref::<StringArray>().expect("plot x schema");
        let y = page.column(2).as_any().downcast_ref::<Float64Array>().expect("plot y schema");
        let z = page.column(3).as_any().downcast_ref::<Float64Array>().expect("plot z schema");
        for index in 0..page.num_rows() {
            let x = serde_json::from_str(json.value(index)).map_err(failure)?;
            let y = (!y.is_null(index)).then(|| y.value(index));
            let z = (!z.is_null(index)).then(|| z.value(index));
            visit(PlotPoint { x, y, z })?;
        }
    }
    Ok(())
}

/* === Tests === */
