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
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
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
        let mut schemas = v1::catalog::record_schemas_client::RecordSchemasClient::new(channel.clone());
        let mut records = v1::evidence::records_client::RecordsClient::new(channel);
        let schema = schemas
            .get(v1::catalog::RecordSchemasGetRequest {
                name: "reading".into(),
                parent: "evidence-a".into(),
                schema_version: 1,
                ..Default::default()
            })
            .await?
            .into_inner()
            .value
            .unwrap();
        assert!(schema.fields.iter().any(|field| field.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "message")
            && field.value_type == v1::ValueType::Text as i32));

        // Continue One Fixed Selection Without Repeating a Record.
        let selection = v1::CapturedRecordSelection {
            scope: Some(v1::EvidenceScope {
                execution_id: Some("evidence-a".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut request = v1::evidence::RecordsListRequest {
            selection: Some(selection.clone()),
            page: Some(harmos_protocol::v1::PageRequest { page_size: 3, ..Default::default() }),
            ..Default::default()
        };
        let first = records.list(request.clone()).await?.into_inner();
        assert_eq!(first.records.iter().map(|record| record.payload.clone()).collect::<Vec<_>>(), vec![vec![0], vec![1], vec![2]]);
        let mut all = first.records.clone();
        let mut page = first.clone();
        while !page.next_page_token.is_empty() {
            request.at = page.resolved_at.clone();
            request.page.as_mut().unwrap().page_token = page.next_page_token;
            page = records.list(request.clone()).await?.into_inner();
            all.extend(page.records.clone());
        }
        assert_eq!(all.len(), 7);
        assert!(all.iter().all(|record| record.scope.as_ref().unwrap().execution_id.as_deref() == Some("evidence-a")));
        assert_eq!(all.iter().map(|record| &record.record_id).collect::<std::collections::BTreeSet<_>>().len(), 7);

        // Reject Foreign Histories and Changed Continuation Selections.
        let mut foreign = request.clone();
        foreign.at = first.resolved_at.clone();
        foreign.at.as_mut().unwrap().positions[0].history.as_mut().unwrap().store = "foreign".into();
        assert!(records.list(foreign).await.is_err());
        let mut changed = request.clone();
        changed.at = first.resolved_at.clone();
        changed.page.as_mut().unwrap().page_token = first.next_page_token.clone();
        changed.selection = Some(v1::CapturedRecordSelection::default());
        assert!(records.list(changed).await.is_err());
        let mut tiny = request.clone();
        tiny.page.as_mut().unwrap().max_bytes = 1;
        assert!(records.list(tiny).await.is_err());

        // Logs Are an Explicit Record Type, with Their Frozen Schema.
        let logs = v1::evidence::records_client::RecordsClient::connect(process.url.clone())
            .await?
            .list(v1::evidence::RecordsListRequest {
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
        assert_eq!(logs.records[0].severity, v1::Severity::Warning as i32);
        let schema = schemas
            .get(v1::catalog::RecordSchemasGetRequest {
                name: "dokime.log".into(),
                schema_version: 1,
                ..Default::default()
            })
            .await?
            .into_inner()
            .value
            .unwrap();
        assert!(schema.fields.iter().any(|field| field.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "message")));

        // A Sealed Execution Finishes Its Bounded Opening with Successful Status.
        let mut watch = v1::evidence::records_client::RecordsClient::connect(process.url.clone())
            .await?
            .watch(v1::evidence::RecordsWatchRequest {
                selection: Some(selection.clone()),
                ..Default::default()
            })
            .await?
            .into_inner();
        let mut watched = Vec::new();
        while let Some(frame) = watch.message().await? {
            assert!(frame.opening);
            assert!(frame.resync_required.is_none());
            watched.extend(frame.records);
        }
        assert_eq!(watched, all);
        process.close().await?;

        // Returned Positions Continue to Resolve After Restart.
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;

        let replay = v1::evidence::records_client::RecordsClient::connect(process.url.clone())
            .await?
            .list(v1::evidence::RecordsListRequest {
                selection: Some(selection),
                at: first.resolved_at.clone(),
                page: Some(harmos_protocol::v1::PageRequest { page_size: 3, ..Default::default() }),
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
