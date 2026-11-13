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

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::{encoding::Message, v1};
use dokime_storage::{Evidence, Execution, Store};

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
async fn client_browses_execution_evidence() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Prepare Two Independent Imported Executions with Frozen Schemas
        let root = tempfile::tempdir()?;
        let credential_root = tempfile::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store = Store::open(root.path())?;
        let contracts = v1::RecordContracts {
            records: vec![v1::RecordContract {
                metadata: Some(v1::Metadata {
                    identifier: "reading".into(),
                    label: "Reading".into(),
                    ..Default::default()
                }),
                message: "dokime.context.v1.evidence.LogsAppendRequest".into(),
            }],
        };
        let catalog = harmos_protocol::v1::Catalog {
            types: Some(prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),
            sections: vec![harmos_protocol::v1::Section {
                r#type: "dokime.v1.RecordContracts".into(),
                payload: contracts.encode_to_vec(),
            }],
            ..Default::default()
        };
        for id in ["evidence-a", "evidence-b"] {
            store.create(&Execution {
                identifier: id.into(),
                cycle: "evidence-cycle".into(),
                origin: Some(v1::Origin {
                    kind: Some(v1::origin::Kind::Import(v1::Import::default())),
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
                        payload: dokime_protocol::context::evidence::RecordsAppendRequest {
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
                    payload: dokime_protocol::context::evidence::LogsAppendRequest {
                        execution: id.into(),
                        severity: v1::Severity::Warning as i32,
                        message: "Review the sensor".into(),
                        ..Default::default()
                    }
                    .encode_to_vec(),
                    ..Default::default()
                },
            )?;
            store.close_streams(id)?;
            store.close(id, v1::Termination::Completed, v1::ExecutionOutcome::default())?;
            store.seal(id)?;
        }
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = dokime::client::HostClient::new(channel.clone(), super::support::client_settings())?;
        let identity = client
            .status
            .get(tonic::Request::new(harmos_protocol::v1::StatusGetRequest {}))
            .await?
            .into_inner()
            .status
            .unwrap()
            .identity
            .unwrap();
        let mut evidence = v1::evidence_client::EvidenceClient::new(channel);

        // Resolve a Captured Schema Without Connecting Its Plugin
        let schema = evidence
            .get_captured_record_schema(v1::EvidenceGetCapturedRecordSchemaRequest {
                record_type: "reading".into(),
                schema_version: 1,
                epoch: identity.host_id.clone(),
            })
            .await?
            .into_inner();
        assert_eq!(schema.record_type, "reading");
        assert!(schema.fields.iter().any(|field| field.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "message")
            && field.value_type == v1::ValueType::Text as i32));

        // Capture a Fixed Cursor and Read a Tail Isolated to One Execution
        let selection = v1::CapturedRecordSelection {
            scope: Some(v1::EvidenceScope {
                execution_id: Some("evidence-a".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let page = evidence
            .list_records(v1::EvidenceListRecordsRequest {
                selection: Some(selection.clone()),
                latest: 3,
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(page.records.iter().map(|record| record.payload.clone()).collect::<Vec<_>>(), vec![vec![4], vec![5], vec![6]]);
        assert!(page.replay_complete);
        let empty = evidence
            .list_records(v1::EvidenceListRecordsRequest {
                selection: Some(selection.clone()),
                after: page.scanned.clone(),
                through: page.through.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert!(empty.records.is_empty() && empty.replay_complete);

        // Reject Cursors Reused for Another Selection or Future Position
        let mut wrong = page.scanned.clone().unwrap();
        wrong.position += 1;
        assert!(
            evidence
                .list_records(v1::EvidenceListRecordsRequest {
                    selection: Some(selection.clone()),
                    after: Some(wrong),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        assert!(
            evidence
                .list_records(v1::EvidenceListRecordsRequest {
                    selection: Some(v1::CapturedRecordSelection::default()),
                    after: page.scanned,
                    ..Default::default()
                })
                .await
                .is_err()
        );

        let log_schema = evidence
            .get_captured_record_schema(v1::EvidenceGetCapturedRecordSchemaRequest {
                record_type: "dokime.log".into(),
                schema_version: 1,
                epoch: identity.host_id.clone(),
            })
            .await?
            .into_inner();
        assert!(log_schema.fields.iter().any(|field| field.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "message")));

        // Read Logs Through Their Explicit Evidence Type
        let logs = evidence
            .list_records(v1::EvidenceListRecordsRequest {
                selection: Some(v1::CapturedRecordSelection {
                    record_types: vec!["dokime.log".into()],
                    severities: vec![v1::Severity::Warning as i32],
                    ..selection.clone()
                }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(logs.records.len(), 1);
        assert_eq!(logs.records[0].message.as_deref(), Some("Review the sensor"));

        // Replay a Sealed Execution and Observe Its Terminal Envelope
        let mut watch = evidence
            .watch_records(v1::EvidenceWatchRecordsRequest {
                selection: Some(selection),
                ..Default::default()
            })
            .await?
            .into_inner();
        let mut count = 0;
        let mut terminal = false;
        while let Some(frame) = watch.message().await? {
            match frame.value.unwrap() {
                v1::evidence_watch_records_output::Value::Item(page) => count += page.records.len(),
                v1::evidence_watch_records_output::Value::Response(_) => terminal = true,
            }
        }
        assert_eq!(count, 7);
        assert!(terminal);

        // Invoke the Same Operations Through Their Bidirectional Envelopes
        let mut run = evidence
            .run_get_captured_record_schema(tokio_stream::iter([v1::EvidenceGetCapturedRecordSchemaRunInput {
                value: Some(v1::evidence_get_captured_record_schema_run_input::Value::Request(
                    v1::EvidenceGetCapturedRecordSchemaRequest {
                        record_type: "reading".into(),
                        schema_version: 1,
                        epoch: identity.host_id,
                    },
                )),
            }]))
            .await?
            .into_inner();
        assert!(matches!(
            run.message().await?.unwrap().value,
            Some(v1::evidence_get_captured_record_schema_run_output::Value::Response(_))
        ));
        assert!(run.message().await?.is_none());
        let mut run = evidence
            .run_list_records(tokio_stream::iter([v1::EvidenceListRecordsRunInput {
                value: Some(v1::evidence_list_records_run_input::Value::Request(v1::EvidenceListRecordsRequest {
                    latest: 2,
                    ..Default::default()
                })),
            }]))
            .await?
            .into_inner();
        assert!(matches!(run.message().await?.unwrap().value, Some(v1::evidence_list_records_run_output::Value::Response(_))));
        assert!(run.message().await?.is_none());
        let mut run = evidence
            .run_watch_records(tokio_stream::iter([v1::EvidenceWatchRecordsRunInput {
                value: Some(v1::evidence_watch_records_run_input::Value::Request(v1::EvidenceWatchRecordsRequest {
                    selection: Some(v1::CapturedRecordSelection {
                        scope: Some(v1::EvidenceScope {
                            execution_id: Some("evidence-b".into()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
            }]))
            .await?
            .into_inner();
        let mut terminal = false;
        while let Some(frame) = run.message().await? {
            terminal |= matches!(frame.value, Some(v1::evidence_watch_records_run_output::Value::Response(_)));
        }
        assert!(terminal);

        // Refuse Ambiguous Wire Shapes Even When Names and Field Types Match
        let mut changed = catalog.clone();
        for file in &mut changed.types.as_mut().unwrap().file {
            if file.package.as_deref() != Some("dokime.context.v1.evidence") {
                continue;
            }
            for message in &mut file.message_type {
                if message.name.as_deref() != Some("LogsAppendRequest") {
                    continue;
                }
                for field in &mut message.field {
                    if field.name.as_deref() == Some("message") {
                        field.number = Some(17);
                    }
                }
            }
        }
        store.create(&Execution {
            identifier: "evidence-conflict".into(),
            cycle: "evidence-cycle".into(),
            origin: Some(v1::Origin {
                kind: Some(v1::origin::Kind::Import(v1::Import::default())),
            }),
            catalog: changed.encode_to_vec(),
            ..Default::default()
        })?;
        assert!(
            evidence
                .get_captured_record_schema(v1::EvidenceGetCapturedRecordSchemaRequest {
                    record_type: "reading".into(),
                    schema_version: 1,
                    ..Default::default()
                })
                .await
                .is_err()
        );
        client.close();
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
