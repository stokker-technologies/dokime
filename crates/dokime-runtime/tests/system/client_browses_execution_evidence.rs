/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Browses Execution Evidence

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use dokime::client::EvidenceRecordsWatchOptions;

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::Client;
use dokime::EvidenceCapturedRecordSelection;
use dokime::EvidenceScope;
use dokime::Import;
use dokime::Metadata;
use dokime::Origin;
use dokime::RecordContract;
use dokime::RecordContracts;
use dokime::Severity;
use dokime::Termination;
use dokime::ValueType;
use dokime::client::EvidenceRecordsListOptions;
use dokime::client::ExecutionOutcome;
use dokime::encoding::Message;
use dokime_protocol::FILE_DESCRIPTOR_SET;
use dokime_protocol::host::evidence::LogsAppendRequest;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence::RecordsListRequest;
use dokime_protocol::model::origin::Kind as OriginKind;
use dokime_storage::Evidence;
use dokime_storage::Execution;
use dokime_storage::Store;
use harmos_protocol::model::Declarations;
use harmos_protocol::model::PageRequest;
use harmos_protocol::model::Section;
use prost_types::FileDescriptorSet;
use std::collections::BTreeSet;
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

#[test]
fn client_browses_execution_evidence() -> Result<(), Box<dyn Error>> {
    super::support::run(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Prepare Two Independent Imported Executions with Frozen Schemas
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let store = Store::open(root.path())?;
            let contracts = RecordContracts {
                records: vec![RecordContract {
                    metadata: Some(Metadata {
                        identifier: "reading".into(),
                        label: "Reading".into(),
                        ..Default::default()
                    }),
                    message: "dokime.host.v1.evidence.LogsAppendRequest".into(),
                }],
            };
            let catalog = Declarations {
                types: Some(FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?),
                sections: vec![Section {
                    r#type: "dokime.model.v1.RecordContracts".into(),
                    payload: contracts.encode_to_vec(),
                }],
                ..Default::default()
            };
            for id in ["evidence-a", "evidence-b"] {
                store.create(&Execution {
                    identifier: id.into(),
                    cycle: "evidence-cycle".into(),
                    origin: Some(Origin {
                        kind: Some(OriginKind::Import(Import::default())),
                    }),
                    catalog: catalog.encode_to_vec(),
                    ..Default::default()
                })?;
                for index in 0..7 {
                    store.append(
                        id,
                        Evidence {
                            kind: "record:reading".into(),
                            step: "capture".into(),
                            payload: RecordsAppendRequest {
                                execution: id.into(),
                                kind: "reading".into(),
                                payload: vec![index],
                                ..Default::default()
                            }
                            .encode_to_vec(),
                            ..Default::default()
                        },
                    )?;
                }
                store.append(
                    id,
                    Evidence {
                        kind: "log".into(),
                        payload: LogsAppendRequest {
                            execution: id.into(),
                            severity: Severity::Warning as i32,
                            message: "Review the sensor".into(),
                            ..Default::default()
                        }
                        .encode_to_vec(),
                        ..Default::default()
                    },
                )?;
                store.close_streams(id)?;
                store.close(id, Termination::Completed, ExecutionOutcome::default())?;
                store.seal(id)?;
            }
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let schemas = Client::from_channel(channel.clone(), super::support::client_settings())?.record_schemas.clone();
            let records = Client::from_channel(channel, super::support::client_settings())?.evidence.records.clone();
            let schema = schemas.get("reading", "evidence-a", 1).await?.value.unwrap();
            assert!(schema.fields.iter().any(|field| field.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "message")
                && field.value_type == ValueType::Text as i32));

            // Continue One Fixed Selection Without Repeating a Record.
            let selection = EvidenceCapturedRecordSelection {
                scope: Some(EvidenceScope {
                    execution_id: Some("evidence-a".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let mut request = RecordsListRequest {
                selection: Some(selection.clone()),
                page: Some(PageRequest { page_size: 3, ..Default::default() }),
                ..Default::default()
            };
            let first = records
                .list_with(
                    ((request.clone()).scope).unwrap_or_default(),
                    (request.clone()).record_type,
                    EvidenceRecordsListOptions {
                        versions: Some((request.clone()).versions),
                        between: Some(((request.clone()).between).unwrap_or_default()),
                        step: Some((request.clone()).step),
                        selection: Some(((request.clone()).selection).unwrap_or_default()),
                        at: (request.clone()).at,
                        page: (request.clone()).page,
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(first.records.iter().map(|record| record.payload.clone()).collect::<Vec<_>>(), vec![vec![0], vec![1], vec![2]]);
            let mut all = first.records.clone();
            let mut page = first.clone();
            while !page.next_page_token.is_empty() {
                request.at = page.resolved_at.clone();
                request.page.as_mut().unwrap().page_token = page.next_page_token;
                page = records
                    .list_with(
                        ((request.clone()).scope).unwrap_or_default(),
                        (request.clone()).record_type,
                        EvidenceRecordsListOptions {
                            versions: Some((request.clone()).versions),
                            between: Some(((request.clone()).between).unwrap_or_default()),
                            step: Some((request.clone()).step),
                            selection: Some(((request.clone()).selection).unwrap_or_default()),
                            at: (request.clone()).at,
                            page: (request.clone()).page,
                            ..Default::default()
                        },
                    )
                    .await?;
                all.extend(page.records.clone());
            }
            assert_eq!(all.len(), 7);
            assert!(all.iter().all(|record| record.scope.as_ref().unwrap().execution_id.as_deref() == Some("evidence-a")));
            assert_eq!(all.iter().map(|record| &record.record_id).collect::<BTreeSet<_>>().len(), 7);

            // Reject Foreign Histories and Changed Continuation Selections.
            let mut foreign = request.clone();
            foreign.at = first.resolved_at.clone();
            foreign.at.as_mut().unwrap().positions[0].history.as_mut().unwrap().store = "foreign".into();
            assert!(
                records
                    .list_with(
                        ((foreign.clone()).scope).unwrap_or_default(),
                        (foreign.clone()).record_type,
                        EvidenceRecordsListOptions {
                            versions: Some((foreign.clone()).versions),
                            between: Some(((foreign.clone()).between).unwrap_or_default()),
                            step: Some((foreign.clone()).step),
                            selection: Some(((foreign.clone()).selection).unwrap_or_default()),
                            at: (foreign.clone()).at,
                            page: (foreign.clone()).page,
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            let mut changed = request.clone();
            changed.at = first.resolved_at.clone();
            changed.page.as_mut().unwrap().page_token = first.next_page_token.clone();
            changed.selection = Some(EvidenceCapturedRecordSelection::default());
            assert!(
                records
                    .list_with(
                        ((changed.clone()).scope).unwrap_or_default(),
                        (changed.clone()).record_type,
                        EvidenceRecordsListOptions {
                            versions: Some((changed.clone()).versions),
                            between: Some(((changed.clone()).between).unwrap_or_default()),
                            step: Some((changed.clone()).step),
                            selection: Some(((changed.clone()).selection).unwrap_or_default()),
                            at: (changed.clone()).at,
                            page: (changed.clone()).page,
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            let mut tiny = request.clone();
            tiny.page.as_mut().unwrap().max_bytes = 1;
            assert!(
                records
                    .list_with(
                        ((tiny.clone()).scope).unwrap_or_default(),
                        (tiny.clone()).record_type,
                        EvidenceRecordsListOptions {
                            versions: Some((tiny.clone()).versions),
                            between: Some(((tiny.clone()).between).unwrap_or_default()),
                            step: Some((tiny.clone()).step),
                            selection: Some(((tiny.clone()).selection).unwrap_or_default()),
                            at: (tiny.clone()).at,
                            page: (tiny.clone()).page,
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );

            // Logs Are an Explicit Record Type, with Their Frozen Schema.
            let logs =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .evidence
                    .records
                    .clone()
                    .list_with(
                        Default::default(),
                        String::new(),
                        EvidenceRecordsListOptions {
                            versions: Some(Default::default()),
                            between: Some(Default::default()),
                            step: Some(String::new()),
                            selection: Some(EvidenceCapturedRecordSelection {
                                record_types: vec!["dokime.log".into()],
                                severities: vec![Severity::Warning as i32],
                                ..selection.clone()
                            }),
                            ..Default::default()
                        },
                    )
                    .await?;
            assert_eq!(logs.records.len(), 1);
            assert_eq!(logs.records[0].severity, Severity::Warning as i32);
            let schema = schemas.get("dokime.log", String::new(), 1).await?.value.unwrap();
            assert!(schema.fields.iter().any(|field| field.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "message")));

            // A Sealed Execution Finishes Its Bounded Opening with Successful Status.
            let mut watch =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .evidence
                    .records
                    .clone()
                    .watch_with(
                        Default::default(),
                        String::new(),
                        EvidenceRecordsWatchOptions {
                            versions: Some(Default::default()),
                            between: Some(Default::default()),
                            step: Some(String::new()),
                            selection: Some(selection.clone()),
                            ..Default::default()
                        },
                    )
                    .await?;
            let mut watched = Vec::new();
            while let Some(frame) = watch.next().await? {
                assert!(frame.opening);
                assert!(frame.resync_required.is_none());
                watched.extend(frame.records);
            }
            assert_eq!(watched, all);
            process.close().await?;

            // Returned Positions Continue to Resolve After Restart.
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;

            let replay =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .evidence
                    .records
                    .clone()
                    .list_with(
                        Default::default(),
                        String::new(),
                        EvidenceRecordsListOptions {
                            versions: Some(Default::default()),
                            between: Some(Default::default()),
                            step: Some(String::new()),
                            selection: Some(selection),
                            at: first.resolved_at.clone(),
                            page: Some(PageRequest { page_size: 3, ..Default::default() }),
                            ..Default::default()
                        },
                    )
                    .await?;
            assert_eq!(replay, first);
            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    })
}
