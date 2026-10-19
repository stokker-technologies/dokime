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
use crate::support::wire::WireMessage;
use prost::Message;
// Local Imports
use super::*;
use dokime_domain::model;
use dokime_runtime::Value;
use dokime_runtime::definition::{FileDefinition, ParameterExpr, ParameterSelection, SuiteSet};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn selection(suite: &dokime_runtime::definition::TestSuite) -> model::planning::wire::RegisteredSelection {
    model::planning::wire::RegisteredSelection {
        suite_id: suite.metadata.identifier.to_string(),
        template_id: suite.test_case_templates[0].test_case_template_id.to_string(),
        ..Default::default()
    }
}

fn integer(value: i64) -> model::values::wire::Value {
    model::values::wire::Value::from(&Value::Integer(value))
}

fn field(value: &model::values::wire::Value, key: &str) -> Value {
    let Value::Object(mut fields) = value.to_core().unwrap() else {
        panic!("planned parameters and file attributes must be objects");
    };
    fields.remove(key).expect("declared field is present")
}

fn plan(selections: Vec<model::planning::wire::RegisteredSelection>) -> model::planning::wire::SelectionPlan {
    model::planning::wire::SelectionPlan { selections, max_cases: 0, case_ids: vec![], cycle_id: String::new() }
}

