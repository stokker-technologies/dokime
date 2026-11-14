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
use dokime::{encoding::Message, v1};
use dokime_storage::{Execution, Sample, Store};

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
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store = Store::open(root.path())?;
        let catalog = harmos_protocol::v1::Catalog {
            types: Some(prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),
            streams: vec![harmos_protocol::v1::StreamDeclaration {
                id: "sensor".into(),
                row: "dokime.client.v1.plots.PlotDomain".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        store.create(&Execution {
            identifier: "telemetry".into(),
            cycle: "sample-cycle".into(),
            origin: Some(v1::Origin {
                kind: Some(v1::origin::Kind::Import(v1::Import::default())),
            }),
            catalog: catalog.encode_to_vec(),
            ..Default::default()
        })?;
        store.open_stream("telemetry", "sensor")?;
        let samples = (0..2400)
            .map(|index| Sample {
                stream: "sensor".into(),
                source_ns: index * 1_000_000_000,
                sequence: index as u64 + 1,
                row: dokime_protocol::client::plots::PlotDomain { min: index as f64, max: -(index as f64) }.encode_to_vec(),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        store.sample_batch("telemetry", &samples)?;
        store.close_streams("telemetry")?;
        store.close("telemetry", v1::Termination::Completed, v1::ExecutionOutcome::default())?;
        store.seal("telemetry")?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut telemetry = dokime_protocol::client::evidence::samples_client::SamplesClient::new(channel);
        let scope = Some(v1::EvidenceScope {
            execution_id: Some("telemetry".into()),
            ..Default::default()
        });
        let mut request = dokime_protocol::client::evidence::SamplesListRequest {
            scope: scope.clone(),
            streams: vec!["sensor".into()],
            page: Some(harmos_protocol::v1::PageRequest { page_size: 137, ..Default::default() }),
            ..Default::default()
        };
        let mut read = Vec::new();
        let mut first = None;
        loop {
            let page = telemetry.list(request.clone()).await?.into_inner();
            assert!(page.samples.len() <= 137);
            if first.is_none() {
                first = Some(page.clone());
            }
            read.extend(page.samples);
            if page.next_page_token.is_empty() {
                break;
            }
            request.at = page.resolved_at;
            request.page.as_mut().unwrap().page_token = page.next_page_token;
        }
        let raw = read
            .iter()
            .map(|row| match row.kind.as_ref().unwrap() {
                dokime_protocol::client::evidence::sample_row::Kind::Raw(sample) => sample,
                _ => panic!("Expected Raw Sample"),
            })
            .collect::<Vec<_>>();
        assert_eq!(raw.len(), samples.len());
        assert_eq!(
            raw.iter().map(|sample| sample.payload.clone()).collect::<Vec<_>>(),
            samples.iter().map(|sample| sample.row.clone()).collect::<Vec<_>>()
        );
        let page = telemetry
            .list(dokime_protocol::client::evidence::SamplesListRequest {
                scope: scope.clone(),
                between: Some(v1::Window {
                    start: Some(prost_types::Timestamp { seconds: 90, nanos: 0 }),
                    end: Some(prost_types::Timestamp { seconds: 92, nanos: 0 }),
                }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(
            page.samples
                .iter()
                .map(|row| match row.kind.as_ref().unwrap() {
                    dokime_protocol::client::evidence::sample_row::Kind::Raw(sample) => sample.sequence,
                    _ => panic!("Expected Raw Sample"),
                })
                .collect::<Vec<_>>(),
            vec![91, 92, 93]
        );
        let tail = telemetry
            .list(dokime_protocol::client::evidence::SamplesListRequest {
                scope: scope.clone(),
                latest: true,
                page: Some(harmos_protocol::v1::PageRequest { page_size: 3, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(
            tail.samples
                .iter()
                .map(|row| match row.kind.as_ref().unwrap() {
                    dokime_protocol::client::evidence::sample_row::Kind::Raw(sample) => sample.sequence,
                    _ => panic!("Expected Raw Sample"),
                })
                .collect::<Vec<_>>(),
            vec![2398, 2399, 2400]
        );
        let mut changed = request.clone();
        changed.scope.as_mut().unwrap().execution_id = Some("foreign".into());
        assert!(telemetry.list(changed).await.is_err());
        let mut watch = telemetry
            .watch(dokime_protocol::client::evidence::SamplesWatchRequest {
                scope: scope.clone(),
                page: Some(harmos_protocol::v1::PageRequest { page_size: 127, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        let mut count = 0;
        while let Some(frame) = watch.message().await? {
            assert!(frame.opening);
            assert!(frame.resync_required.is_none());
            assert!(frame.samples.len() <= 127);
            count += frame.samples.len();
        }
        assert_eq!(count, 2400);
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut telemetry = dokime_protocol::client::evidence::samples_client::SamplesClient::connect(process.url.clone()).await?;
        let first = first.unwrap();
        let replay = telemetry
            .list(dokime_protocol::client::evidence::SamplesListRequest {
                scope,
                streams: vec!["sensor".into()],
                at: first.resolved_at.clone(),
                page: Some(harmos_protocol::v1::PageRequest { page_size: 137, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(replay, first);
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
