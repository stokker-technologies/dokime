/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A Plugin Duration Setting Overrides the Host's Interval Window for One Stream

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, error::Error};

// Third Party Imports

// Local Imports
use super::support;
use dokime_protocol::model::{Value, value};
use dokime_runtime::Host;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn window_setting_overrides_host_window() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        // Both 1 kHz Streams Store Each Sample With the Sample Count of the Window That Granted It.
        let root = support::tempdir()?;
        let host = Host::open(root.path(), support::settings(), support::service_settings()).await?;
        let entry = host.add_managed("boundary", &support::boundary_fixture()?, &[]).await?;
        let rows = HashMap::from([(
            "rows".into(),
            Value {
                kind: Some(value::Kind::UnsignedInteger(1000)),
            },
        )]);
        let origin = host.materialize(&entry, "execution.windows", rows).await?;
        let cycle = host.start_cycle(vec![origin]).await?.wait().await?;
        let mut granted = HashMap::<String, Vec<f64>>::new();
        for sample in &cycle.executions[0].samples {
            let reading = <support::Reading as prost::Message>::decode(sample.row.as_slice())?;
            granted.entry(sample.stream.clone()).or_default().push(reading.value);
        }

        // The Host Grants Its Interval Window; the Plugin's 10 ms Sample Window Setting Grants Its Stream 10 ms Ones.
        let host_count = (dokime_runtime::INTERVAL_WINDOW.as_millis()) as f64;
        let outcome = &cycle.executions[0].execution.outcome;
        let host_window = granted.get("execution.window.host").unwrap_or_else(|| panic!("Sample the Host Window Stream: {outcome:?}"));
        let setting_window =
            granted.get("execution.window.setting").unwrap_or_else(|| panic!("Sample the Setting Window Stream: {outcome:?}"));
        assert_ne!(host_count, 10.0, "Compare Window Lengths: The Host Window Must Differ From the Setting");
        assert_eq!(host_window.len(), 1000, "Sample the Host Window Stream: Retain Every Sample");
        assert_eq!(setting_window.len(), 1000, "Sample the Setting Window Stream: Retain Every Sample");
        assert!(host_window.iter().all(|count| *count == host_count), "Grant the Host Window: {host_window:?}");
        assert!(setting_window.iter().all(|count| *count == 10.0), "Grant the Setting Window: {setting_window:?}");
        host.shutdown().await?;
        Ok(())
    }))
}