fn create() -> contracts::execution::CreateCycle {
    contracts::execution::CreateCycle {
        metadata: Some(model::identity::wire::Metadata {
            identifier: "planning.cycle".into(),
            label: "Plan".into(),
            description: None,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn append(cycle_id: &str, plan: model::planning::wire::SelectionPlan) -> support::wire::SelectionAppend {
    support::wire::SelectionAppend { cycle_id: cycle_id.into(), plan: Some(plan), context: None }
}

async fn empty_cycle(execution: &WireClient) -> String {
    execution.unary::<contracts::execution::CreateCycle>(create()).await.unwrap().into_inner().cycle_id
}

async fn assert_no_cases(owner: &Runtime) {
    assert!(owner.handle().cycles().await.iter().all(|cycle| cycle.cases().next().is_none()));
}

fn error_code(status: &tonic::Status) -> String {
    let envelope = tonic_types::Status::decode(status.details()).unwrap();
    <model::execution::wire::Rejection as WireMessage>::from_wire(envelope.details[0].value.as_slice()).unwrap().code
}

#[tokio::test]
async fn ordered_selections_provider() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let mut first = support::fixture_suite(&["passed"]);
    first.tags = vec!["suite-tag".into()];
    let parameter = &mut first.test_case_templates[0].parameters[0];
    parameter.expansion = dokime_runtime::definition::ParameterExpansion::Cartesian;
    parameter.value_type = dokime_runtime::ValueType::Integer;
    parameter.expression = ParameterExpr::Literal { value: Value::Integer(0) };
    first.test_case_templates[0].files.push(FileDefinition {
        metadata: support::metadata("fixture.file", "Input"),
        default: Some("plugin:default".into()),
        required: false,
        media_types: vec!["application/octet-stream".into()],
        attributes: [("maximum".into(), Value::Integer(i64::MAX))].into(),
    });
    let mut second = support::fixture_suite(&["passed"]);
    second.metadata = support::metadata("fixture.second", "Second");
    second.test_case_templates[0].metadata = support::metadata("fixture.second.template", "Second template");
    let parameter = &mut second.test_case_templates[0].parameters[0];
    parameter.list = Some(dokime_runtime::definition::ParameterList {
        element_type: dokime_runtime::ValueType::Integer,
        min_items: None,
        max_items: None,
    });
    parameter.value_type = dokime_runtime::ValueType::Array;
    parameter.expression = ParameterExpr::Literal { value: Value::Array(vec![Value::Integer(0)]) };
    parameter.selection = Some(ParameterSelection { choices: vec![] });
    owner
        .handle()
        .set_suites(
            "provider:fixture",
            SuiteSet::new([support::fixture_registration(first.clone()), support::fixture_registration(second.clone())]),
        )
        .await
        .unwrap();
    let server = Server::start(owner.handle(), keyed()).await.unwrap();
    let planning = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut selected = selection(&first);
    selected.parameters.push(model::planning::wire::RegisteredSelectionParametersEntry {
        key: "fixture.result".into(),
        value: Some(model::planning::wire::SelectionValues { values: vec![integer(i64::MIN), integer(i64::MAX), integer(i64::MIN)] }),
    });
    let mut multiple = selection(&second);
    multiple.parameters.push(model::planning::wire::RegisteredSelectionParametersEntry {
        key: "fixture.result".into(),
        value: Some(model::planning::wire::SelectionValues {
            values: vec![model::values::wire::Value::from(&Value::Array(vec![
                Value::Integer(i64::MIN),
                Value::Integer(i64::MAX),
                Value::Integer(i64::MIN),
            ]))],
        }),
    });
    let input = plan(vec![selected.clone(), multiple, selected]);
    let preview = planning
        .unary::<contracts::planning::PreviewSelection>(authenticated(contracts::planning::PreviewSelection { plan: Some(input.clone()) }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(preview.cases.len(), 7);
    assert_eq!(preview.cases.iter().map(|case| case.selection_index).collect::<Vec<_>>(), [0, 0, 0, 1, 2, 2, 2]);
    assert_eq!(preview.cases.iter().map(|case| &case.case_id).collect::<std::collections::BTreeSet<_>>().len(), 7);
    assert_eq!(field(preview.cases[0].parameters.as_ref().unwrap(), "fixture.result"), Value::Integer(i64::MIN));
    assert_eq!(field(preview.cases[1].parameters.as_ref().unwrap(), "fixture.result"), Value::Integer(i64::MAX));
    assert!(matches!(field(preview.cases[3].parameters.as_ref().unwrap(), "fixture.result"), Value::Array(_)));
    let repeated = planning
        .unary::<contracts::planning::PreviewSelection>(authenticated(contracts::planning::PreviewSelection { plan: Some(input.clone()) }))
        .await
        .unwrap()
        .into_inner();
    for (actual, expected) in repeated.cases.iter().zip(&preview.cases) {
        assert_ne!(actual.case_id, expected.case_id);
        let mut fields = actual.clone();
        fields.case_id.clone_from(&expected.case_id);
        assert_eq!(&fields, expected);
    }
    assert_eq!(repeated.cases.len(), preview.cases.len());
    let catalog = WireClient::connect(server.local_endpoint()).await.unwrap();
    let definition = catalog
        .unary::<contracts::catalog::GetSuite>(authenticated(contracts::catalog::GetSuite {
            suite_id: first.metadata.identifier.to_string(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(definition.tags, ["fixture", "suite-tag"]);
    assert_eq!(definition.templates[0].steps[0].step_id, first.test_case_templates[0].phases[0].step_id.to_string());
    let file = &definition.templates[0].files[0];
    assert_eq!(file.default_value.as_deref(), Some("plugin:default"));
    assert_eq!(field(file.attributes.as_ref().unwrap(), "maximum"), Value::Integer(i64::MAX));
    let execution = WireClient::connect(server.local_endpoint()).await.unwrap();
    let cycle_id = support::planned_cycle(&execution, authenticated(create()), input).await.unwrap();
    let snapshot = owner.handle().watch_state().latest();
    let id = dokime_runtime::TestCycleId::from_uuid(cycle_id).unwrap();
    let cycle = snapshot.state().cycle_view(&id).unwrap();
    assert_eq!(cycle.lifecycle, dokime_domain::model::execution::result::CycleStatus::Ready);
    assert_eq!(cycle.test_executions.len(), cycle.cases().count());
    for (execution, case) in cycle.test_executions.iter().zip(cycle.cases()) {
        assert_eq!(execution.test_case_id, case.test_case_id);
        assert_eq!(execution.status, dokime_domain::model::execution::result::ExecutionStatus::Skipped);
        assert!(execution.started_at.is_none());
    }
    for (case, expected) in cycle.cases().zip(&preview.cases) {
        assert_ne!(case.test_case_id.as_str(), expected.case_id);
        let suite = snapshot.state().suites.suite_for_case(case).unwrap();
        assert_eq!(suite.suite_id, expected.suite_id);
        assert_eq!(case.suite.as_ref(), Some(&suite.suite.metadata));
        assert_eq!(case.test_case_template_id.as_str(), expected.template_id);
        let target = snapshot.state().suites.provider_target_for_case(case).unwrap();
        assert_eq!(target.source, "provider:fixture");
        assert_eq!(target.binding.workflow(), "execute_cycle");
    }
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn planning_refuses_inputs() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let mut first = support::fixture_suite(&["passed"]);
    first.test_case_templates[0].files.push(FileDefinition {
        metadata: support::metadata("fixture.file", "Required"),
        default: Some("plugin:default".into()),
        required: true,
        media_types: vec![],
        attributes: Default::default(),
    });
    let mut second = support::fixture_suite(&["passed"]);
    second.metadata = support::metadata("fixture.second", "Second");
    second.test_case_templates[0].metadata = support::metadata("fixture.second.template", "Second template");
    let mut other = support::fixture_registration(second.clone());
    other.workflow = Some("another_workflow".into());
    owner.handle().set_suites("provider:fixture", SuiteSet::new([support::fixture_registration(first.clone()), other])).await.unwrap();
    let server = Server::start(owner.handle(), Config::default()).await.unwrap();
    let planning = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mixed = plan(vec![selection(&first), selection(&second)]);
    let failure = planning
        .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(mixed.clone()) })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "mixed_provider_targets");
    owner.handle().set_suites("provider:fixture", SuiteSet::new([support::fixture_registration(first.clone())])).await.unwrap();
    owner.handle().set_suites("provider:other", SuiteSet::new([support::fixture_registration(second.clone())])).await.unwrap();
    let failure = planning
        .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(mixed.clone()) })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "mixed_provider_targets");
    let execution = WireClient::connect(server.local_endpoint()).await.unwrap();
    let target = empty_cycle(&execution).await;
    let mut invalid = selection(&first);
    invalid.files.push(model::planning::wire::RegisteredSelectionFilesEntry {
        key: "fixture.file".into(),
        value: Some(model::planning::wire::SelectedFile { kind: Some(model::planning::wire::SelectedFileKind::Clear) }),
    });
    let mut unknown = selection(&first);
    unknown.template_id = first.test_case_templates[0].metadata.identifier.to_string();
    let mut cartesian = selection(&first);
    cartesian.parameters.push(model::planning::wire::RegisteredSelectionParametersEntry {
        key: "fixture.result".into(),
        value: Some(model::planning::wire::SelectionValues {
            values: vec![
                model::values::wire::Value::from(&Value::Text("a".into())),
                model::values::wire::Value::from(&Value::Text("b".into())),
            ],
        }),
    });
    for (label, expected, input) in [
        ("no selections", Code::InvalidArgument, plan(vec![])),
        ("a cleared file", Code::InvalidArgument, plan(vec![invalid])),
        ("an unknown template", Code::NotFound, plan(vec![unknown])),
        (
            "a case ceiling above the limit",
            Code::ResourceExhausted,
            model::planning::wire::SelectionPlan { max_cases: 1001, ..plan(vec![selection(&first)]) },
        ),
        (
            "an expansion past its own ceiling",
            Code::InvalidArgument,
            model::planning::wire::SelectionPlan { max_cases: 3, ..plan(vec![cartesian.clone(), cartesian]) },
        ),
        ("more selections than the request admits", Code::ResourceExhausted, plan(vec![selection(&first); 257])),
    ] {
        assert_eq!(
            planning
                .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(input.clone()) })
                .await
                .unwrap_err()
                .code(),
            expected,
            "previewing {label}"
        );
        assert_eq!(execution.append_selection(append(&target, input)).await.unwrap_err().code(), expected, "appending {label}");
    }
    assert_no_cases(&owner).await;
    server.shutdown().await.unwrap();
    // Authentication Is A Construction Setting Of The Runtime Service.
    owner.shutdown().await.unwrap();
    let mut owner = runtime(root.path()).await;
    owner.handle().set_suites("provider:fixture", SuiteSet::new([support::fixture_registration(first.clone())])).await.unwrap();
    let remote = Server::start(owner.handle(), keyed()).await.unwrap();
    let planning = WireClient::connect(remote.local_endpoint()).await.unwrap();
    planning
        .unary::<contracts::planning::PreviewSelection>(authenticated(contracts::planning::PreviewSelection {
            plan: Some(plan(vec![selection(&first)])),
        }))
        .await
        .unwrap();
    assert_eq!(
        planning
            .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection {
                plan: Some(plan(vec![selection(&first)]))
            })
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let execution = WireClient::connect(remote.local_endpoint()).await.unwrap();
    execution.unary::<contracts::execution::CreateCycle>(authenticated(create())).await.unwrap();
    remote.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn template_conflicts_disambiguate() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let first = support::fixture_suite(&["passed"]);
    let mut second = support::fixture_suite(&["passed"]);
    second.metadata = support::metadata("fixture.second", "Second");
    // Unique UUIDs are insufficient: native domains resolve this identifier.
    owner
        .handle()
        .set_suites(
            "provider:fixture",
            SuiteSet::new([support::fixture_registration(first.clone()), support::fixture_registration(second.clone())]),
        )
        .await
        .unwrap();
    let server = Server::start(owner.handle(), Config::default()).await.unwrap();
    let planning = WireClient::connect(server.local_endpoint()).await.unwrap();
    let failure = planning
        .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(plan(vec![selection(&first)])) })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "ambiguous_registered_identity");
    second.test_case_templates[0].metadata = support::metadata("fixture.second.template", "Second");
    second.metadata = first.metadata.clone();
    let mut registration = support::fixture_registration(second);
    registration.suite_id = "fixture.second".into();
    owner
        .handle()
        .set_suites("provider:fixture", SuiteSet::new([support::fixture_registration(first.clone()), registration]))
        .await
        .unwrap();
    let execution = WireClient::connect(server.local_endpoint()).await.unwrap();
    let cycle_id = support::planned_cycle(&execution, Request::new(create()), plan(vec![selection(&first)])).await.unwrap();
    let cycles = owner.handle().cycles().await;
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].test_cycle_id.as_str(), cycle_id);
    assert!(cycles[0].cases().all(|case| case.suite_id == first.metadata.identifier.as_str()));
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn effective_native_materialization() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let mut suite = support::fixture_suite(&["passed"]);
    suite.test_case_templates[0].parameters[0].expansion = dokime_runtime::definition::ParameterExpansion::Cartesian;
    suite.test_case_templates[0].parameters[0].value_type = dokime_runtime::ValueType::Integer;
    let server = Server::start(owner.handle(), Config::default()).await.unwrap();
    let planning = WireClient::connect(server.local_endpoint()).await.unwrap();
    let execution = WireClient::connect(server.local_endpoint()).await.unwrap();
    let target = empty_cycle(&execution).await;
    for expression in [
        ParameterExpr::Range { start: 0, stop: i64::MAX, step: 1 },
        ParameterExpr::Range { start: i64::MIN, stop: i64::MIN, step: -1 },
        ParameterExpr::Arange { start: 1e20, stop: 2e20, step: 1.0 },
        ParameterExpr::Array { values: vec![ParameterExpr::Linspace { start: 0.0, stop: 1.0, count: usize::MAX }] },
    ] {
        suite.test_case_templates[0].parameters[0].expression = expression;
        support::register_fixture_execution(&owner.handle(), suite.clone()).await;
        let input = plan(vec![selection(&suite)]);
        let failure = planning
            .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(input.clone()) })
            .await
            .unwrap_err();
        assert_eq!(error_code(&failure), "parameter_expansion_limit");
        let failure = execution.append_selection(append(&target, input)).await.unwrap_err();
        assert_eq!(error_code(&failure), "parameter_expansion_limit");
        // The registered expression is irrelevant when explicitly overridden.
        let mut selected = selection(&suite);
        selected.parameters.push(model::planning::wire::RegisteredSelectionParametersEntry {
            key: "fixture.result".into(),
            value: Some(model::planning::wire::SelectionValues { values: vec![integer(7)] }),
        });
        let preview = planning
            .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(plan(vec![selected])) })
            .await
            .unwrap()
            .into_inner();
        assert_eq!(preview.cases.len(), 1);
        assert_eq!(field(preview.cases[0].parameters.as_ref().unwrap(), "fixture.result"), Value::Integer(7));
    }
    suite.test_case_templates[0].parameters[0].expression = ParameterExpr::Range { start: 0, stop: 100, step: 1 };
    let mut other = suite.test_case_templates[0].parameters[0].clone();
    other.parameter_id = dokime_runtime::model::identity::new_id::<dokime_runtime::ParameterId>();
    other.metadata = support::metadata("fixture.other", "Other dimension");
    suite.test_case_templates[0].parameters.push(other);
    support::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let input = plan(vec![selection(&suite)]);
    let failure = planning
        .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(input.clone()) })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "invalid_selection");
    assert!(failure.message().contains("10201"));
    assert!(execution.append_selection(append(&target, input)).await.is_err());
    assert_no_cases(&owner).await;
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn payload_budget_allocation() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let mut suite = support::fixture_suite(&["passed"]);
    let template = &mut suite.test_case_templates[0];
    template.parameters[0].expansion = dokime_runtime::definition::ParameterExpansion::Cartesian;
    template.parameters[0].value_type = dokime_runtime::ValueType::Integer;
    template.parameters[0].expression = ParameterExpr::Range { start: 0, stop: 999, step: 1 };
    for index in 0..32 {
        template.files.push(FileDefinition {
            metadata: support::metadata(&format!("fixture.file{index}"), "File"),
            default: Some("x".repeat(4096)),
            required: false,
            media_types: vec![],
            attributes: Default::default(),
        });
    }
    support::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let server = Server::start(owner.handle(), Config::default()).await.unwrap();
    let planning = WireClient::connect(server.local_endpoint()).await.unwrap();
    let execution = WireClient::connect(server.local_endpoint()).await.unwrap();
    let target = empty_cycle(&execution).await;
    let input = plan(vec![selection(&suite)]);
    let failure = planning
        .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(input.clone()) })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "planning_payload_limit");
    assert_eq!(error_code(&execution.append_selection(append(&target, input)).await.unwrap_err()), "planning_payload_limit");
    // A smaller explicit selection is valid with the same large default files.
    let mut selected = selection(&suite);
    selected.parameters.push(model::planning::wire::RegisteredSelectionParametersEntry {
        key: "fixture.result".into(),
        value: Some(model::planning::wire::SelectionValues { values: vec![integer(0)] }),
    });
    assert_eq!(
        planning
            .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(plan(vec![selected])) })
            .await
            .unwrap()
            .into_inner()
            .cases
            .len(),
        1
    );
    suite.test_case_templates[0].files.clear();
    suite.test_case_templates[0].metadata.label = "label".repeat(1000);
    suite.metadata.description = Some("description".repeat(1000));
    support::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let failure = planning
        .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(plan(vec![selection(&suite)])) })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "planning_payload_limit");
    suite.test_case_templates[0].metadata.label = "Template".into();
    suite.metadata.description = None;
    let template = &mut suite.test_case_templates[0];
    template.parameters[0].value_type = dokime_runtime::ValueType::Array;
    template.parameters[0].expression = ParameterExpr::Literal { value: Value::Array(vec![Value::Text("x".repeat(1024)); 32]) };
    let mut other = template.parameters[0].clone();
    other.parameter_id = dokime_runtime::model::identity::new_id::<dokime_runtime::ParameterId>();
    other.metadata = support::metadata("fixture.other", "Other");
    other.expression = ParameterExpr::Array { values: vec![ParameterExpr::Reference { parameter: "fixture.result".into() }; 1000] };
    template.parameters.push(other);
    support::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let input = plan(vec![selection(&suite)]);
    assert_eq!(
        error_code(
            &planning
                .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(input.clone()) })
                .await
                .unwrap_err()
        ),
        "planning_payload_limit"
    );
    assert_eq!(error_code(&execution.append_selection(append(&target, input)).await.unwrap_err()), "planning_payload_limit");
    assert_no_cases(&owner).await;
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */

