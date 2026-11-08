/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

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
async fn captured_sample_scope() {
    use dokime_domain::model::{
        execution::RecordScope,
        identity::{SessionId, TestCycleId, TestExecutionId},
    };
    use dokime_runtime::domains::evidence::{ScopeSelection, TelemetrySelection};
    use dokime_runtime::testing::telemetry_admission::PreparedSample;

    let root = tempfile::tempdir().unwrap();
    let settings = dokime_runtime::storage::sqlite::Settings {
        telemetry_interval: std::time::Duration::from_secs(60),
        ..dokime_runtime::config::local_storage_defaults()
    };
    let backend = Arc::new(dokime_runtime::storage::Backend::primary_with(root.path().to_owned(), settings).await.unwrap());
    let local = State::new(backend.local_state());
    let state = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
        dokime_runtime::config::storage_state_defaults(),
    ));
    let storage = Manager::new(StorageBackends { primary: backend, state }, dokime_runtime::config::storage_defaults()).unwrap();
    let runtime = Runtime::<TestApp>::launch((), StorageConfig { storage: Some(storage.clone()) }).await.unwrap();
    let contract = dokime_runtime::telemetry::TelemetryContract::declared("capture.reading", "Reading").typed::<CaptureReading>(1);
    let codec = Arc::new(telemetry::codec(&contract, &Default::default()).unwrap());
    local.register_telemetry_contract(contract.clone(), codec.clone()).await.unwrap();
    storage.write_schema(codec.schema().clone(), codec.graph().clone()).await.unwrap();
    storage.flush().await.unwrap();
    let a = RecordScope {
        test_execution_id: Some(TestExecutionId::from_seed("capture-a")),
        test_cycle_id: Some(TestCycleId::from_seed("cycle-a")),
        test_case_id: Some(dokime_domain::model::identity::CaseId::from_seed("case-a")),
        template_id: Some(dokime_domain::model::identity::TemplateId::from_seed("template-a")),
        suite_id: Some(dokime_domain::model::identity::SuiteId::from_seed("suite-a")),
        ..Default::default()
    };
    let b = RecordScope {
        test_execution_id: Some(TestExecutionId::from_seed("capture-b")),
        test_cycle_id: Some(TestCycleId::from_seed("cycle-b")),
        test_case_id: Some(dokime_domain::model::identity::CaseId::from_seed("case-b")),
        template_id: Some(dokime_domain::model::identity::TemplateId::from_seed("template-b")),
        suite_id: Some(dokime_domain::model::identity::SuiteId::from_seed("suite-b")),
        ..Default::default()
    };
    let mut references = Vec::new();
    for index in 1..=4 {
        let mut sample = dokime_runtime::telemetry::TelemetrySample::new(
            dokime_runtime::model::identity::new_id(),
            contract.stream.clone(),
            index,
            dokime_domain::model::time::Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(index as i64)),
            "observer",
        )
        .typed(&CaptureReading { value: index as f64 });
        sample.schema_version = contract.schema_version;
        sample.values.insert("value".into(), dokime_domain::model::values::Value::Number(index as f64));
        if index == 3 {
            sample.scope = Some(b.clone());
        }
        let sample = Arc::new(sample);
        references.push(PreparedSample {
            sample: sample.clone(),
            lease: None,
            contract: 0,
            ordinal: index,
        });
        for scope in if index == 1 {
            vec![a.clone()]
        } else if index == 2 {
            vec![a.clone(), b.clone()]
        } else {
            Vec::new()
        } {
            references.push(PreparedSample {
                sample: sample.clone(),
                lease: Some(scope),
                contract: 0,
                ordinal: index,
            });
        }
    }
    let before = storage.cursor_accepted();
    let rows = stage_rows(&SessionId::from_seed("capture-session"), &references, std::slice::from_ref(&codec)).unwrap();
    let committed = Admission {
        storage: storage.clone(),
        local: local.clone(),
        rows,
    }
    .commit(false)
    .await
    .unwrap();
    assert!(committed.accepted.iter().all(|accepted| *accepted));
    let at = storage.cursor_accepted();
    assert_eq!(at.sequence - before.sequence, 4, "capture owners share one stored payload");
    assert_eq!(storage.cursor_saved(), before);
    for saved in [false, true] {
        if saved {
            storage.flush().await.unwrap();
        }
        for (scope, expected) in [
            (
                ScopeSelection {
                    test_execution_id: a.test_execution_id.clone(),
                    ..Default::default()
                },
                vec![1, 2],
            ),
            (
                ScopeSelection {
                    test_cycle_id: a.test_cycle_id.clone(),
                    ..Default::default()
                },
                vec![1, 2],
            ),
            (
                ScopeSelection {
                    test_execution_id: b.test_execution_id.clone(),
                    ..Default::default()
                },
                vec![2, 3],
            ),
            (
                ScopeSelection {
                    test_cycle_id: b.test_cycle_id.clone(),
                    ..Default::default()
                },
                vec![2, 3],
            ),
            (
                ScopeSelection {
                    test_execution_id: Some(TestExecutionId::from_seed("unrelated")),
                    ..Default::default()
                },
                vec![],
            ),
        ] {
            let selection = TelemetrySelection { scope, ..Default::default() };
            let page = collect_page(&storage, &at, &selection, &Default::default(), &local).await.unwrap();
            let values = page.samples.iter().map(|sample| sample.values["value"].clone()).collect::<Vec<_>>();
            assert_eq!(values, expected.iter().map(|value| dokime_domain::model::values::Value::Number(*value as f64)).collect::<Vec<_>>());
            for sample in page.samples.iter() {
                if sample.values["value"] == dokime_domain::model::values::Value::Number(3.0) {
                    assert_eq!(sample.scope.as_ref(), Some(&b));
                } else {
                    assert!(sample.scope.is_none());
                }
            }
        }
        let selected = ScopeSelection {
            test_execution_id: a.test_execution_id.clone(),
            ..Default::default()
        };
        let numeric = local
            .telemetry_numeric(
                at.clone(),
                StreamSelection {
                    schema: codec.schema().clone(),
                    scope: Some(Identifier::new(a.test_execution_id.as_ref().unwrap().as_str()).unwrap()),
                    item: None,
                    stream: None,
                    source_start: None,
                    source_end: None,
                    after: 0,
                },
                harmos::storage::ScanPage {
                    after: 0,
                    rows: NonZeroUsize::new(8).unwrap(),
                    bytes: NonZeroUsize::MAX,
                },
                vec!["value".into()],
                selected.clone(),
            )
            .await
            .unwrap();
        assert_eq!(numeric.iter().map(|(_, row)| row[0]).collect::<Vec<_>>(), [Some(1.0), Some(2.0)]);
        let selection = TelemetrySelection { scope: selected, ..Default::default() };
        let latest =
            local.latest_telemetry_time(at.clone(), codec.schema().clone(), Vec::new(), selection.clone(), (*codec).clone()).await.unwrap();
        assert_eq!(latest, Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(2)));
        let (_, prior) = local
            .telemetry_predecessor_page(
                at.clone(),
                codec.schema().clone(),
                selection,
                Vec::new(),
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(3),
                String::new(),
            )
            .await
            .unwrap();
        assert_eq!(prior.iter().map(|(_, entry)| entry.ordinal).collect::<Vec<_>>(), [2]);
    }
    let foreign = b.test_execution_id.as_ref().unwrap().to_string();
    let original_contract = local
        .read(move |connection| {
            connection.query_row("SELECT contract FROM streams WHERE execution=?1", [foreign], |row| row.get::<_, String>(0))
        })
        .await
        .unwrap();
    let foreign = b.test_execution_id.as_ref().unwrap().to_string();
    local.call(move |connection| connection.execute("UPDATE streams SET contract='invalid' WHERE execution=?1", [foreign])).await.unwrap();
    for scope in [
        ScopeSelection {
            test_execution_id: a.test_execution_id.clone(),
            ..Default::default()
        },
        ScopeSelection {
            test_cycle_id: a.test_cycle_id.clone(),
            ..Default::default()
        },
        ScopeSelection {
            test_case_id: a.test_case_id.clone(),
            ..Default::default()
        },
        ScopeSelection {
            template_id: a.template_id.clone(),
            ..Default::default()
        },
        ScopeSelection {
            suite_id: a.suite_id.clone(),
            ..Default::default()
        },
    ] {
        let selection = TelemetrySelection { scope, ..Default::default() };
        let page = collect_page(&storage, &at, &selection, &Default::default(), &local).await.unwrap();
        assert_eq!(
            page.samples.iter().map(|sample| sample.values["value"].clone()).collect::<Vec<_>>(),
            [dokime_domain::model::values::Value::Number(1.0), dokime_domain::model::values::Value::Number(2.0)]
        );
        assert_eq!(
            local.latest_telemetry_time(at.clone(), codec.schema().clone(), Vec::new(), selection.clone(), (*codec).clone()).await.unwrap(),
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(2))
        );
        let (_, prior) = local
            .telemetry_predecessor_page(
                at.clone(),
                codec.schema().clone(),
                selection,
                Vec::new(),
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(3),
                String::new(),
            )
            .await
            .unwrap();
        assert_eq!(prior.iter().map(|(_, entry)| entry.ordinal).collect::<Vec<_>>(), [2]);
    }
    let selection = TelemetrySelection {
        scope: ScopeSelection {
            test_execution_id: b.test_execution_id.clone(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(collect_page(&storage, &at, &selection, &Default::default(), &local).await.is_err());
    let foreign = b.test_execution_id.as_ref().unwrap().to_string();
    local
        .call(move |connection| {
            connection.execute("UPDATE streams SET contract=?1 WHERE execution=?2", rusqlite::params![original_contract, foreign])
        })
        .await
        .unwrap();
    let selected = TelemetrySelection {
        scope: ScopeSelection {
            test_execution_id: a.test_execution_id.clone(),
            ..Default::default()
        },
        after_stream_cursors: std::collections::BTreeMap::from([(
            dokime_runtime::domains::telemetry::telemetry_stream_id(None, &contract.stream),
            1,
        )]),
        ..Default::default()
    };
    let page = collect_page(&storage, &at, &selected, &Default::default(), &local).await.unwrap();
    assert_eq!(page.samples.iter().map(|sample| sample.sequence).collect::<Vec<_>>(), [2]);
    let rows: i64 =
        local.call(|connection| connection.query_row("SELECT sum(rows) FROM stream_batches", [], |row| row.get(0))).await.unwrap();
    assert_eq!(rows, 4);
    let files = local.telemetry_files(a.test_execution_id.as_ref().unwrap().to_string(), at.clone()).await.unwrap();
    assert_eq!(files.iter().map(|file| file.rows()).sum::<u64>(), 2);
    let file = &files[0];
    let batches = arrow_ipc::reader::FileReader::try_new(std::fs::File::open(file.path()).unwrap(), None).unwrap();
    assert_eq!(batches.map(|batch| batch.unwrap().num_rows()).sum::<usize>(), 2);
    let summary = file.summary("value").unwrap();
    assert_eq!(summary, (2, 3.0));
    drop(files);
    runtime.shutdown().await.unwrap();
    storage.close().await.unwrap();
    drop(storage);
    drop(local);
    let backend = dokime_runtime::storage::Backend::primary(root.path().to_owned()).await.unwrap();
    use harmos::storage::StorageBackend;
    backend.open().await.unwrap();
    let rows = State::new(backend.local_state())
        .telemetry_read(
            at,
            StreamSelection {
                schema: codec.schema().clone(),
                scope: Some(Identifier::new(a.test_execution_id.as_ref().unwrap().as_str()).unwrap()),
                item: None,
                stream: None,
                source_start: None,
                source_end: None,
                after: 0,
            },
            harmos::storage::ScanPage {
                after: 0,
                rows: NonZeroUsize::new(8).unwrap(),
                bytes: NonZeroUsize::MAX,
            },
        )
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|(_, entry)| entry.sample.scope.is_none()));
    backend.close().await.unwrap();
}
