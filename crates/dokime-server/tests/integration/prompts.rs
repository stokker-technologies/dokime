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
use std::collections::BTreeMap;
use std::sync::Arc;
// Third Party Imports
use crate::support::wire::WireMessage;
use clap::Parser;
use prost::Message;
// Local Imports
use super::*;
use dokime::domain::model;
use {
    dokime::PromptId, dokime::RecordScope, dokime::TestCycleId, dokime::Value, dokime::ValueType, dokime::domains::prompts::FormPrompt,
    dokime::domains::prompts::Prompt, dokime::domains::prompts::PromptField, dokime::domains::prompts::PromptFieldKind,
    dokime::domains::prompts::PromptRequest, dokime::domains::prompts::Rule, dokime::expressions::BooleanExpression,
    dokime::runtime::command::CommandContext,
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

#[tokio::test]
async fn execution_prompt_identity() {
    // === Admit One Execution And Keep Its Provider Running === //

    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let definition = dokime::domains::prompts::ConfirmationPrompt::new("fixture.confirm", "Start Capture").definition();
    let config = prompt_config(root.path(), definition.clone());
    let mut owner = Runtime::with_execution_fixture(config, "fixture", gate.clone()).await.unwrap();
    let handle = owner.handle();
    let mut suite = support::fixture_suite(&["passed"]);
    suite.test_case_templates[0].metadata.identifier = Identifier::new("fixture.suite.probe").unwrap();
    suite.test_case_templates[0].prompt_definitions.push(definition);
    support::register_fixture_execution(&handle, suite.clone()).await;
    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
    let endpoint = server.local_endpoint();
    let target = dokime::client::Target::local(endpoint.clone()).unwrap();
    let client = dokime::client::Client::connect(target, None, None).await.unwrap();
    let cycle = client.create_cycle(create()).await.unwrap();
    let selection = model::planning::wire::RegisteredSelection {
        suite_id: suite.metadata.identifier.to_string(),
        template_id: suite.test_case_templates[0].test_case_template_id.to_string(),
        ..Default::default()
    };
    let cases = client.materialize_template(contracts::planning::MaterializeTemplate { selection: Some(selection) }).await.unwrap();
    let first = uuid::Uuid::now_v7().to_string();
    let sibling = uuid::Uuid::now_v7().to_string();
    let bindings = [&first, &sibling]
        .into_iter()
        .map(|id| contracts::execution::ExecutionBinding { execution_id: id.clone(), case_id: cases.cases[0].case_id.clone() })
        .collect();
    cycle.append(bindings).await.unwrap();
    assert_ne!(first, sibling);
    let execution_id = dokime::TestExecutionId::from_uuid(&first).unwrap();
    let execution = client.execution(execution_id.clone());
    let other = client.execution(dokime::TestExecutionId::from_uuid(&sibling).unwrap());
    let command = ["dokime", "execution", "start", &first, "--endpoint", &endpoint];
    dokime_cli::Cli::try_parse_from(command).unwrap().run().await.unwrap();
    gate.started.notified().await;

    // Recovery Must Name This Execution's Explicitly Owned Container.
    let recovery_directory = root.path().join("recovery");
    std::fs::create_dir(&recovery_directory).unwrap();
    let recovery_path = recovery_directory.to_str().unwrap();
    let cycle_text = cycle.id().to_string();
    let foreign_cycle = TestCycleId::from_seed("foreign-recovery-cycle").to_string();
    for leaf in [vec!["execution", "wait"], vec!["execution", "prompt", "next"]] {
        let mut command = vec!["dokime"];
        command.extend(leaf);
        command.extend([
            &first,
            "--cancel-on-interrupt",
            "--owned-cycle",
            &foreign_cycle,
            "--recovery-directory",
            recovery_path,
            "--endpoint",
            &endpoint,
        ]);
        let refused = dokime_cli::Cli::try_parse_from(command).unwrap().run().await.unwrap_err();
        assert_eq!(refused.exit_code(), std::process::ExitCode::from(2));
        assert!(refused.to_string().contains("not explicitly owned cycle"));
        assert_eq!(handle.watch_state().latest().state().cycles[cycle.id()].phase, model::execution::result::CyclePhase::Active);
        assert!(execution.checkpoint().await.is_err(), "wrong-container recovery cannot stop this execution");
        assert_eq!(std::fs::read_dir(&recovery_directory).unwrap().count(), 0);
    }

    // === Read And Answer Only The Explicit Execution's Question === //

    let id = dokime::model::identity::new_id::<PromptId>();
    handle
        .request_prompt(PromptRequest {
            prompt_id: id.clone(),
            prompt_type: Identifier::new("fixture.confirm").unwrap(),
            values: BTreeMap::new(),
            scope: RecordScope::default().with_cycle(cycle.id().clone()).with_test_execution(execution_id),
        })
        .await
        .unwrap();
    let pending = execution.pending_prompts().await.unwrap();
    assert_eq!(pending.message().await.unwrap().unwrap().prompt_id, id.to_string());
    assert!(pending.message().await.unwrap().is_none());
    pending.close().await;
    let foreign = other.prompt(&id).await.unwrap_err();
    assert!(matches!(foreign, dokime::client::Error::Rpc(status) if status.code() == Code::InvalidArgument));
    let values = model::values::wire::Value::from(Value::Object(BTreeMap::new()));
    assert!(other.respond_prompt(&id, "yes".into(), values, true).await.is_err());
    assert!(handle.prompt_response(&id).await.is_none());
    let id_text = id.to_string();
    let command = ["dokime", "execution", "prompt", "next", &first, "--wait-seconds", "5", "--id-only", "--endpoint", &endpoint];
    dokime_cli::Cli::try_parse_from(command).unwrap().run().await.unwrap();
    let command = ["dokime", "execution", "prompt", "show", &first, &id_text, "--endpoint", &endpoint];
    dokime_cli::Cli::try_parse_from(command).unwrap().run().await.unwrap();
    let command = ["dokime", "execution", "prompt", "answer", &first, &id_text, "--action", "yes", "--endpoint", &endpoint];
    dokime_cli::Cli::try_parse_from(command).unwrap().run().await.unwrap();
    assert_eq!(handle.prompt_response(&id).await.unwrap().action, "yes");
    assert!(
        dokime_cli::Cli::try_parse_from(command).unwrap().run().await.is_err(),
        "a duplicate answer cannot replace the committed response"
    );
    assert!(execution.prompt(&id).await.unwrap().response.is_some());

    // === Wait For The Durable Result Without Sealing The Container === //

    gate.release.notify_one();
    let command = [
        "dokime",
        "execution",
        "wait",
        &first,
        "--wait-seconds",
        "5",
        "--endpoint",
        &endpoint,
        "--cancel-on-interrupt",
        "--owned-cycle",
        &cycle_text,
        "--recovery-directory",
        recovery_path,
    ];
    dokime_cli::Cli::try_parse_from(command).unwrap().run().await.unwrap();
    assert_eq!(handle.watch_state().latest().state().cycles[cycle.id()].phase, model::execution::result::CyclePhase::Active);
    assert_eq!(std::fs::read_dir(&recovery_directory).unwrap().count(), 0, "normal wait never seals or exports the owned cycle");

    // === Stop A Different Execution While Its Question Waits === //

    let command = ["dokime", "execution", "start", &sibling, "--endpoint", &endpoint];
    dokime_cli::Cli::try_parse_from(command).unwrap().run().await.unwrap();
    gate.started.notified().await;
    let waiting_id = dokime::model::identity::new_id::<PromptId>();
    let other_id = dokime::TestExecutionId::from_uuid(&sibling).unwrap();
    handle
        .request_prompt(PromptRequest {
            prompt_id: waiting_id.clone(),
            prompt_type: Identifier::new("fixture.confirm").unwrap(),
            values: BTreeMap::new(),
            scope: RecordScope::default().with_cycle(cycle.id().clone()).with_test_execution(other_id),
        })
        .await
        .unwrap();
    let command = ["dokime", "execution", "stop", &sibling, "--endpoint", &endpoint];
    dokime_cli::Cli::try_parse_from(command).unwrap().run().await.unwrap();
    let command = ["dokime", "execution", "wait", &sibling, "--wait-seconds", "5", "--endpoint", &endpoint];
    let cancelled = dokime_cli::Cli::try_parse_from(command).unwrap().run().await.unwrap_err();
    assert_eq!(cancelled.exit_code(), std::process::ExitCode::from(1), "cancelled evidence is sealed without claiming success");
    assert!(handle.prompt_response(&waiting_id).await.is_none());
    assert!(handle.prompt_settled_at(&waiting_id).await.is_some());
    let state = handle.watch_state().latest();
    let stopped = state.state().run_requests.values().find(|request| request.selected_execution_ids.contains(other.id())).unwrap();
    assert!(stopped.summary.phase.terminal(), "a sealed Stop must settle its prepared request: {stopped:?}");
    assert_eq!((stopped.summary.prepared, stopped.summary.finished), (1, 1));
    drop(state);
    let values = model::values::wire::Value::from(Value::Object(BTreeMap::new()));
    assert!(other.respond_prompt(&waiting_id, "yes".into(), values, true).await.is_err());
    assert_eq!(execution.wait(None, None).await.unwrap().checkpoint, execution.checkpoint().await.ok());

    // === Materialize Plain Parameters And Append The Explicit Case === //

    let cycle_text = cycle.id().to_string();
    let command = [
        "dokime",
        "plan",
        "materialize",
        "--template",
        "fixture.suite.probe",
        "--parameters-json",
        "{\"fixture.result\":\"manual-value\"}",
        "--id-only",
        "--endpoint",
        &endpoint,
    ];
    dokime_cli::Cli::try_parse_from(command).unwrap().run().await.unwrap();
    let case_id = {
        let state = handle.watch_state().latest();
        state
            .state()
            .suites
            .cases_for_suite(suite.metadata.identifier.as_str())
            .find(|case| case.parameters.get("fixture.result") == Some(&Value::Text("manual-value".into())))
            .unwrap()
            .test_case_id
            .to_string()
    };
    let command = ["dokime", "cycle", "append", &cycle_text, "--case-id", &case_id, "--id-only", "--endpoint", &endpoint];
    dokime_cli::Cli::try_parse_from(command).unwrap().run().await.unwrap();
    let members = cycle.executions().await.unwrap();
    assert_eq!(members.len(), 3);
    let prepared = members.iter().find(|member| member.id().as_str() != first && member.id().as_str() != sibling).unwrap();
    let state = handle.watch_state().latest();
    let prepared = &state.state().executions[prepared.id()];
    assert_eq!(prepared.snapshot.case().parameters["fixture.result"], Value::Text("manual-value".into()));
    drop(state);
    client.close_and_wait().await;
    server.shutdown().await.unwrap();
    drop(handle);
    let shutdown = owner.shutdown().await;
    assert!(shutdown.is_ok(), "shutdown: {shutdown:?}; requests: {:?}", owner.handle().watch_state().latest().state().run_requests);
}

#[tokio::test]
async fn prompt_answers_identity() {
    let root = tempfile::tempdir().unwrap();
    let mut definition = FormPrompt::<BTreeMap<String, i64>>::new("fixture.answer", "Exact answer")
        .field(PromptField {
            metadata: support::metadata("counter", "Counter"),
            value_type: ValueType::Integer,
            kind: PromptFieldKind::Int,
            multiple: false,
            required: true,
            default: Some(Value::Integer(i64::MAX)),
            choices: vec![],
            validation: Default::default(),
        })
        .definition();
    definition.rules =
        vec![Rule { predicate: BooleanExpression::selector("counter").eq(i64::MAX), message: "Keep the exact counter".into() }];
    let mut owner = prompt_runtime(root.path(), definition).await;
    let handle = owner.handle();
    let ids = [dokime::model::identity::new_id::<PromptId>(), dokime::model::identity::new_id::<PromptId>()];
    for id in &ids {
        handle
            .request_prompt(PromptRequest {
                prompt_id: id.clone(),
                prompt_type: Identifier::new("fixture.answer").unwrap(),
                values: BTreeMap::new(),
                scope: RecordScope::default(),
            })
            .await
            .unwrap();
    }
    let server = Server::start(handle.clone(), keyed()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(
        client.unary::<contracts::prompts::ListDefinitions>(contracts::prompts::ListDefinitions::default()).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    let definitions = client
        .unary::<contracts::prompts::ListDefinitions>(authenticated(contracts::prompts::ListDefinitions::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(definitions.definitions.len(), 1);
    let definition = client
        .unary::<contracts::prompts::GetDefinition>(authenticated(contracts::prompts::GetDefinition {
            prompt_type: "fixture.answer".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(definition.fields[0].default_value.as_ref().unwrap().to_core().unwrap(), Value::Integer(i64::MAX));
    assert!(matches!(
        definition.rules[0].predicate.as_ref().unwrap().nodes[0].kind,
        Some(model::prompts::wire::AnswerPredicateKind::Binary(_))
    ));
    let first = client
        .unary::<contracts::prompts::ListPending>(authenticated(contracts::prompts::ListPending { limit: 1, ..Default::default() }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.prompts.len(), 1);
    let after = first.next_after_prompt_id.clone().unwrap();
    let second = client
        .unary::<contracts::prompts::ListPending>(authenticated(contracts::prompts::ListPending {
            limit: 1,
            after_prompt_id: after.clone(),
            cursor: first.cursor.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(second.prompts.len(), 1);
    assert_ne!(first.prompts[0].prompt_id, second.prompts[0].prompt_id);
    assert!(second.next_after_prompt_id.is_none());
    let different = client
        .unary::<contracts::prompts::ListPending>(authenticated(contracts::prompts::ListPending {
            after_prompt_id: after.clone(),
            cursor: first.cursor.clone(),
            cycle_id: Some(uuid::Uuid::now_v7().to_string()),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert_eq!(different.code(), Code::FailedPrecondition);
    let answer = |value| contracts::prompts::Respond {
        prompt_id: ids[0].to_string(),
        action: "submit".into(),
        values: Some(model::values::wire::Value::from(&Value::Object(BTreeMap::from([("counter".into(), Value::Integer(value))])))),
        accepted: true,
        context: None,
    };
    let bad = client.unary::<contracts::prompts::Respond>(authenticated(answer(i64::MAX - 1))).await.unwrap_err();
    assert_eq!(bad.code(), Code::InvalidArgument);
    let envelope = tonic_types::Status::decode(bad.details()).unwrap();
    assert_eq!(envelope.details[0].type_url, "type.googleapis.com/dokime.v1.Rejection");
    assert!(!<model::execution::wire::Rejection as WireMessage>::from_wire(envelope.details[0].value.as_slice()).unwrap().code.is_empty());
    assert!(handle.prompt_response(&ids[0]).await.is_none());
    let receipt = client.unary::<contracts::prompts::Respond>(authenticated(answer(i64::MAX))).await.unwrap().into_inner();
    assert_eq!(receipt.prompt_id, ids[0].to_string());
    assert!(receipt.committed_revision > first.cursor.as_ref().unwrap().position);
    assert!(receipt.accepted_at.is_some());
    let prompt = client
        .unary::<contracts::prompts::GetPrompt>(authenticated(contracts::prompts::GetPrompt { prompt_id: ids[0].to_string() }))
        .await
        .unwrap()
        .into_inner();
    let response = prompt.response.unwrap();
    assert_eq!(response.responder, "remote");
    assert!(matches!(
        response.values.unwrap().to_core().unwrap(),
        Value::Object(fields) if fields.get("counter") == Some(&Value::Integer(i64::MAX))
    ));
    assert!(client.unary::<contracts::prompts::Respond>(authenticated(answer(i64::MAX))).await.is_err());
    assert_eq!(
        client
            .unary::<contracts::prompts::ListPending>(authenticated(contracts::prompts::ListPending {
                after_prompt_id: after,
                cursor: first.cursor,
                ..Default::default()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    assert_eq!(
        client
            .unary::<contracts::prompts::ListPending>(authenticated(contracts::prompts::ListPending::default()))
            .await
            .unwrap()
            .into_inner()
            .prompts
            .len(),
        1
    );
    let read = WireClient::connect(server.local_endpoint()).await.unwrap();
    read.unary::<contracts::prompts::GetPrompt>(auth(contracts::prompts::GetPrompt { prompt_id: ids[0].to_string() }, TOKEN))
        .await
        .unwrap();
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn prompt_condition_commit() {
    use dokime::definition::RuntimeCondition;
    use {
        dokime::records::RecordCondition, model::records::product::ProductConnectionDetails,
        model::records::product::ProductConnectionState,
    };
    let root = tempfile::tempdir().unwrap();
    let mut definition = FormPrompt::<BTreeMap<String, bool>>::new("fixture.condition", "Connect device").definition();
    definition.actions[0].requires_condition = true;
    definition.condition = Some(RuntimeCondition::Record {
        condition: RecordCondition {
            record_type: Identifier::new("tool.connection.acquired").unwrap(),
            match_fields: BTreeMap::from([("resource_id".into(), Value::Text("fixture-device".into()))]),
        },
    });
    let mut owner = prompt_runtime(root.path(), definition).await;
    let handle = owner.handle();
    let id = dokime::model::identity::new_id::<PromptId>();
    handle
        .request_prompt(PromptRequest {
            prompt_id: id.clone(),
            prompt_type: Identifier::new("fixture.condition").unwrap(),
            values: BTreeMap::new(),
            scope: RecordScope::default(),
        })
        .await
        .unwrap();
    let server = Server::start(handle.clone(), keyed()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let view = client
        .unary::<contracts::prompts::GetPrompt>(authenticated(contracts::prompts::GetPrompt { prompt_id: id.to_string() }))
        .await
        .unwrap()
        .into_inner();
    let definition = view.prompt.unwrap().definition.unwrap();
    assert!(definition.actions[0].requires_condition);
    let Some(model::prompts::wire::EvidenceConditionKind::Record(condition)) =
        definition.condition.unwrap().nodes.into_iter().next().unwrap().kind
    else {
        panic!("record condition lost");
    };
    assert_eq!(condition.record_type, "tool.connection.acquired");
    assert!(matches!(
        condition.match_fields.unwrap().to_core().unwrap(),
        Value::Object(fields) if fields.get("resource_id") == Some(&Value::Text("fixture-device".into()))
    ));
    let answer = contracts::prompts::Respond { prompt_id: id.to_string(), action: "submit".into(), accepted: true, ..Default::default() };
    let rejected = client.unary::<contracts::prompts::Respond>(authenticated(answer.clone())).await.unwrap_err();
    assert_eq!(rejected.code(), Code::FailedPrecondition);
    let envelope = tonic_types::Status::decode(rejected.details()).unwrap();
    assert_eq!(
        <model::execution::wire::Rejection as WireMessage>::from_wire(envelope.details[0].value.as_slice()).unwrap().code,
        "prompt_condition_not_satisfied"
    );
    assert!(handle.prompt_response(&id).await.is_none());
    handle
        .publish_product_record(ProductConnectionDetails::new("fixture-device", ProductConnectionState::Connected).into_draft("fixture"))
        .await
        .unwrap();
    client.unary::<contracts::prompts::Respond>(authenticated(answer)).await.unwrap();
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn prompt_level_watched() {
    use dokime::records::RecordSeverity;

    // === Ask A Warning Prompt === //

    let root = tempfile::tempdir().unwrap();
    let definition =
        FormPrompt::<BTreeMap<String, bool>>::new("fixture.level", "Check the bench").level(RecordSeverity::Warning).definition();
    let mut owner = prompt_runtime(root.path(), definition).await;
    let handle = owner.handle();
    let id = dokime::model::identity::new_id::<PromptId>();
    handle
        .request_prompt(PromptRequest {
            prompt_id: id.clone(),
            prompt_type: Identifier::new("fixture.level").unwrap(),
            values: BTreeMap::new(),
            scope: RecordScope::default(),
        })
        .await
        .unwrap();
    let server = Server::start(handle.clone(), keyed()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();

    // === Read The Prompt And Its Request Record === //

    let view = client
        .unary::<contracts::prompts::GetPrompt>(authenticated(contracts::prompts::GetPrompt { prompt_id: id.to_string() }))
        .await
        .unwrap()
        .into_inner();
    let mut follow = client
        .download::<contracts::evidence::WatchRecords>(authenticated(contracts::evidence::WatchRecords {
            selection: Some(model::evidence::wire::CapturedRecordSelection {
                record_types: vec!["prompt.request.form".into()],
                ..Default::default()
            }),
            after: None,
        }))
        .await
        .unwrap()
        .into_inner();
    let mut watched = Vec::new();
    while watched.is_empty() {
        watched = follow.message().await.unwrap().unwrap().records;
    }

    // === Compare === //

    let level = view.prompt.unwrap().definition.unwrap().level;
    assert_eq!(level, model::identity::wire::Severity::Warn);
    assert_eq!(watched.len(), 1);
    assert_eq!(watched[0].severity, model::identity::wire::Severity::Warn);
    drop(follow);
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn settled_prompt_refused() {
    // === Ask A Cycle's Question With No Cancel Action === //

    let root = tempfile::tempdir().unwrap();
    let mut definition = FormPrompt::<BTreeMap<String, bool>>::new("fixture.settle", "Inspect Charging State").definition();
    definition.actions.retain(|action| action.id != "cancel");
    let mut owner = prompt_runtime(root.path(), definition).await;
    let handle = owner.handle();
    let cycle = handle
        .create_cycle(
            dokime::transactions::CycleDefinition {
                metadata: support::metadata("cycle.prompt.settle", "Prompt Settle"),
                continue_automatically: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let id = dokime::model::identity::new_id::<PromptId>();
    handle
        .request_prompt(PromptRequest {
            prompt_id: id.clone(),
            prompt_type: Identifier::new("fixture.settle").unwrap(),
            values: BTreeMap::new(),
            scope: RecordScope::default().with_cycle(cycle.clone()),
        })
        .await
        .unwrap();
    let server = Server::start(handle.clone(), keyed()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let pending = |cycle: &TestCycleId| contracts::prompts::ListPending { cycle_id: Some(cycle.to_string()), ..Default::default() };
    let waiting = client.unary::<contracts::prompts::ListPending>(authenticated(pending(&cycle))).await.unwrap().into_inner();
    assert_eq!(waiting.prompts.len(), 1);

    // === Seal The Cycle === //

    handle.seal_cycle(cycle.clone(), CommandContext::default()).await.unwrap();

    // === Nothing Waits, And A Late Answer Is A Failed Precondition === //

    let settled = client.unary::<contracts::prompts::ListPending>(authenticated(pending(&cycle))).await.unwrap().into_inner();
    assert!(settled.prompts.is_empty());
    let late = client
        .unary::<contracts::prompts::Respond>(authenticated(contracts::prompts::Respond {
            prompt_id: id.to_string(),
            action: "submit".into(),
            values: None,
            accepted: true,
            context: None,
        }))
        .await
        .unwrap_err();
    assert_eq!(late.code(), Code::FailedPrecondition);
    assert!(handle.prompt_response(&id).await.is_none());
    assert!(handle.prompt_settled_at(&id).await.is_some());

    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

async fn prompt_runtime(root: &std::path::Path, definition: dokime::domains::prompts::PromptDefinition) -> Runtime {
    Runtime::start(prompt_config(root, definition)).await.unwrap()
}

fn prompt_config(root: &std::path::Path, definition: dokime::domains::prompts::PromptDefinition) -> dokime::RuntimeConfig {
    let mut declarations = harmos::sidecar::Registrar::new();
    declarations
        .register(
            dokime::plugin::registration::PROMPTS_NAMESPACE,
            dokime::plugin::registration::domain_schema(dokime::plugin::registration::PROMPTS_NAMESPACE),
            &[definition],
        )
        .unwrap();
    crate::support::budgets::runtime_config(root, support::metadata("grpc.prompts", "Questions"))
        .with_declared_plugin("prompt.fixture", "1.0.0", "Questions", declarations)
        .unwrap()
}

/* === Tests === */
