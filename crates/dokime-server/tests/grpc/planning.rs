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
use prost::Message;
// Local Imports
use super::*;
use dokime_app::core::Value;
use dokime_app::definition::{FileDefinition, ParameterExpr, ParameterSelection, SuiteSet};
use dokime_protocol::v1::{
    catalog_client::CatalogClient, execution_client::ExecutionClient,
    planning_client::PlanningClient,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn selection(suite: &dokime_app::definition::TestSuite) -> v1::RegisteredSelection {
    v1::RegisteredSelection {
        suite_id: suite.metadata.identifier.to_string(),
        template_id: suite.test_case_templates[0]
            .test_case_template_id
            .to_string(),
        ..Default::default()
    }
}

fn integer(value: i64) -> v1::Value {
    v1::Value {
        kind: Some(v1::value::Kind::IntegerValue(value)),
    }
}

fn plan(selections: Vec<v1::RegisteredSelection>) -> v1::SelectionPlan {
    v1::SelectionPlan {
        selections,
        max_cases: 0,
        case_ids: vec![],
    }
}

fn create(plan: v1::SelectionPlan) -> v1::CreateCycleRequest {
    v1::CreateCycleRequest {
        plan: Some(plan),
        metadata: Some(v1::Metadata {
            identifier: "planning.cycle".into(),
            label: "Plan".into(),
            description: None,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn error_code(status: &tonic::Status) -> String {
    let envelope = tonic_types::Status::decode(status.details()).unwrap();
    v1::Rejection::decode(envelope.details[0].value.as_slice())
        .unwrap()
        .code
}

pub(crate) async fn ordered_selections_provider() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let mut first = common::fixture_suite(&["passed"]);
    first.tags = vec!["suite-tag".into()];
    let parameter = &mut first.test_case_templates[0].parameters[0];
    parameter.expansion = dokime_app::definition::ParameterExpansion::Cartesian;
    parameter.value_type = dokime_app::core::ValueType::Integer;
    parameter.expression = ParameterExpr::Literal {
        value: Value::Integer(0),
    };
    first.test_case_templates[0].files.push(FileDefinition {
        metadata: common::metadata("fixture.file", "Input"),
        default: Some("plugin:default".into()),
        required: false,
        media_types: vec!["application/octet-stream".into()],
        attributes: [("maximum".into(), Value::Integer(i64::MAX))].into(),
    });
    let mut second = common::fixture_suite(&["passed"]);
    second.metadata = common::metadata("fixture.second", "Second");
    second.test_case_templates[0].metadata =
        common::metadata("fixture.second.template", "Second template");
    let parameter = &mut second.test_case_templates[0].parameters[0];
    parameter.list = Some(dokime_app::definition::ParameterList {
        element_type: dokime_app::core::ValueType::Integer,
        min_items: None,
        max_items: None,
    });
    parameter.value_type = dokime_app::core::ValueType::Array;
    parameter.expression = ParameterExpr::Literal {
        value: Value::Array(vec![Value::Integer(0)]),
    };
    parameter.selection = Some(ParameterSelection { choices: vec![] });
    owner
        .handle()
        .set_suites(
            "provider:fixture",
            SuiteSet::new([
                common::fixture_registration(first.clone()),
                common::fixture_registration(second.clone()),
            ]),
        )
        .await
        .unwrap();
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap(),
    )
    .await
    .unwrap();
    let mut planning = PlanningClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let mut selected = selection(&first);
    selected.parameters.insert(
        "fixture.result".into(),
        v1::SelectionValues {
            values: vec![integer(i64::MIN), integer(i64::MAX), integer(i64::MIN)],
        },
    );
    let mut multiple = selection(&second);
    multiple.parameters.insert(
        "fixture.result".into(),
        v1::SelectionValues {
            values: vec![v1::Value {
                kind: Some(v1::value::Kind::ArrayValue(v1::ValueList {
                    values: vec![integer(i64::MIN), integer(i64::MAX), integer(i64::MIN)],
                })),
            }],
        },
    );
    let input = plan(vec![selected.clone(), multiple, selected]);
    let preview = planning
        .preview_selection(authenticated(v1::PreviewSelectionRequest {
            plan: Some(input.clone()),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(preview.cases.len(), 7);
    assert_eq!(
        preview
            .cases
            .iter()
            .map(|case| case.selection_index)
            .collect::<Vec<_>>(),
        [0, 0, 0, 1, 2, 2, 2]
    );
    assert_eq!(
        preview
            .cases
            .iter()
            .map(|case| &case.case_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        7
    );
    assert_eq!(
        preview.cases[0].parameters.as_ref().unwrap().fields["fixture.result"],
        integer(i64::MIN)
    );
    assert_eq!(
        preview.cases[1].parameters.as_ref().unwrap().fields["fixture.result"],
        integer(i64::MAX)
    );
    assert!(matches!(
        preview.cases[3].parameters.as_ref().unwrap().fields["fixture.result"].kind,
        Some(v1::value::Kind::ArrayValue(_))
    ));
    let repeated = planning
        .preview_selection(authenticated(v1::PreviewSelectionRequest {
            plan: Some(input.clone()),
        }))
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
    let mut catalog = CatalogClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let definition = catalog
        .get_suite(authenticated(v1::GetSuiteRequest {
            suite_id: first.metadata.identifier.to_string(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(definition.tags, ["fixture", "suite-tag"]);
    assert_eq!(
        definition.templates[0].steps[0].step_id,
        first.test_case_templates[0].phases[0].step_id.to_string()
    );
    let file = &definition.templates[0].files[0];
    assert_eq!(file.default_value.as_deref(), Some("plugin:default"));
    assert_eq!(
        file.attributes.as_ref().unwrap().fields["maximum"],
        integer(i64::MAX)
    );
    let mut execution = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let reply = execution
        .create_cycle(authenticated(create(input)))
        .await
        .unwrap()
        .into_inner();
    let snapshot = owner.handle().watch_state().latest();
    let id = dokime_app::core::TestCycleId::from_uuid(reply.cycle_id).unwrap();
    let cycle = snapshot.state().cycle_view(&id).unwrap();
    assert_eq!(cycle.lifecycle, dokime_app::execution::CycleStatus::Ready);
    assert_eq!(cycle.test_executions.len(), cycle.cases().count());
    for (execution, case) in cycle.test_executions.iter().zip(cycle.cases()) {
        assert_eq!(execution.test_case_id, case.test_case_id);
        assert_eq!(
            execution.status,
            dokime_app::execution::ExecutionStatus::Skipped
        );
        assert!(execution.started_at.is_none());
    }
    for (case, expected) in cycle.cases().zip(&preview.cases) {
        assert_ne!(case.test_case_id.as_str(), expected.case_id);
        let suite = snapshot.state().suites.suite_for_case(case).unwrap();
        assert_eq!(suite.suite_id, expected.suite_id);
        assert_eq!(case.suite.as_ref(), Some(&suite.suite.metadata));
        assert_eq!(case.test_case_template_id.as_str(), expected.template_id);
        let target = snapshot
            .state()
            .suites
            .provider_target_for_case(case)
            .unwrap();
        assert_eq!(target.source, "provider:fixture");
        assert_eq!(target.binding.workflow(), "execute_cycle");
    }
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn planning_refuses_inputs() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let mut first = common::fixture_suite(&["passed"]);
    first.test_case_templates[0].files.push(FileDefinition {
        metadata: common::metadata("fixture.file", "Required"),
        default: Some("plugin:default".into()),
        required: true,
        media_types: vec![],
        attributes: Default::default(),
    });
    let mut second = common::fixture_suite(&["passed"]);
    second.metadata = common::metadata("fixture.second", "Second");
    second.test_case_templates[0].metadata =
        common::metadata("fixture.second.template", "Second template");
    let mut other = common::fixture_registration(second.clone());
    other.workflow = Some("another_workflow".into());
    owner
        .handle()
        .set_suites(
            "provider:fixture",
            SuiteSet::new([common::fixture_registration(first.clone()), other]),
        )
        .await
        .unwrap();
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let mut planning = PlanningClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let mixed = plan(vec![selection(&first), selection(&second)]);
    let failure = planning
        .preview_selection(v1::PreviewSelectionRequest {
            plan: Some(mixed.clone()),
        })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "mixed_provider_targets");
    owner
        .handle()
        .set_suites(
            "provider:fixture",
            SuiteSet::new([common::fixture_registration(first.clone())]),
        )
        .await
        .unwrap();
    owner
        .handle()
        .set_suites(
            "provider:other",
            SuiteSet::new([common::fixture_registration(second.clone())]),
        )
        .await
        .unwrap();
    let failure = planning
        .preview_selection(v1::PreviewSelectionRequest {
            plan: Some(mixed.clone()),
        })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "mixed_provider_targets");
    let mut execution = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let mut invalid = selection(&first);
    invalid.files.insert(
        "fixture.file".into(),
        v1::SelectedFile {
            kind: Some(v1::selected_file::Kind::Clear(0)),
        },
    );
    let mut unknown = selection(&first);
    unknown.template_id = first.test_case_templates[0].metadata.identifier.to_string();
    let mut cartesian = selection(&first);
    cartesian.parameters.insert(
        "fixture.result".into(),
        v1::SelectionValues {
            values: vec![
                v1::Value {
                    kind: Some(v1::value::Kind::TextValue("a".into())),
                },
                v1::Value {
                    kind: Some(v1::value::Kind::TextValue("b".into())),
                },
            ],
        },
    );
    for (label, expected, input) in [
        ("no selections", Code::InvalidArgument, plan(vec![])),
        ("a cleared file", Code::InvalidArgument, plan(vec![invalid])),
        ("an unknown template", Code::NotFound, plan(vec![unknown])),
        (
            "a case ceiling above the limit",
            Code::ResourceExhausted,
            v1::SelectionPlan {
                max_cases: 1001,
                ..plan(vec![selection(&first)])
            },
        ),
        (
            "an expansion past its own ceiling",
            Code::InvalidArgument,
            v1::SelectionPlan {
                max_cases: 3,
                ..plan(vec![cartesian.clone(), cartesian])
            },
        ),
        (
            "more selections than the request admits",
            Code::ResourceExhausted,
            plan(vec![selection(&first); 257]),
        ),
    ] {
        assert_eq!(
            planning
                .preview_selection(v1::PreviewSelectionRequest {
                    plan: Some(input.clone())
                })
                .await
                .unwrap_err()
                .code(),
            expected,
            "previewing {label}"
        );
        assert_eq!(
            execution
                .create_cycle(create(input))
                .await
                .unwrap_err()
                .code(),
            expected,
            "creating a cycle from {label}"
        );
    }
    assert!(owner.handle().cycles().await.is_empty());
    server.shutdown().await.unwrap();
    let read = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Read)])
            .unwrap(),
    )
    .await
    .unwrap();
    let mut planning = PlanningClient::connect(read.local_endpoint())
        .await
        .unwrap();
    planning
        .preview_selection(authenticated(v1::PreviewSelectionRequest {
            plan: Some(plan(vec![selection(&first)])),
        }))
        .await
        .unwrap();
    assert_eq!(
        planning
            .preview_selection(v1::PreviewSelectionRequest {
                plan: Some(plan(vec![selection(&first)]))
            })
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let mut execution = ExecutionClient::connect(read.local_endpoint())
        .await
        .unwrap();
    assert_eq!(
        execution
            .create_cycle(authenticated(create(plan(vec![selection(&first)]))))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    read.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn template_conflicts_disambiguate() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let first = common::fixture_suite(&["passed"]);
    let mut second = common::fixture_suite(&["passed"]);
    second.metadata = common::metadata("fixture.second", "Second");
    // Unique UUIDs are insufficient: native capabilities resolve this identifier.
    owner
        .handle()
        .set_suites(
            "provider:fixture",
            SuiteSet::new([
                common::fixture_registration(first.clone()),
                common::fixture_registration(second.clone()),
            ]),
        )
        .await
        .unwrap();
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let mut planning = PlanningClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let failure = planning
        .preview_selection(v1::PreviewSelectionRequest {
            plan: Some(plan(vec![selection(&first)])),
        })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "ambiguous_registered_identity");
    second.test_case_templates[0].metadata = common::metadata("fixture.second.template", "Second");
    second.metadata = first.metadata.clone();
    let mut registration = common::fixture_registration(second);
    registration.suite_id = "fixture.second".into();
    owner
        .handle()
        .set_suites(
            "provider:fixture",
            SuiteSet::new([common::fixture_registration(first.clone()), registration]),
        )
        .await
        .unwrap();
    let mut execution = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let created = execution
        .create_cycle(create(plan(vec![selection(&first)])))
        .await
        .unwrap()
        .into_inner();
    let cycles = owner.handle().cycles().await;
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].test_cycle_id.as_str(), created.cycle_id);
    assert!(
        cycles[0]
            .cases()
            .all(|case| case.suite_id == first.metadata.identifier.as_str())
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn effective_native_materialization() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let mut suite = common::fixture_suite(&["passed"]);
    suite.test_case_templates[0].parameters[0].expansion =
        dokime_app::definition::ParameterExpansion::Cartesian;
    suite.test_case_templates[0].parameters[0].value_type = dokime_app::core::ValueType::Integer;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let mut planning = PlanningClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let mut execution = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    for expression in [
        ParameterExpr::Range {
            start: 0,
            stop: i64::MAX,
            step: 1,
        },
        ParameterExpr::Range {
            start: i64::MIN,
            stop: i64::MIN,
            step: -1,
        },
        ParameterExpr::Arange {
            start: 1e20,
            stop: 2e20,
            step: 1.0,
        },
        ParameterExpr::Array {
            values: vec![ParameterExpr::Linspace {
                start: 0.0,
                stop: 1.0,
                count: usize::MAX,
            }],
        },
    ] {
        suite.test_case_templates[0].parameters[0].expression = expression;
        common::register_fixture_execution(&owner.handle(), suite.clone()).await;
        let input = plan(vec![selection(&suite)]);
        let failure = planning
            .preview_selection(v1::PreviewSelectionRequest {
                plan: Some(input.clone()),
            })
            .await
            .unwrap_err();
        assert_eq!(error_code(&failure), "parameter_expansion_limit");
        let failure = execution.create_cycle(create(input)).await.unwrap_err();
        assert_eq!(error_code(&failure), "parameter_expansion_limit");
        // The registered expression is irrelevant when explicitly overridden.
        let mut selected = selection(&suite);
        selected.parameters.insert(
            "fixture.result".into(),
            v1::SelectionValues {
                values: vec![integer(7)],
            },
        );
        let preview = planning
            .preview_selection(v1::PreviewSelectionRequest {
                plan: Some(plan(vec![selected])),
            })
            .await
            .unwrap()
            .into_inner();
        assert_eq!(preview.cases.len(), 1);
        assert_eq!(
            preview.cases[0].parameters.as_ref().unwrap().fields["fixture.result"],
            integer(7)
        );
    }
    suite.test_case_templates[0].parameters[0].expression = ParameterExpr::Range {
        start: 0,
        stop: 100,
        step: 1,
    };
    let mut other = suite.test_case_templates[0].parameters[0].clone();
    other.parameter_id = dokime_app::core::ParameterId::new();
    other.metadata = common::metadata("fixture.other", "Other dimension");
    suite.test_case_templates[0].parameters.push(other);
    common::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let input = plan(vec![selection(&suite)]);
    let failure = planning
        .preview_selection(v1::PreviewSelectionRequest {
            plan: Some(input.clone()),
        })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "invalid_selection");
    assert!(failure.message().contains("10201"));
    assert!(execution.create_cycle(create(input)).await.is_err());
    assert!(owner.handle().cycles().await.is_empty());
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn payload_budget_allocation() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let mut suite = common::fixture_suite(&["passed"]);
    let template = &mut suite.test_case_templates[0];
    template.parameters[0].expansion = dokime_app::definition::ParameterExpansion::Cartesian;
    template.parameters[0].value_type = dokime_app::core::ValueType::Integer;
    template.parameters[0].expression = ParameterExpr::Range {
        start: 0,
        stop: 999,
        step: 1,
    };
    for index in 0..32 {
        template.files.push(FileDefinition {
            metadata: common::metadata(&format!("fixture.file{index}"), "File"),
            default: Some("x".repeat(4096)),
            required: false,
            media_types: vec![],
            attributes: Default::default(),
        });
    }
    common::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let mut planning = PlanningClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let mut execution = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let input = plan(vec![selection(&suite)]);
    let failure = planning
        .preview_selection(v1::PreviewSelectionRequest {
            plan: Some(input.clone()),
        })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "planning_payload_limit");
    assert_eq!(
        error_code(&execution.create_cycle(create(input)).await.unwrap_err()),
        "planning_payload_limit"
    );
    // A smaller explicit selection is valid with the same large default files.
    let mut selected = selection(&suite);
    selected.parameters.insert(
        "fixture.result".into(),
        v1::SelectionValues {
            values: vec![integer(0)],
        },
    );
    assert_eq!(
        planning
            .preview_selection(v1::PreviewSelectionRequest {
                plan: Some(plan(vec![selected]))
            })
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
    common::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let failure = planning
        .preview_selection(v1::PreviewSelectionRequest {
            plan: Some(plan(vec![selection(&suite)])),
        })
        .await
        .unwrap_err();
    assert_eq!(error_code(&failure), "planning_payload_limit");
    suite.test_case_templates[0].metadata.label = "Template".into();
    suite.metadata.description = None;
    let template = &mut suite.test_case_templates[0];
    template.parameters[0].value_type = dokime_app::core::ValueType::Array;
    template.parameters[0].expression = ParameterExpr::Literal {
        value: Value::Array(vec![Value::Text("x".repeat(1024)); 32]),
    };
    let mut other = template.parameters[0].clone();
    other.parameter_id = dokime_app::core::ParameterId::new();
    other.metadata = common::metadata("fixture.other", "Other");
    other.expression = ParameterExpr::Array {
        values: vec![
            ParameterExpr::Reference {
                parameter: "fixture.result".into()
            };
            1000
        ],
    };
    template.parameters.push(other);
    common::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let input = plan(vec![selection(&suite)]);
    assert_eq!(
        error_code(
            &planning
                .preview_selection(v1::PreviewSelectionRequest {
                    plan: Some(input.clone())
                })
                .await
                .unwrap_err()
        ),
        "planning_payload_limit"
    );
    assert_eq!(
        error_code(&execution.create_cycle(create(input)).await.unwrap_err()),
        "planning_payload_limit"
    );
    assert!(owner.handle().cycles().await.is_empty());
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */

pub(crate) async fn registered_cases_selection() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let suite = common::fixture_suite(&["passed"]);
    owner
        .handle()
        .set_suites(
            "provider:fixture",
            SuiteSet::new([common::fixture_registration(suite.clone())]),
        )
        .await
        .unwrap();
    let server = Server::start(owner.handle(), Config::local())
        .await
        .unwrap();
    let mut planning = PlanningClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let request = v1::RegisterSelectionRequest {
        plan: Some(plan(vec![selection(&suite)])),
        idempotency_key: "register-once".into(),
    };
    let first = planning
        .register_selection(request.clone())
        .await
        .unwrap()
        .into_inner();
    let retried = planning
        .register_selection(request)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.cases, retried.cases);
    assert!(
        owner
            .handle()
            .watch_state()
            .latest()
            .state()
            .cycles
            .is_empty()
    );
    let ids = vec![first.cases[0].case_id.clone(); 2];
    let selected = v1::SelectionPlan {
        case_ids: ids.clone(),
        ..Default::default()
    };
    let preview = planning
        .preview_selection(v1::PreviewSelectionRequest {
            plan: Some(selected.clone()),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        preview
            .cases
            .iter()
            .map(|case| case.case_id.clone())
            .collect::<Vec<_>>(),
        ids
    );
    let mut execution = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let created = execution
        .create_cycle(create(selected))
        .await
        .unwrap()
        .into_inner();
    let snapshot = owner.handle().watch_state().latest();
    let id = dokime_app::core::TestCycleId::from_uuid(created.cycle_id).unwrap();
    let cycle = snapshot.state().cycle_view(&id).unwrap();
    assert_eq!(cycle.test_executions.len(), 2);
    assert_ne!(
        cycle.test_executions[0].test_execution_id,
        cycle.test_executions[1].test_execution_id
    );
    assert_eq!(
        cycle.test_executions[0].test_case_id,
        cycle.test_executions[1].test_case_id
    );

    let mixed = v1::SelectionPlan {
        case_ids: ids,
        selections: vec![selection(&suite)],
        ..Default::default()
    };
    assert_eq!(
        planning
            .preview_selection(v1::PreviewSelectionRequest { plan: Some(mixed) })
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
