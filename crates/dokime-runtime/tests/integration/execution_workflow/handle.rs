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
use dokime_runtime::definition::ParameterSource;
use dokime_runtime::{Identifier, Metadata, RecordScope, Value};
use harmos::state::Transaction;
use tempfile::tempdir;
// Local Imports
use dokime_runtime::transactions::UpsertScopedParameter;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn identifier(value: impl Into<String>) -> Identifier {
    Identifier::new(value).unwrap()
}

/* === Tests === */
#[tokio::test]
async fn wait_until_shutdown() {
    let root = tempdir().unwrap();
    let mut runtime_owner = {
        let runtime_config =
            crate::service_support::budgets::runtime_config(root.path(), Metadata::new(identifier("runtime.wait"), "Runtime Wait", None));
        dokime_runtime::Runtime::start(runtime_config)
    }
    .await
    .unwrap();
    let runtime = runtime_owner.handle();
    let mut state = runtime.watch_state();

    let shutdown = tokio::spawn(async move { runtime_owner.shutdown().await });
    state.wait_until(|snapshot| snapshot.state().shutting_down).await.unwrap();
    drop(runtime);
    shutdown.await.unwrap().unwrap();
}

pub(crate) fn scoped_parameter_values() {
    let mut state = dokime_runtime::state::RuntimeState::new(dokime_runtime::model::identity::new_id());
    let update = UpsertScopedParameter {
        scope: RecordScope::default(),
        key: "fixture.result".into(),
        value: "passed".into(),
        source: ParameterSource::Runtime,
        metadata: None,
        group_id: None,
        unit: None,
    };
    update.check(&state).unwrap();
    update.apply(&mut state);
    let before = state.run_parameters.clone();
    update.apply(&mut state);
    assert_eq!(state.run_parameters, before);
    assert!(state.run_parameters.contains_key("fixture.result"));
    let parameter = state.run_parameters.get("fixture.result").unwrap();
    assert_eq!(parameter.source, ParameterSource::Runtime);
    assert_eq!(parameter.value, Value::from("passed"));

    let traceability = UpsertScopedParameter {
        scope: RecordScope::default(),
        key: "fixture.serial_number".into(),
        value: "SN-123".into(),
        source: ParameterSource::Traceability,
        metadata: None,
        group_id: Some(identifier("traceability.runtime")),
        unit: None,
    };
    traceability.check(&state).unwrap();
    traceability.apply(&mut state);

    let parameter = state.run_parameters.get("fixture.serial_number").unwrap();
    assert_eq!(parameter.source, ParameterSource::Traceability);
    assert_eq!(parameter.group_id.as_ref().map(Identifier::as_str), Some("traceability.runtime"));
    assert_eq!(parameter.value, Value::from("SN-123"));
}