#[tokio::test]
async fn materialization_rejects_nonfinite_parameters_before_registration() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let mut suite = support::fixture_suite(&["passed"]);
    let handle = owner.handle();
    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    for value_type in [
        dokime_runtime::ValueType::Number,
        dokime_runtime::ValueType::Boolean,
        dokime_runtime::ValueType::Text,
        dokime_runtime::ValueType::Array,
        dokime_runtime::ValueType::Object,
    ] {
        suite.test_case_templates[0].parameters[0].value_type = value_type.clone();
        handle.set_suites("provider:fixture", SuiteSet::new([support::fixture_registration(suite.clone())])).await.unwrap();
        for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let value = match value_type {
                dokime_runtime::ValueType::Array => Value::Array(vec![Value::Object([("nested".into(), Value::Number(number))].into())]),
                dokime_runtime::ValueType::Object => Value::Object([("nested".into(), Value::Array(vec![Value::Number(number)]))].into()),
                _ => Value::Number(number),
            };
            let mut selected = selection(&suite);
            selected.parameters.push(model::planning::wire::RegisteredSelectionParametersEntry {
                key: "fixture.result".into(),
                value: Some(model::planning::wire::SelectionValues { values: vec![model::values::wire::Value::from(&value)] }),
            });
            let request = contracts::planning::MaterializeTemplate { selection: Some(selected) };
            let refused = client.unary::<contracts::planning::MaterializeTemplate>(request).await.unwrap_err();
            assert_eq!(refused.code(), Code::InvalidArgument, "type {value_type:?}, number {number}");
            assert!(refused.message().contains("finite"));
            let snapshot = handle.watch_state().latest();
            assert_eq!(snapshot.state().suites.cases_for_suite(suite.metadata.identifier.as_str()).count(), 0);
            assert!(snapshot.state().cycles.is_empty());
            assert!(snapshot.state().executions.is_empty());
            assert!(snapshot.state().run_requests.is_empty());
        }
    }

    // Finite Nested Values Still Materialize Without Starting An Execution.
    let value = Value::Object([("nested".into(), Value::Array(vec![Value::Number(1.25)]))].into());
    let mut selected = selection(&suite);
    selected.parameters.push(model::planning::wire::RegisteredSelectionParametersEntry {
        key: "fixture.result".into(),
        value: Some(model::planning::wire::SelectionValues { values: vec![model::values::wire::Value::from(&value)] }),
    });
    let reply = client
        .unary::<contracts::planning::MaterializeTemplate>(contracts::planning::MaterializeTemplate { selection: Some(selected) })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(reply.cases.len(), 1);
    assert_eq!(field(reply.cases[0].parameters.as_ref().unwrap(), "fixture.result"), value);
    let snapshot = handle.watch_state().latest();
    assert_eq!(snapshot.state().suites.cases_for_suite(suite.metadata.identifier.as_str()).count(), 1);
    assert!(snapshot.state().executions.is_empty());
    assert!(snapshot.state().run_requests.is_empty());
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn registered_cases_selection() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let suite = support::fixture_suite(&["passed"]);
    owner.handle().set_suites("provider:fixture", SuiteSet::new([support::fixture_registration(suite.clone())])).await.unwrap();
    let server = Server::start(owner.handle(), Config::local()).await.unwrap();
    let planning = WireClient::connect(server.local_endpoint()).await.unwrap();
    let request =
        contracts::planning::RegisterSelection { plan: Some(plan(vec![selection(&suite)])), idempotency_key: "register-once".into() };
    let first = planning.unary::<contracts::planning::RegisterSelection>(request.clone()).await.unwrap().into_inner();
    let retried = planning.unary::<contracts::planning::RegisterSelection>(request).await.unwrap().into_inner();
    assert_eq!(first.cases, retried.cases);
    assert!(owner.handle().watch_state().latest().state().cycles.is_empty());
    let ids = vec![first.cases[0].case_id.clone(); 2];
    let selected = model::planning::wire::SelectionPlan { case_ids: ids.clone(), ..Default::default() };
    let preview = planning
        .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(selected.clone()) })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(preview.cases.iter().map(|case| case.case_id.clone()).collect::<Vec<_>>(), ids);
    let execution = WireClient::connect(server.local_endpoint()).await.unwrap();
    let cycle_id = support::planned_cycle(&execution, Request::new(create()), selected).await.unwrap();
    let snapshot = owner.handle().watch_state().latest();
    let id = dokime_runtime::TestCycleId::from_uuid(cycle_id).unwrap();
    let cycle = snapshot.state().cycle_view(&id).unwrap();
    assert_eq!(cycle.test_executions.len(), 2);
    assert_ne!(cycle.test_executions[0].test_execution_id, cycle.test_executions[1].test_execution_id);
    assert_eq!(cycle.test_executions[0].test_case_id, cycle.test_executions[1].test_case_id);

    let mixed = model::planning::wire::SelectionPlan { case_ids: ids, selections: vec![selection(&suite)], ..Default::default() };
    assert_eq!(
        planning
            .unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection { plan: Some(mixed) })
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
