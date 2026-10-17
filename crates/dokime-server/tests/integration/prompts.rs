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
// Third Party Imports
use prost::Message;
// Local Imports
use super::*;
use dokime_runtime::{
    core::{PromptId, RecordScope, TestCycleId, Value, ValueType},
    domains::prompts::{FormPrompt, Prompt, PromptField, PromptFieldKind, PromptRequest, Rule},
    expressions::BooleanExpression,
    runtime::command::CommandContext,
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
async fn prompt_answers_identity() {
    const READ_TOKEN: &str = "prompt-read-token-012345678901234567890123456789";
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
    definition.rules = vec![Rule {
        predicate: BooleanExpression::selector("counter").eq(i64::MAX),
        message: "Keep the exact counter".into(),
    }];
    let mut owner = prompt_runtime(root.path(), definition).await;
    let handle = owner.handle();
    let ids = [dokime_runtime::model::identity::new_id::<PromptId>(), dokime_runtime::model::identity::new_id::<PromptId>()];
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
    let server = Server::start(
        handle.clone(),
        Config::local()
            .with_bearer_auth(vec![
                grant(Role::Control),
                BearerGrant::new("prompt-reader".into(), Role::Read, READ_TOKEN).unwrap(),
            ])
            .unwrap(),
    )
    .await
    .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(
        client
            .unary::<contracts::prompts::ListDefinitions>(
                contracts::prompts::ListPromptDefinitionsRequest::default()
            )
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let definitions = client
        .unary::<contracts::prompts::ListDefinitions>(authenticated(
            contracts::prompts::ListPromptDefinitionsRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(definitions.definitions.len(), 1);
    let definition = client
        .unary::<contracts::prompts::GetDefinition>(authenticated(
            contracts::prompts::GetPromptDefinitionRequest {
                prompt_type: "fixture.answer".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        definition.fields[0]
            .default_value
            .as_ref()
            .unwrap()
            .to_core()
            .unwrap(),
        Value::Integer(i64::MAX)
    );
    assert!(matches!(
        definition.rules[0].predicate.as_ref().unwrap().nodes[0].kind,
        Some(contracts::prompts::AnswerPredicateKind::Binary(_))
    ));
    let first = client
        .unary::<contracts::prompts::ListPending>(authenticated(
            contracts::prompts::ListPendingPromptsRequest {
                limit: 1,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.prompts.len(), 1);
    let after = first.next_after_prompt_id.clone().unwrap();
    let second = client
        .unary::<contracts::prompts::ListPending>(authenticated(
            contracts::prompts::ListPendingPromptsRequest {
                limit: 1,
                after_prompt_id: after.clone(),
                cursor: first.cursor.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(second.prompts.len(), 1);
    assert_ne!(first.prompts[0].prompt_id, second.prompts[0].prompt_id);
    assert!(second.next_after_prompt_id.is_none());
    let different = client
        .unary::<contracts::prompts::ListPending>(authenticated(
            contracts::prompts::ListPendingPromptsRequest {
                after_prompt_id: after.clone(),
                cursor: first.cursor.clone(),
                cycle_id: Some(uuid::Uuid::now_v7().to_string()),
                ..Default::default()
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(different.code(), Code::FailedPrecondition);
    let answer = |value| contracts::prompts::RespondToPromptRequest {
        prompt_id: ids[0].to_string(),
        action: "submit".into(),
        values: Some(contracts::value::Value::from(&Value::Object(
            BTreeMap::from([("counter".into(), Value::Integer(value))]),
        ))),
        accepted: true,
        context: None,
    };
    let bad = client
        .unary::<contracts::prompts::Respond>(authenticated(answer(i64::MAX - 1)))
        .await
        .unwrap_err();
    assert_eq!(bad.code(), Code::InvalidArgument);
    let envelope = tonic_types::Status::decode(bad.details()).unwrap();
    assert_eq!(
        envelope.details[0].type_url,
        "type.googleapis.com/dokime.v1.Rejection"
    );
    assert!(
        !<contracts::execution::Rejection as harmos::Protobuf>::decode(
            envelope.details[0].value.as_slice()
        )
        .unwrap()
        .code
        .is_empty()
    );
    assert!(handle.prompt_response(&ids[0]).await.is_none());
    let receipt = client
        .unary::<contracts::prompts::Respond>(authenticated(answer(i64::MAX)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(receipt.prompt_id, ids[0].to_string());
    assert!(receipt.committed_revision > first.cursor.as_ref().unwrap().position);
    assert!(receipt.accepted_at.is_some());
    let prompt = client
        .unary::<contracts::prompts::GetPrompt>(authenticated(
            contracts::prompts::GetPromptRequest {
                prompt_id: ids[0].to_string(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let response = prompt.response.unwrap();
    assert_eq!(response.responder, "fixture");
    assert!(matches!(
        response.values.unwrap().to_core().unwrap(),
        Value::Object(fields) if fields.get("counter") == Some(&Value::Integer(i64::MAX))
    ));
    assert!(
        client
            .unary::<contracts::prompts::Respond>(authenticated(answer(i64::MAX)))
            .await
            .is_err()
    );
    assert_eq!(
        client
            .unary::<contracts::prompts::ListPending>(authenticated(
                contracts::prompts::ListPendingPromptsRequest {
                    after_prompt_id: after,
                    cursor: first.cursor,
                    ..Default::default()
                }
            ))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    assert_eq!(
        client
            .unary::<contracts::prompts::ListPending>(authenticated(
                contracts::prompts::ListPendingPromptsRequest::default()
            ))
            .await
            .unwrap()
            .into_inner()
            .prompts
            .len(),
        1
    );
    let read = WireClient::connect(server.local_endpoint()).await.unwrap();
    read.unary::<contracts::prompts::GetPrompt>(auth(
        contracts::prompts::GetPromptRequest {
            prompt_id: ids[0].to_string(),
        },
        READ_TOKEN,
    ))
    .await
    .unwrap();
    assert_eq!(
        read.unary::<contracts::prompts::Respond>(auth(answer(i64::MAX), READ_TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn prompt_condition_commit() {
    use dokime_runtime::definition::RuntimeCondition;
    use dokime_runtime::records::{ProductConnectionDetails, ProductConnectionState, RecordCondition};
    let root = tempfile::tempdir().unwrap();
    let mut definition =
        FormPrompt::<BTreeMap<String, bool>>::new("fixture.condition", "Connect device")
            .definition();
    definition.actions[0].requires_condition = true;
    definition.condition = Some(RuntimeCondition::Record {
        condition: RecordCondition {
            record_type: Identifier::new("tool.connection.acquired").unwrap(),
            match_fields: BTreeMap::from([(
                "resource_id".into(),
                Value::Text("fixture-device".into()),
            )]),
        },
    });
    let mut owner = prompt_runtime(root.path(), definition).await;
    let handle = owner.handle();
    let id = dokime_runtime::model::identity::new_id::<PromptId>();
    handle
        .request_prompt(PromptRequest {
            prompt_id: id.clone(),
            prompt_type: Identifier::new("fixture.condition").unwrap(),
            values: BTreeMap::new(),
            scope: RecordScope::default(),
        })
        .await
        .unwrap();
    let server = Server::start(
        handle.clone(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap(),
    )
    .await
    .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let view = client
        .unary::<contracts::prompts::GetPrompt>(authenticated(
            contracts::prompts::GetPromptRequest {
                prompt_id: id.to_string(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let definition = view.prompt.unwrap().definition.unwrap();
    assert!(definition.actions[0].requires_condition);
    let Some(contracts::prompts::EvidenceConditionKind::Record(condition)) = definition
        .condition
        .unwrap()
        .nodes
        .into_iter()
        .next()
        .unwrap()
        .kind
    else {
        panic!("record condition lost");
    };
    assert_eq!(condition.record_type, "tool.connection.acquired");
    assert!(matches!(
        condition.match_fields.unwrap().to_core().unwrap(),
        Value::Object(fields) if fields.get("resource_id") == Some(&Value::Text("fixture-device".into()))
    ));
    let answer = contracts::prompts::RespondToPromptRequest {
        prompt_id: id.to_string(),
        action: "submit".into(),
        accepted: true,
        ..Default::default()
    };
    let rejected = client
        .unary::<contracts::prompts::Respond>(authenticated(answer.clone()))
        .await
        .unwrap_err();
    assert_eq!(rejected.code(), Code::FailedPrecondition);
    let envelope = tonic_types::Status::decode(rejected.details()).unwrap();
    assert_eq!(
        <contracts::execution::Rejection as harmos::Protobuf>::decode(
            envelope.details[0].value.as_slice()
        )
        .unwrap()
        .code,
        "prompt_condition_not_satisfied"
    );
    assert!(handle.prompt_response(&id).await.is_none());
    handle
        .publish_product_record(
            ProductConnectionDetails::new("fixture-device", ProductConnectionState::Connected)
                .into_draft("fixture"),
        )
        .await
        .unwrap();
    client
        .unary::<contracts::prompts::Respond>(authenticated(answer))
        .await
        .unwrap();
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn prompt_level_watched() {
    use dokime_runtime::records::RecordSeverity;

    // === Ask A Warning Prompt === //

    let root = tempfile::tempdir().unwrap();
    let definition = FormPrompt::<BTreeMap<String, bool>>::new("fixture.level", "Check the bench")
        .level(RecordSeverity::Warning)
        .definition();
    let mut owner = prompt_runtime(root.path(), definition).await;
    let handle = owner.handle();
    let id = dokime_runtime::model::identity::new_id::<PromptId>();
    handle
        .request_prompt(PromptRequest {
            prompt_id: id.clone(),
            prompt_type: Identifier::new("fixture.level").unwrap(),
            values: BTreeMap::new(),
            scope: RecordScope::default(),
        })
        .await
        .unwrap();
    let server = Server::start(
        handle.clone(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap(),
    )
    .await
    .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();

    // === Read The Prompt And Its Request Record === //

    let view = client
        .unary::<contracts::prompts::GetPrompt>(authenticated(
            contracts::prompts::GetPromptRequest {
                prompt_id: id.to_string(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let mut follow = client
        .download::<contracts::evidence::WatchRecords>(authenticated(
            contracts::evidence::WatchRecordsRequest {
                selection: Some(contracts::evidence::CapturedRecordSelection {
                    record_types: vec!["prompt.request.form".into()],
                    ..Default::default()
                }),
                after: None,
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let mut watched = Vec::new();
    while watched.is_empty() {
        watched = follow.message().await.unwrap().unwrap().records;
    }

    // === Compare === //

    let level = view.prompt.unwrap().definition.unwrap().level;
    assert_eq!(level, contracts::common::Severity::Warn);
    assert_eq!(watched.len(), 1);
    assert_eq!(watched[0].severity, contracts::common::Severity::Warn);
    drop(follow);
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn settled_prompt_refused() {
    // === Ask A Cycle's Question With No Cancel Action === //

    let root = tempfile::tempdir().unwrap();
    let mut definition =
        FormPrompt::<BTreeMap<String, bool>>::new("fixture.settle", "Inspect Charging State")
            .definition();
    definition.actions.retain(|action| action.id != "cancel");
    let mut owner = prompt_runtime(root.path(), definition).await;
    let handle = owner.handle();
    let cycle = handle
        .create_cycle(
            dokime_runtime::transactions::CycleDefinition {
                metadata: support::metadata("cycle.prompt.settle", "Prompt Settle"),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let id = dokime_runtime::model::identity::new_id::<PromptId>();
    handle
        .request_prompt(PromptRequest {
            prompt_id: id.clone(),
            prompt_type: Identifier::new("fixture.settle").unwrap(),
            values: BTreeMap::new(),
            scope: RecordScope::default().with_cycle(cycle.clone()),
        })
        .await
        .unwrap();
    let server = Server::start(
        handle.clone(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap(),
    )
    .await
    .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let pending = |cycle: &TestCycleId| contracts::prompts::ListPendingPromptsRequest {
        cycle_id: Some(cycle.to_string()),
        ..Default::default()
    };
    let waiting = client
        .unary::<contracts::prompts::ListPending>(authenticated(pending(&cycle)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(waiting.prompts.len(), 1);

    // === Stop The Cycle === //

    handle
        .stop_cycle(cycle.clone(), CommandContext::default())
        .await
        .unwrap();

    // === Nothing Waits, And A Late Answer Is A Failed Precondition === //

    let settled = client
        .unary::<contracts::prompts::ListPending>(authenticated(pending(&cycle)))
        .await
        .unwrap()
        .into_inner();
    assert!(settled.prompts.is_empty());
    let late = client
        .unary::<contracts::prompts::Respond>(authenticated(
            contracts::prompts::RespondToPromptRequest {
                prompt_id: id.to_string(),
                action: "submit".into(),
                values: None,
                accepted: true,
                context: None,
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(late.code(), Code::FailedPrecondition);
    assert!(handle.prompt_response(&id).await.is_none());
    assert!(handle.prompt_settled_at(&id).await.is_some());

    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

async fn prompt_runtime(
    root: &std::path::Path,
    definition: dokime_runtime::domains::prompts::PromptDefinition,
) -> Runtime {
    let mut declarations = harmos::sidecar::Registrar::new();
    declarations
        .register(
            dokime_runtime::plugin::registration::PROMPTS_NAMESPACE,
            dokime_runtime::plugin::registration::domain_schema(
                dokime_runtime::plugin::registration::PROMPTS_NAMESPACE,
            ),
            &[definition],
        )
        .unwrap();
    let config =
        crate::support::budgets::runtime_config(root, support::metadata("grpc.prompts", "Questions"))
            .with_declared_plugin("prompt.fixture", "1.0.0", "Questions", declarations)
            .unwrap();
    Runtime::start(config).await.unwrap()
}

/* === Tests === */
