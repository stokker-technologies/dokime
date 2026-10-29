/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{collections::BTreeMap, time::Duration};

use dokime_runtime::definition::ParameterSource;
use dokime_runtime::{Identifier, Metadata, RecordScope, Value};
use tempfile::tempdir;

use dokime_runtime::domains::evidence::RecordSelection;
use dokime_runtime::domains::execution::authoring::{RuntimeResource, SidecarStepContext, StepContextError};

fn identifier(value: impl Into<String>) -> Identifier {
    Identifier::new(value).unwrap()
}

// resource store publishes lifecycle records and keeps handle runtime only.

pub(crate) async fn resource_store_handle() {
    let root = tempdir().unwrap();
    let mut runtime_owner = {
        let runtime_config = crate::service_support::budgets::runtime_config(
            root.path(),
            Metadata::new(identifier("runtime.authoring"), "Runtime Authoring", None),
        );
        dokime_runtime::Runtime::start(runtime_config)
    }
    .await
    .unwrap();
    let runtime = runtime_owner.handle();
    let mut ctx = SidecarStepContext::with_source(runtime.clone(), RecordScope::default(), "example.tracking");

    ctx.resources().set_async(RuntimeResource::new("tooling.session", "tooling", "Tooling Session", 42_i64)).await.unwrap();
    assert_eq!(ctx.resources().get::<i64>("tooling.session"), Some(&42));

    ctx.resources().mark_opened_async("tooling.session", BTreeMap::new()).await.unwrap();
    let error = ctx.resources().remove_async::<String>("tooling.session", BTreeMap::new()).await.unwrap_err();
    assert!(matches!(
        error,
        StepContextError::ResourceTypeMismatch {
            resource_id,
            expected_type: "alloc::string::String",
        } if resource_id == "tooling.session"
    ));
    assert!(ctx.resources().contains("tooling.session"));

    let removed = ctx.resources().remove_async::<i64>("tooling.session", BTreeMap::new()).await.unwrap();

    assert_eq!(removed, Some(42));
    assert!(!ctx.resources().contains("tooling.session"));
    let records = runtime.selected_records(RecordSelection::default()).await.unwrap();
    let record_types = records.iter().map(|record| record.record_type.as_str()).collect::<Vec<_>>();

    assert!(record_types.contains(&"resource.registered"));
    assert!(record_types.contains(&"resource.opened"));
    assert!(record_types.contains(&"resource.removed"));

    drop(ctx);
    runtime_owner.shutdown().await.unwrap();
}

pub(crate) async fn wait_completes_duration() {
    let root = tempdir().unwrap();
    let mut runtime_owner = {
        let runtime_config =
            crate::service_support::budgets::runtime_config(root.path(), Metadata::new(identifier("runtime.wait"), "Runtime Wait", None));
        dokime_runtime::Runtime::start(runtime_config)
    }
    .await
    .unwrap();
    let runtime = runtime_owner.handle();
    let ctx = SidecarStepContext::new(runtime, RecordScope::default());

    tokio::time::pause();
    {
        let waiting = ctx.wait_async(Duration::from_millis(10));
        tokio::pin!(waiting);
        assert!(futures_util::FutureExt::now_or_never(&mut waiting).is_none());
        tokio::time::advance(Duration::from_millis(9)).await;
        assert!(futures_util::FutureExt::now_or_never(&mut waiting).is_none());
        tokio::time::advance(Duration::from_millis(1)).await;
        waiting.await.unwrap();
    }
    tokio::time::resume();

    drop(ctx);
    runtime_owner.shutdown().await.unwrap();
}

pub(crate) async fn wait_until_shutdown() {
    let root = tempdir().unwrap();
    let mut runtime_owner = {
        let runtime_config =
            crate::service_support::budgets::runtime_config(root.path(), Metadata::new(identifier("runtime.wait"), "Runtime Wait", None));
        dokime_runtime::Runtime::start(runtime_config)
    }
    .await
    .unwrap();
    let runtime = runtime_owner.handle();
    let ctx = SidecarStepContext::new(runtime.clone(), RecordScope::default());

    let shutdown = tokio::spawn(async move { runtime_owner.shutdown().await });
    ctx.wait_until_stopped_async().await.unwrap();
    drop(ctx);
    drop(runtime);
    shutdown.await.unwrap().unwrap();
}

pub(crate) async fn file_helpers_scope() {
    let root = tempdir().unwrap();
    let mut runtime_owner = {
        let runtime_config =
            crate::service_support::budgets::runtime_config(root.path(), Metadata::new(identifier("runtime.files"), "Runtime Files", None));
        dokime_runtime::Runtime::start(runtime_config)
    }
    .await
    .unwrap();
    let runtime = runtime_owner.handle();
    let ctx = SidecarStepContext::new(runtime, RecordScope::default());

    let error = ctx.target_async("logs/run.log").await.unwrap_err();

    assert!(matches!(error, StepContextError::MissingExecutionScope { accessor: "execution files" }));

    drop(ctx);
    runtime_owner.shutdown().await.unwrap();
}

pub(crate) async fn param_helpers_values() {
    let root = tempdir().unwrap();
    let mut runtime_owner = {
        let runtime_config = crate::service_support::budgets::runtime_config(
            root.path(),
            Metadata::new(identifier("runtime.parameters"), "Runtime Parameters", None),
        );
        dokime_runtime::Runtime::start(runtime_config)
    }
    .await
    .unwrap();
    let runtime = runtime_owner.handle();
    let ctx = SidecarStepContext::new(runtime.clone(), RecordScope::default());

    assert!(ctx.set_runtime_parameter_async("fixture.result", "passed").await.unwrap());
    assert!(!ctx.set_runtime_parameter_async("fixture.result", "passed").await.unwrap());
    let snapshot = runtime.watch_state().latest();
    let state = snapshot.state();
    assert!(state.run_parameters.contains_key("fixture.result"));
    let parameter = state.run_parameters.get("fixture.result").unwrap();
    assert_eq!(parameter.source, ParameterSource::Runtime);
    assert_eq!(parameter.value, Value::from("passed"));

    assert!(ctx.set_traceability_parameter_async("fixture.serial_number", "SN-123").await.unwrap());

    let snapshot = runtime.watch_state().latest();
    let state = snapshot.state();
    let parameter = state.run_parameters.get("fixture.serial_number").unwrap();
    assert_eq!(parameter.source, ParameterSource::Traceability);
    assert_eq!(parameter.group_id.as_ref().map(Identifier::as_str), Some("traceability.runtime"));
    assert_eq!(parameter.value, Value::from("SN-123"));

    drop(ctx);
    runtime_owner.shutdown().await.unwrap();
}
