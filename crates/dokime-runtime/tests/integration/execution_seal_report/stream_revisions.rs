/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Row Changes Through Live Plugin Reload And Backend Restart

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use async_trait::async_trait;

// Local Imports
use crate::service_support::{
    prelude::*,
    reports,
    suites::{materialize_cases, start_cases},
};
use dokime_runtime::{execution::Runnable, runtime::EmbeddedProvider};
use harmos::{schema::Described, sidecar::StreamDeclaration};

/* === Constants === */

/* === Macros === */

macro_rules! fixture {
    ($module:ident, {$($field:ident: $kind:ty),*}, $row:expr) => {
        mod $module {
            #[dokime_runtime::stream(id = "revision.reading", window = 32, label = "Reading")]
            pub(super) struct Row { $(pub(super) $field: $kind),* }
            pub(super) struct Checks;
            #[dokime_runtime::catalog]
            enum Templates { Measure(Measure) }
            #[dokime_runtime::catalog]
            enum Streams { Reading(Row) }
            #[dokime_runtime::suite(id = "revision.checks", label = "Checks", templates = Templates)]
            impl Checks {}
            pub(super) struct Measure;
            #[dokime_runtime::template(id = "revision.checks.measure", label = "Measure", streams = Streams)]
            impl Measure {
                async fn initialize(_: &mut dokime_runtime::execution::InitializationContext<'_>) -> dokime_runtime::Result<Self> { Ok(Self) }
                #[dokime_runtime::step(label = "Measure")]
                async fn measure(&mut self, ctx: &mut dokime_runtime::execution::StepContext<'_>) -> dokime_runtime::Result<()> {
                    ctx.publish($row).await;
                    Ok(())
                }
            }
        }
    };
}

fixture!(initial, { value: f64 }, Row { value: 7.0 });
fixture!(expanded, { added: u64, value: f64 }, Row { added: 29, value: 13.0 });
fixture!(array, { value: Vec<f64> }, Row { value: vec![3.0, 5.0] });
fixture!(incompatible, { value: String }, Row { value: "changed".into() });

/* === Type Aliases === */

/* === Structs === */

struct Provider(Variant);

/* === Enums === */

#[derive(Clone, Copy)]
enum Variant {
    Initial,
    Expanded,
    Incompatible,
    Array,
}

/* === Traits === */

/* === Implementations === */

impl Provider {
    fn suite(&self) -> dokime_runtime::domains::TestSuiteDefinition {
        match self.0 {
            Variant::Initial => initial::Checks::definition(),
            Variant::Expanded => expanded::Checks::definition(),
            Variant::Incompatible => incompatible::Checks::definition(),
            Variant::Array => array::Checks::definition(),
        }
    }
}

#[async_trait]
impl EmbeddedProvider for Provider {
    fn catalog(&self) -> dokime_runtime::InstalledPlugin {
        dokime_runtime::InstalledPlugin {
            source: "revision_plugin".into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime_runtime::definition::SuiteSet::new([self.suite()]),
            ..Default::default()
        }
    }
    fn streams(&self) -> Vec<StreamDeclaration> {
        vec![match self.0 {
            Variant::Initial => declaration::<initial::Row>(),
            Variant::Expanded => declaration::<expanded::Row>(),
            Variant::Incompatible => declaration::<incompatible::Row>(),
            Variant::Array => declaration::<array::Row>(),
        }]
    }
    async fn start(&self) -> dokime_runtime::runtime::Result<Vec<Arc<dyn Runnable>>> {
        let workflow: Arc<dyn Runnable> = match self.0 {
            Variant::Initial => {
                Arc::new(dokime_runtime::execution::Workflow::<initial::Measure>::new("revision.checks", "revision_plugin"))
            }
            Variant::Expanded => {
                Arc::new(dokime_runtime::execution::Workflow::<expanded::Measure>::new("revision.checks", "revision_plugin"))
            }
            Variant::Incompatible => {
                Arc::new(dokime_runtime::execution::Workflow::<incompatible::Measure>::new("revision.checks", "revision_plugin"))
            }
            Variant::Array => Arc::new(dokime_runtime::execution::Workflow::<array::Measure>::new("revision.checks", "revision_plugin")),
        };
        Ok(vec![workflow])
    }
    async fn stop(&self) -> dokime_runtime::runtime::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn declaration<R: harmos::StreamSpec + harmos::RowMetadata + Described>() -> StreamDeclaration {
    StreamDeclaration {
        id: R::ID.into(),
        window: R::WINDOW,
        metadata: <R as harmos::StreamSpec>::metadata(),
        row_metadata: <R as harmos::RowMetadata>::row_metadata(),
        row: harmos::schema::graph::Schema::from_declaration(R::DEFINITION, &Default::default()).unwrap(),
    }
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn stream_revisions_reload_and_restart_reports() {
    for restart in [false, true] {
        let root = tempdir().unwrap();
        let mut owner = dokime_runtime::Runtime::start(config(root.path()).register(Arc::new(Provider(Variant::Initial)))).await.unwrap();
        let mut handle = owner.handle();
        let previous_execution = if restart {
            // Capture Evidence Before Restart
            let suite = Provider(Variant::Initial).suite();
            let cycle = start_cases(
                &handle,
                metadata("revision.before_restart", "Before Restart"),
                materialize_cases("revision.checks", &suite.suite),
            )
            .await;
            handle.update_cycle_policy(cycle.clone(), false, CommandContext::default()).await.unwrap();
            let execution = handle.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
            let receipt = handle.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
            assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
            handle.wait_execution_report(&execution).await.unwrap();
            crate::service_support::waits::wait_for_run_settled(&handle, &receipt.command_id).await;

            // Restart Before Creating The Mixed-Revision Cycle
            owner.shutdown().await.unwrap();
            drop(handle);
            drop(owner);
            owner = dokime_runtime::Runtime::start(config(root.path()).register(Arc::new(Provider(Variant::Initial)))).await.unwrap();
            handle = owner.handle();
            let recovered = handle.saved_cycle_header(&cycle).await.unwrap().unwrap();
            assert_eq!(recovered.cycle.phase, dokime_domain::model::execution::result::CyclePhase::Sealed);
            Some(execution)
        } else {
            None
        };
        let suite = Provider(Variant::Initial).suite();
        let cycle = start_cases(&handle, metadata("revision.cycle", "Revisions"), materialize_cases("revision.checks", &suite.suite)).await;
        handle.update_cycle_policy(cycle.clone(), false, CommandContext::default()).await.unwrap();
        let mut executions = Vec::new();
        for variant in [Variant::Initial, Variant::Expanded, Variant::Incompatible] {
            if !matches!(variant, Variant::Initial) {
                owner.register(Arc::new(Provider(variant))).await.unwrap();
                let suite = Provider(variant).suite();
                handle
                    .append_fixture_cases(cycle.clone(), materialize_cases("revision.checks", &suite.suite), CommandContext::default())
                    .await
                    .unwrap();
            }
            let execution = handle.watch_state().latest().state().cycles[&cycle].execution_ids.last().unwrap().clone();
            let receipt = handle.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
            assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
            handle.wait_execution_report(&execution).await.unwrap();
            crate::service_support::waits::wait_for_run_settled(&handle, &receipt.command_id).await;
            executions.push(execution.clone());
            let current = handle.watch_state().latest().state().telemetry_definitions[&identifier("revision.reading")].clone();
            let expected = match variant {
                Variant::Initial => <initial::Row as dokime_runtime::telemetry::TelemetryStream>::contract(),
                Variant::Expanded => <expanded::Row as dokime_runtime::telemetry::TelemetryStream>::contract(),
                Variant::Incompatible => <incompatible::Row as dokime_runtime::telemetry::TelemetryStream>::contract(),
                Variant::Array => <array::Row as dokime_runtime::telemetry::TelemetryStream>::contract(),
            };
            assert_eq!(current.storage_schema, expected.storage_schema);
            let inspection = handle.inspect_execution(&execution).await.unwrap();
            let steps = &inspection.execution.steps;
            assert_eq!(steps.first().unwrap().metadata.identifier.as_str(), "initialization");
            let template = inspection.execution.snapshot.template().unwrap();
            assert_eq!(steps[1].step_id, template.phases[0].step_id);
            drop(inspection);
            let rows = handle
                .selected_telemetry(TelemetrySelection {
                    streams: vec![identifier("revision.reading").into()],
                    scope: ScopeSelection {
                        test_execution_id: Some(execution.clone()),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .await
                .unwrap();
            assert_eq!(rows.len(), 1);
            let expected = match variant {
                Variant::Initial => Value::Number(7.0),
                Variant::Expanded => Value::Number(13.0),
                Variant::Incompatible => Value::Text("changed".into()),
                Variant::Array => Value::Array(vec![Value::Number(3.0), Value::Number(5.0)]),
            };
            assert_eq!(rows[0].values["value"], expected);
            assert_eq!(rows[0].values.get("added"), matches!(variant, Variant::Expanded).then_some(&Value::Unsigned(29)));
            drop(rows);
            for previous in &executions {
                let mut report = handle.open_execution_report(previous).await.unwrap();
                let bytes = reports::opened_bytes(&mut report).await;
                let report = reports::parse(std::str::from_utf8(&bytes).unwrap());
                let samples = report["snapshot"]["telemetry"]["revision.reading"]["samples"].as_array().unwrap();
                assert_eq!(samples.len(), 1);
                let index = executions.iter().position(|id| id == previous).unwrap();
                assert_eq!(samples[0]["value"], [serde_json::json!(7.0), serde_json::json!(13.0), serde_json::json!("changed")][index]);
            }
            if let Some(previous) = &previous_execution {
                let mut report = handle.open_execution_report(previous).await.unwrap();
                let bytes = reports::opened_bytes(&mut report).await;
                let report = reports::parse(std::str::from_utf8(&bytes).unwrap());
                let samples = report["snapshot"]["telemetry"]["revision.reading"]["samples"].as_array().unwrap();
                assert_eq!(samples.len(), 1);
                assert_eq!(samples[0]["value"], serde_json::json!(7.0));
            }
            let mut report = handle.open_cycle_report(&cycle).await.unwrap();
            let bytes = reports::opened_bytes(&mut report).await;
            let report = reports::parse(std::str::from_utf8(&bytes).unwrap());
            let samples = report["snapshot"]["telemetry"]["revision.reading"]["samples"].as_array().unwrap();
            assert_eq!(samples.len(), executions.len());
            for (sample, expected) in samples.iter().zip([serde_json::json!(7.0), serde_json::json!(13.0), serde_json::json!("changed")]) {
                assert_eq!(sample["value"], expected);
            }
            if matches!(variant, Variant::Incompatible) {
                assert!(
                    report["warnings"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|warning| warning.as_str().is_some_and(|warning| warning.starts_with("report.stream_column_type_changed:")))
                );
            }
        }
        let initial = <initial::Row as dokime_runtime::telemetry::TelemetryStream>::contract();
        let expanded = <expanded::Row as dokime_runtime::telemetry::TelemetryStream>::contract();
        let incompatible = <incompatible::Row as dokime_runtime::telemetry::TelemetryStream>::contract();
        let fields = vec!["added".to_owned()];
        let selection = dokime_runtime::runtime::query::selection::Selection::samples(
            &[initial.clone(), expanded, incompatible.clone()],
            "revision.reading",
            dokime_runtime::runtime::query::selection::SelectionRequest {
                versions: &[],
                fields: &fields,
                predicates: &[],
                between: None,
                operation: "revision.test",
            },
            handle.query_config(),
        )
        .unwrap();
        assert!(selection.schema().fields.iter().find(|field| field.id == "added").unwrap().nullable);
        let error = dokime_runtime::runtime::query::selection::Selection::samples(
            &[initial, incompatible],
            "revision.reading",
            dokime_runtime::runtime::query::selection::SelectionRequest {
                versions: &[],
                fields: &[],
                predicates: &[],
                between: None,
                operation: "revision.test",
            },
            handle.query_config(),
        )
        .err()
        .unwrap();
        assert_eq!(error.reason, dokime_domain::model::query::error::Reason::IncompatibleSchema);
        assert_eq!(error.field.as_deref(), Some("value"));
        owner.shutdown().await.unwrap();
    }
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn stream_revisions_restart_retains_execution_reports() {
    let root = tempdir().unwrap();
    let mut owner = dokime_runtime::Runtime::start(config(root.path()).register(Arc::new(Provider(Variant::Initial)))).await.unwrap();
    let mut handle = owner.handle();
    let mut executions = Vec::new();
    for (index, variant) in [Variant::Initial, Variant::Expanded].into_iter().enumerate() {
        if index != 0 {
            owner.shutdown().await.unwrap();
            drop(handle);
            drop(owner);
            owner = dokime_runtime::Runtime::start(config(root.path()).register(Arc::new(Provider(variant)))).await.unwrap();
            handle = owner.handle();
        }
        let suite = Provider(variant).suite();
        let cycle = start_cases(
            &handle,
            metadata(&format!("revision.restart.{index}"), "Revisions"),
            materialize_cases("revision.checks", &suite.suite),
        )
        .await;
        handle.update_cycle_policy(cycle.clone(), false, CommandContext::default()).await.unwrap();
        let execution = handle.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
        let receipt = handle.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
        assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
        handle.wait_execution_report(&execution).await.unwrap();
        crate::service_support::waits::wait_for_run_settled(&handle, &receipt.command_id).await;
        executions.push(execution);
        for (previous_index, previous) in executions.iter().enumerate() {
            let rows = handle
                .selected_telemetry(TelemetrySelection {
                    streams: vec![identifier("revision.reading").into()],
                    scope: ScopeSelection {
                        test_execution_id: Some(previous.clone()),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .await
                .unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].values["value"], Value::Number([7.0, 13.0][previous_index]));
            assert_eq!(rows[0].values.get("added"), (previous_index == 1).then_some(&Value::Unsigned(29)));
            drop(rows);
            let mut report = handle.open_execution_report(previous).await.unwrap();
            let bytes = reports::opened_bytes(&mut report).await;
            let report = reports::parse(std::str::from_utf8(&bytes).unwrap());
            let samples = report["snapshot"]["telemetry"]["revision.reading"]["samples"].as_array().unwrap();
            assert_eq!(samples.len(), 1);
            assert_eq!(samples[0]["value"], serde_json::json!([7.0, 13.0][previous_index]));
        }
    }
    owner.shutdown().await.unwrap();
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn stream_revisions_array_to_text_keeps_cycle_values() {
    let root = tempdir().unwrap();
    let mut owner = dokime_runtime::Runtime::start(config(root.path()).register(Arc::new(Provider(Variant::Array)))).await.unwrap();
    let handle = owner.handle();
    let suite = Provider(Variant::Array).suite();
    let cycle = start_cases(&handle, metadata("revision.array", "Revisions"), materialize_cases("revision.checks", &suite.suite)).await;
    handle.update_cycle_policy(cycle.clone(), false, CommandContext::default()).await.unwrap();
    for variant in [Variant::Array, Variant::Incompatible] {
        if matches!(variant, Variant::Incompatible) {
            owner.register(Arc::new(Provider(variant))).await.unwrap();
            let suite = Provider(variant).suite();
            handle
                .append_fixture_cases(cycle.clone(), materialize_cases("revision.checks", &suite.suite), CommandContext::default())
                .await
                .unwrap();
        }
        let execution = handle.watch_state().latest().state().cycles[&cycle].execution_ids.last().unwrap().clone();
        let receipt = handle.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
        assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
        handle.wait_execution_report(&execution).await.unwrap();
        crate::service_support::waits::wait_for_run_settled(&handle, &receipt.command_id).await;
    }
    let mut report = handle.open_cycle_report(&cycle).await.unwrap();
    let bytes = reports::opened_bytes(&mut report).await;
    let report = reports::parse(std::str::from_utf8(&bytes).unwrap());
    let samples = report["snapshot"]["telemetry"]["revision.reading"]["samples"].as_array().unwrap();
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0]["value"], serde_json::json!([3.0, 5.0]));
    assert_eq!(samples[1]["value"], serde_json::json!("changed"));
    assert!(
        report["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning.as_str().is_some_and(|warning| warning.starts_with("report.stream_column_type_changed:")))
    );
    owner.shutdown().await.unwrap();
}
