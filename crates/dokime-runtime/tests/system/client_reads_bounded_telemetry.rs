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
use dokime::Client;
use dokime::EvidenceScope;
use dokime::Import;
use dokime::Origin;
use dokime::Termination;
use dokime::Window;
use dokime::client::EvidenceSamplesListOptions;
use dokime::client::EvidenceSamplesWatchOptions;
use dokime::client::ExecutionOutcome;
use dokime::client::PlotsPlotDomain;
use dokime::encoding::Message;
use dokime_protocol::FILE_DESCRIPTOR_SET;
use dokime_protocol::host::evidence::SamplesListRequest;
use dokime_protocol::host::evidence::sample_row::Kind as SampleRowKind;
use dokime_protocol::model::origin::Kind as OriginKind;
use dokime_storage::Execution;
use dokime_storage::Sample;
use dokime_storage::Store;
use harmos_protocol::model::Declarations;
use harmos_protocol::model::PageRequest;
use harmos_protocol::model::StreamDeclaration;
use prost_types::FileDescriptorSet;
use prost_types::Timestamp;
use std::error::Error;
use tonic::transport::Endpoint;

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
async fn client_reads_bounded_telemetry() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Retain Exact Sample Bytes and Frozen Numeric Descriptors
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store = Store::open(root.path())?;
        let catalog = Declarations {
            types: Some(FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?),
            streams: vec![StreamDeclaration {
                id: "sensor".into(),
                row: "dokime.host.v1.plots.PlotDomain".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        store.create(&Execution {
            identifier: "telemetry".into(),
            cycle: "sample-cycle".into(),
            origin: Some(Origin {
                kind: Some(OriginKind::Import(Import::default())),
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
                row: PlotsPlotDomain { min: index as f64, max: -(index as f64) }.encode_to_vec(),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        store.sample_batch("telemetry", &samples)?;
        store.close_streams("telemetry")?;
        store.close("telemetry", Termination::Completed, ExecutionOutcome::default())?;
        store.seal("telemetry")?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let telemetry = Client::from_channel(channel, super::support::client_settings())?.evidence.samples.clone();
        let scope = Some(EvidenceScope {
            execution_id: Some("telemetry".into()),
            ..Default::default()
        });
        let mut request = SamplesListRequest {
            scope: scope.clone(),
            streams: vec!["sensor".into()],
            page: Some(PageRequest { page_size: 137, ..Default::default() }),
            ..Default::default()
        };
        let mut read = Vec::new();
        let mut first = None;
        loop {
            let page = telemetry
                .list_with(
                    ((request.clone()).scope).unwrap_or_default(),
                    (request.clone()).streams,
                    EvidenceSamplesListOptions {
                        versions: Some((request.clone()).versions),
                        fields: Some((request.clone()).fields),
                        sources: Some((request.clone()).sources),
                        signals: Some((request.clone()).signals),
                        between: Some(((request.clone()).between).unwrap_or_default()),
                        bucket: Some(((request.clone()).bucket).unwrap_or_default()),
                        latest: Some((request.clone()).latest),
                        at: (request.clone()).at,
                        page: (request.clone()).page,
                        ..Default::default()
                    },
                )
                .await?;
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
                SampleRowKind::Raw(sample) => sample,
                _ => panic!("Expected Raw Sample"),
            })
            .collect::<Vec<_>>();
        assert_eq!(raw.len(), samples.len());
        assert_eq!(
            raw.iter().map(|sample| sample.payload.clone()).collect::<Vec<_>>(),
            samples.iter().map(|sample| sample.row.clone()).collect::<Vec<_>>()
        );
        let page = telemetry
            .list_with(
                (scope.clone()).unwrap_or_default(),
                Default::default(),
                EvidenceSamplesListOptions {
                    versions: Some(Default::default()),
                    fields: Some(Default::default()),
                    sources: Some(Default::default()),
                    signals: Some(Default::default()),
                    between: Some(Window {
                        start: Some(Timestamp { seconds: 90, nanos: 0 }),
                        end: Some(Timestamp { seconds: 92, nanos: 0 }),
                    }),
                    bucket: Some(Default::default()),
                    latest: Some(Default::default()),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(
            page.samples
                .iter()
                .map(|row| match row.kind.as_ref().unwrap() {
                    SampleRowKind::Raw(sample) => sample.sequence,
                    _ => panic!("Expected Raw Sample"),
                })
                .collect::<Vec<_>>(),
            vec![91, 92, 93]
        );
        let tail = telemetry
            .list_with(
                (scope.clone()).unwrap_or_default(),
                Default::default(),
                EvidenceSamplesListOptions {
                    versions: Some(Default::default()),
                    fields: Some(Default::default()),
                    sources: Some(Default::default()),
                    signals: Some(Default::default()),
                    between: Some(Default::default()),
                    bucket: Some(Default::default()),
                    latest: Some(true),
                    page: Some(PageRequest { page_size: 3, ..Default::default() }),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(
            tail.samples
                .iter()
                .map(|row| match row.kind.as_ref().unwrap() {
                    SampleRowKind::Raw(sample) => sample.sequence,
                    _ => panic!("Expected Raw Sample"),
                })
                .collect::<Vec<_>>(),
            vec![2398, 2399, 2400]
        );
        let mut changed = request.clone();
        changed.scope.as_mut().unwrap().execution_id = Some("foreign".into());
        assert!(
            telemetry
                .list_with(
                    ((changed.clone()).scope).unwrap_or_default(),
                    (changed.clone()).streams,
                    EvidenceSamplesListOptions {
                        versions: Some((changed.clone()).versions),
                        fields: Some((changed.clone()).fields),
                        sources: Some((changed.clone()).sources),
                        signals: Some((changed.clone()).signals),
                        between: Some(((changed.clone()).between).unwrap_or_default()),
                        bucket: Some(((changed.clone()).bucket).unwrap_or_default()),
                        latest: Some((changed.clone()).latest),
                        at: (changed.clone()).at,
                        page: (changed.clone()).page,
                        ..Default::default()
                    }
                )
                .await
                .is_err()
        );
        let mut watch = telemetry
            .watch_with(
                (scope.clone()).unwrap_or_default(),
                Default::default(),
                EvidenceSamplesWatchOptions {
                    versions: Some(Default::default()),
                    fields: Some(Default::default()),
                    sources: Some(Default::default()),
                    signals: Some(Default::default()),
                    between: Some(Default::default()),
                    bucket: Some(Default::default()),
                    latest: Some(Default::default()),
                    page: Some(PageRequest { page_size: 127, ..Default::default() }),
                    ..Default::default()
                },
            )
            .await?;
        let mut count = 0;
        while let Some(frame) = watch.next().await? {
            assert!(frame.opening);
            assert!(frame.resync_required.is_none());
            assert!(frame.samples.len() <= 127);
            count += frame.samples.len();
        }
        assert_eq!(count, 2400);
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let telemetry =
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .evidence
                .samples
                .clone();
        let first = first.unwrap();
        let replay = telemetry
            .list_with(
                (scope).unwrap_or_default(),
                vec!["sensor".into()],
                EvidenceSamplesListOptions {
                    versions: Some(Default::default()),
                    fields: Some(Default::default()),
                    sources: Some(Default::default()),
                    signals: Some(Default::default()),
                    between: Some(Default::default()),
                    bucket: Some(Default::default()),
                    latest: Some(Default::default()),
                    at: first.resolved_at.clone(),
                    page: Some(PageRequest { page_size: 137, ..Default::default() }),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(replay, first);
        process.close().await?;
        Ok::<(), Box<dyn Error>>(())
    })
    .await?
}
