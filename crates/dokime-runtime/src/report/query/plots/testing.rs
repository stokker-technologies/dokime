/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Original Report Points Remain Available For Zoom At Both Lane Volumes

use super::*;
use crate::Value;
use crate::model::identity::new_id;
use crate::telemetry::TelemetrySample;
use dokime_domain::model::time::Timestamp;

pub async fn report_points(count: usize) {
    let timestamp = Timestamp::new("2026-10-21T00:00:00Z");
    let stream = Identifier::new("fixture.capture").unwrap();
    let input = crate::plot::spilled::Input::new().unwrap();
    for index in 0..count {
        let mut sample = TelemetrySample::new(new_id(), stream.clone(), index as u64, timestamp.clone(), "fixture");
        let value = if index == 25 { 1_000_000.0 } else { index as f64 };
        sample.values.insert("index".into(), Value::Number(index as f64));
        sample.values.insert("value".into(), Value::Number(value));
        input.append(&sample).unwrap();
    }
    let request = PlotPayloadRequest {
        stream: stream.to_string(),
        x_key: Some("index".into()),
        y_keys: vec!["value".into()],
        plot_type: Some(PlotType::TimeSeries),
        max_points: Some(32),
        technique: crate::plot::DownsampleTechnique::MinMax,
        ..Default::default()
    };

    let context = PlotPayloadContext {
        default_max_points: 2000,
        x_format: None,
        series_format: None,
        run_started_at: None,
        cycle_started_at: None,
    };
    let cache = ReadCache::new();
    let mut plots = BTreeMap::new();
    for index in 0..8 {
        let payload = cache.capture(&input, &request, timestamp.as_str(), &context).await.unwrap();
        let metadata = payload.try_map(|payload| Ok::<_, crate::runtime::Error>((index, payload))).unwrap();
        metadata
            .try_map(|(id, payload)| {
                plots.insert(id, payload);
                Ok::<_, crate::runtime::Error>(())
            })
            .unwrap();
    }
    let owner = cache.captured.get().unwrap();
    assert_eq!(Arc::strong_count(owner), 9);
    for payload in plots.values() {
        let crate::report::value::Value::Shared(value) = payload else { panic!("shared plot owner") };
        assert!(Arc::ptr_eq(value, owner));
    }
    let payload = serde_json::to_value(&plots[&0]).unwrap();
    let series = &payload["series"][0];
    assert_eq!(series["points"].as_array().unwrap().len(), count);
    assert_eq!(series["raw_points"], count);
    assert_eq!(series["downsampled"], false);
    assert!(series["points"].as_array().unwrap().iter().any(|point| point["y"] == 1_000_000.0));
    assert_eq!(payload["max_points"], 32);

    // Shared Ownership Is Transparent To JSON And Exact Integer Import/Export.
    let exact = crate::report::value::Value::Object(BTreeMap::from([
        ("signed".into(), crate::report::value::Value::Integer(i64::MIN)),
        ("unsigned".into(), crate::report::value::Value::Unsigned(u64::MAX)),
    ]));
    let shared = crate::report::value::Value::Shared(Arc::new(exact.clone()));
    let bytes = serde_json::to_vec(&shared).unwrap();
    assert_eq!(bytes, serde_json::to_vec(&exact).unwrap());
    let restored: crate::report::value::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, shared);
    assert_eq!(shared, restored);
}
