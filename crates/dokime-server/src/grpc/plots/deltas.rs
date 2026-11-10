/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Compare Complete Display Buckets And Transmit Their Replacements.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports

// Local Imports
use dokime::harmos::storage::ReadBatch;
use dokime::runtime::query::{PlotPayload, PlotPayloadEnvelope, PlotPayloadRequest, PlotPoint};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn between(
    previous: &PlotPayload, value: ReadBatch<PlotPayloadEnvelope>, request: &PlotPayloadRequest,
) -> Result<Option<ReadBatch<PlotPayloadEnvelope>>, dokime::runtime::Error> {
    let Some(width) = value.payload.bucket_width_s.filter(|width| Some(*width) == previous.bucket_width_s) else { return Ok(None) };
    if previous.series.iter().map(|series| &series.key).ne(value.payload.series.iter().map(|series| &series.key)) {
        return Ok(None);
    }
    let old = buckets(previous, width);
    let new = buckets(&value.payload, width);
    let changed = old
        .keys()
        .chain(new.keys())
        .chain(previous.bucket_counts.keys())
        .chain(value.payload.bucket_counts.keys())
        .copied()
        .filter(|key| old.get(key) != new.get(key) || previous.bucket_counts.get(key) != value.payload.bucket_counts.get(key))
        .collect::<BTreeSet<_>>();
    let full = value.payload.cartesian.clone();
    let maximum = value.payload.max_points;
    let value = value.try_map(|mut value| {
        value.payload.replaced_buckets = changed.iter().map(|key| *key as f64).collect();
        value.payload.summaries.clear();
        for series in &mut value.payload.series {
            series.points.retain(|point| bucket(point, width).is_some_and(|key| changed.contains(&key)));
        }
        Ok::<_, dokime::runtime::Error>(value)
    })?;
    let mut request = request.clone();
    request.panel_width = None;
    let value = dokime::runtime::query::reduce_plot_payload(value, &request, maximum)?;
    let value = value.try_map(|mut value| {
        if let (Some(frame), Some(full)) = (&mut value.payload.cartesian, full) {
            frame.x_domain = full.x_domain;
            frame.y_domain = full.y_domain;
            frame.signature = full.signature;
            frame.point_count = full.point_count;
        }
        Ok::<_, dokime::runtime::Error>(value)
    })?;
    Ok(Some(value))
}

/* === Private Functions === */

fn buckets(payload: &PlotPayload, width: f64) -> BTreeMap<i128, Vec<(usize, PlotPoint)>> {
    let mut buckets = BTreeMap::<i128, Vec<(usize, PlotPoint)>>::new();
    for (series, values) in payload.series.iter().enumerate() {
        for point in &values.points {
            if let Some(key) = bucket(point, width) {
                buckets.entry(key).or_default().push((series, point.clone()));
            }
        }
    }
    buckets
}

fn bucket(point: &PlotPoint, width: f64) -> Option<i128> {
    let x = point.x.as_f64().or_else(|| {
        let at = dokime::domain::model::time::Timestamp::new(point.x.as_str()?);
        Some(at.to_datetime().ok()?.unix_timestamp_nanos() as f64 / 1e9)
    })?;
    Some((x / width).floor() as i128)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::runtime::query::{PlotSeries, PlotWindow};

    fn envelope(points: &[(f64, f64)]) -> PlotPayloadEnvelope {
        PlotPayloadEnvelope {
            plot_id: "signal".into(),
            revision: 2,
            diagnostics: Default::default(),
            warnings: Vec::new(),
            payload: PlotPayload {
                computed: None,
                stream: "signal".into(),
                x_key: "timestamp".into(),
                series_by: None,
                context: BTreeMap::new(),
                series: vec![PlotSeries {
                    key: "value".into(),
                    label: "Value".into(),
                    unit: None,
                    raw_points: points.len(),
                    downsampled: false,
                    format: None,
                    points: points
                        .iter()
                        .map(|(x, y)| PlotPoint {
                            x: serde_json::json!(x),
                            y: Some(*y),
                            z: None,
                        })
                        .collect(),
                }],
                cartesian: None,
                analysis: None,
                markers: Vec::new(),
                max_points: 100,
                window_s: Some(30.0),
                generated_at: String::new(),
                x_format: None,
                run_started_at: None,
                cycle_started_at: None,
                summaries: Vec::new(),
                bucket_width_s: Some(10.0),
                bucket_window: Some(PlotWindow { from: 0.0, to: 30.0 }),
                replaced_buckets: Vec::new(),
                bucket_counts: BTreeMap::new(),
            },
        }
    }

    #[test]
    fn bucket_deltas_replace_late_samples_and_expire_old_buckets() {
        let old = envelope(&[(0.0, 1.0), (1.0, 2.0), (10.0, 3.0), (11.0, 4.0)]);
        let new = envelope(&[(0.0, 1.0), (1.0, 2.0), (5.0, -9.0), (10.0, 3.0), (11.0, 4.0), (20.0, 7.0)]);
        let request = PlotPayloadRequest {
            plot_type: Some(dokime::document::PlotType::TimeSeries),
            ..Default::default()
        };
        let delta = between(&old.payload, ReadBatch::new(new.clone()), &request).unwrap().unwrap();
        assert_eq!(delta.payload.replaced_buckets, vec![0.0, 2.0]);
        assert_eq!(
            delta.payload.series[0].points.iter().map(|point| point.x.as_f64().unwrap()).collect::<Vec<_>>(),
            vec![0.0, 1.0, 5.0, 20.0]
        );
        let expired = envelope(&[(10.0, 3.0), (11.0, 4.0), (20.0, 7.0)]);
        let delta = between(&new.payload, ReadBatch::new(expired), &request).unwrap().unwrap();
        assert_eq!(delta.payload.replaced_buckets, vec![0.0]);
        assert!(delta.payload.series[0].points.is_empty());
        let delta = between(&new.payload, ReadBatch::new(new.clone()), &request).unwrap().unwrap();
        assert!(delta.payload.replaced_buckets.is_empty());
        assert!(delta.payload.series[0].points.is_empty());
        let mut interior = new.clone();
        interior.payload.bucket_counts.insert(0, 5);
        let delta = between(&new.payload, ReadBatch::new(interior), &request).unwrap().unwrap();
        assert_eq!(delta.payload.replaced_buckets, vec![0.0]);
        assert_eq!(delta.payload.series[0].points.len(), 3);
        let mut resized = new;
        resized.payload.bucket_width_s = Some(1.0);
        assert!(between(&old.payload, ReadBatch::new(resized), &request).unwrap().is_none());
    }
}
