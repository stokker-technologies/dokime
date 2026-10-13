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
use dokime_app::{
    capabilities::prompts::{
        FormPrompt, Prompt, PromptField, PromptFieldKind, PromptRequest, Rule,
    },
    core::{PromptId, RecordScope, Value, ValueType},
    expressions::BooleanExpression,
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

pub(crate) async fn prompt_answers_identity() {
    let root = tempfile::tempdir().unwrap();
    let mut definition = FormPrompt::<BTreeMap<String, i64>>::new("fixture.answer", "Exact answer")
        .field(PromptField {
            metadata: common::metadata("counter", "Counter"),
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
    let ids = [PromptId::new(), PromptId::new()];
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
            .with_bearer_auth(vec![grant(Role::Control)])
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
    server.shutdown().await.unwrap();
    let server = Server::start(
        handle.clone(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Read)])
            .unwrap(),
    )
    .await
    .unwrap();
    let read = WireClient::connect(server.local_endpoint()).await.unwrap();
    read.unary::<contracts::prompts::GetPrompt>(authenticated(
        contracts::prompts::GetPromptRequest {
            prompt_id: ids[0].to_string(),
        },
    ))
    .await
    .unwrap();
    assert_eq!(
        read.unary::<contracts::prompts::Respond>(authenticated(answer(i64::MAX)))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn prompt_condition_commit() {
    use dokime_app::definition::RuntimeCondition;
    use dokime_app::records::{ProductConnectionDetails, ProductConnectionState, RecordCondition};
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
    let id = PromptId::new();
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

async fn prompt_runtime(
    root: &std::path::Path,
    definition: dokime_app::capabilities::prompts::PromptDefinition,
) -> Runtime {
    let mut declarations = harmos::plugin::sidecar::Registrar::new();
    declarations
        .register(
            dokime::plugin::registration::PROMPTS_NAMESPACE,
            dokime::plugin::registration::domain_schema(
                dokime::plugin::registration::PROMPTS_NAMESPACE,
            ),
            &[definition],
        )
        .unwrap();
    let config = RuntimeConfig::new(root, common::metadata("grpc.prompts", "Questions"))
        .with_declared_plugin("prompt.fixture", "1.0.0", "Questions", declarations)
        .unwrap();
    Runtime::start(config).await.unwrap()
}

/* === Tests === */
