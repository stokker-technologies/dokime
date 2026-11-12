/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Client Reads Bounded Telemetry

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::{v1, encoding::Message};
use dokime_storage::{Store, Execution, Sample};


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

#[tokio::test]
async fn client_reads_bounded_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Retain Exact Sample Bytes and Frozen Numeric Descriptors
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let catalog = harmos_protocol::v1::Catalog {
            types: Some(prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),
            streams: vec![harmos_protocol::v1::StreamDeclaration { id: "sensor".into(), row: "dokime.v1.PlotDomain".into(), ..Default::default() }],
            ..Default::default()
        };
        store.create(&Execution { identifier: "telemetry".into(), cycle: "sample-cycle".into(),
            origin: Some(v1::Origin { kind: Some(v1::origin::Kind::Import(v1::Import::default())) }),
            catalog: catalog.encode_to_vec(), ..Default::default() })?;
        store.open_stream("telemetry", "sensor")?;
        let samples = (0..2400).map(|index| Sample { stream: "sensor".into(), source_ns: index * 1_000_000_000,
            sequence: index as u64 + 1, row: v1::PlotDomain { min: index as f64, max: -(index as f64) }.encode_to_vec(), ..Default::default() }).collect::<Vec<_>>();
        store.sample_batch("telemetry", &samples)?;
        store.close_streams("telemetry")?;
        store.close("telemetry", v1::Termination::Completed, v1::ExecutionOutcome::default())?;
        store.seal("telemetry")?;
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = dokime::client::HostClient::new(channel.clone());
        let mut telemetry = v1::telemetry_client::TelemetryClient::new(channel);
        let scope = Some(v1::EvidenceScope { execution_id: Some("telemetry".into()), ..Default::default() });

        // Read Bounded Forward Pages Without Losing or Repeating a Row
        let mut request = v1::TelemetryListSamplesRequest { scope: scope.clone(), streams: vec!["sensor".into()], limit: 137, ..Default::default() };
        let mut read = Vec::new();
        loop {
            let page = telemetry.list_samples(request.clone()).await?.into_inner();
            assert!(page.samples.len() <= 137);
            read.extend(page.samples);
            if page.complete { break; }
            request.after_stream_cursors = page.through_stream_cursors.into_iter().map(|cursor| v1::ListTelemetrySamplesRequestAfterStreamCursorsEntry { key: cursor.key, value: cursor.value }).collect();
        }
        assert_eq!(read.len(), samples.len());
        assert_eq!(read.iter().map(|sample| sample.payload.clone()).collect::<Vec<_>>(), samples.iter().map(|sample| sample.row.clone()).collect::<Vec<_>>());

        // Read an Inclusive Source-Time Window and a Latest Tail
        let page = telemetry.list_samples(v1::TelemetryListSamplesRequest { scope: scope.clone(), start: Some(prost_types::Timestamp { seconds: 90, nanos: 0 }), end: Some(prost_types::Timestamp { seconds: 92, nanos: 0 }), ..Default::default() }).await?.into_inner();
        assert_eq!(page.samples.iter().map(|sample| sample.sequence).collect::<Vec<_>>(), vec![91,92,93]);
        let tail = telemetry.list_samples(v1::TelemetryListSamplesRequest { scope: scope.clone(), latest: true, limit: 3, ..Default::default() }).await?.into_inner();
        assert_eq!(tail.samples.iter().map(|sample| sample.sequence).collect::<Vec<_>>(), vec![2398,2399,2400]);
        assert!(telemetry.list_samples(v1::TelemetryListSamplesRequest { scope: scope.clone(), latest: true, after_stream_cursors: request.after_stream_cursors, ..Default::default() }).await.is_err());

        // Replay Every Page and Observe the Terminal Response
        let mut watch = telemetry.watch_samples(v1::TelemetryWatchSamplesRequest { selection: Some(v1::ListSamples { scope: scope.clone(), limit: 127, ..Default::default() }), follow: true }).await?.into_inner();
        let mut count = 0;
        let mut terminal = false;
        while let Some(frame) = watch.message().await? {
            match frame.value.unwrap() {
                v1::telemetry_watch_samples_output::Value::Item(page) => { assert!(page.samples.len() <= 127); count += page.samples.len(); }
                v1::telemetry_watch_samples_output::Value::Response(_) => terminal = true,
            }
        }
        assert_eq!(count, 2400);
        assert!(terminal);
        client.close();
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    }).await?
}

