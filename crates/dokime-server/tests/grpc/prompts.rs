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
use dokime_protocol::v1::prompts_client::PromptsClient;

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
    let mut client = PromptsClient::connect(server.local_endpoint())
        .await
        .unwrap();
    assert_eq!(
        client
            .list_definitions(v1::ListPromptDefinitionsRequest::default())
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let definitions = client
        .list_definitions(authenticated(v1::ListPromptDefinitionsRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(definitions.definitions.len(), 1);
    let definition = client
        .get_definition(authenticated(v1::GetPromptDefinitionRequest {
            prompt_type: "fixture.answer".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        definition.fields[0].default_value.as_ref().unwrap().kind,
        Some(v1::value::Kind::IntegerValue(i64::MAX))
    );
    assert!(matches!(
        definition.rules[0].predicate.as_ref().unwrap().kind,
        Some(v1::answer_predicate::Kind::Binary(_))
    ));
    let first = client
        .list_pending(authenticated(v1::ListPendingPromptsRequest {
            limit: 1,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.prompts.len(), 1);
    let after = first.next_after_prompt_id.clone().unwrap();
    let second = client
        .list_pending(authenticated(v1::ListPendingPromptsRequest {
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
        .list_pending(authenticated(v1::ListPendingPromptsRequest {
            after_prompt_id: after.clone(),
            cursor: first.cursor.clone(),
            cycle_id: Some(uuid::Uuid::now_v7().to_string()),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert_eq!(different.code(), Code::FailedPrecondition);
    let answer = |value| v1::RespondToPromptRequest {
        prompt_id: ids[0].to_string(),
        action: "submit".into(),
        values: Some(v1::ValueObject {
            fields: BTreeMap::from([(
                "counter".into(),
                v1::Value {
                    kind: Some(v1::value::Kind::IntegerValue(value)),
                },
            )]),
        }),
        accepted: true,
        context: None,
    };
    let bad = client
        .respond(authenticated(answer(i64::MAX - 1)))
        .await
        .unwrap_err();
    assert_eq!(bad.code(), Code::InvalidArgument);
    let envelope = tonic_types::Status::decode(bad.details()).unwrap();
    assert_eq!(
        envelope.details[0].type_url,
        "type.googleapis.com/dokime.v1.Rejection"
    );
    assert!(
        !v1::Rejection::decode(envelope.details[0].value.as_slice())
            .unwrap()
            .code
            .is_empty()
    );
    assert!(handle.prompt_response(&ids[0]).await.is_none());
    let receipt = client
        .respond(authenticated(answer(i64::MAX)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(receipt.prompt_id, ids[0].to_string());
    assert!(receipt.committed_revision > first.cursor.as_ref().unwrap().position);
    assert!(receipt.accepted_at.is_some());
    let prompt = client
        .get_prompt(authenticated(v1::GetPromptRequest {
            prompt_id: ids[0].to_string(),
        }))
        .await
        .unwrap()
        .into_inner();
    let response = prompt.response.unwrap();
    assert_eq!(response.responder, "fixture");
    assert_eq!(
        response.values.unwrap().fields["counter"].kind,
        Some(v1::value::Kind::IntegerValue(i64::MAX))
    );
    assert!(
        client
            .respond(authenticated(answer(i64::MAX)))
            .await
            .is_err()
    );
    assert_eq!(
        client
            .list_pending(authenticated(v1::ListPendingPromptsRequest {
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
            .list_pending(authenticated(v1::ListPendingPromptsRequest::default()))
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
    let mut read = PromptsClient::connect(server.local_endpoint())
        .await
        .unwrap();
    read.get_prompt(authenticated(v1::GetPromptRequest {
        prompt_id: ids[0].to_string(),
    }))
    .await
    .unwrap();
    assert_eq!(
        read.respond(authenticated(answer(i64::MAX)))
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
    let mut client = PromptsClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let view = client
        .get_prompt(authenticated(v1::GetPromptRequest {
            prompt_id: id.to_string(),
        }))
        .await
        .unwrap()
        .into_inner();
    let definition = view.prompt.unwrap().definition.unwrap();
    assert!(definition.actions[0].requires_condition);
    let Some(v1::evidence_condition::Kind::Record(condition)) = definition.condition.unwrap().kind
    else {
        panic!("record condition lost");
    };
    assert_eq!(condition.record_type, "tool.connection.acquired");
    assert_eq!(
        condition.match_fields.unwrap().fields["resource_id"].kind,
        Some(v1::value::Kind::TextValue("fixture-device".into()))
    );
    let answer = v1::RespondToPromptRequest {
        prompt_id: id.to_string(),
        action: "submit".into(),
        accepted: true,
        ..Default::default()
    };
    let rejected = client
        .respond(authenticated(answer.clone()))
        .await
        .unwrap_err();
    assert_eq!(rejected.code(), Code::FailedPrecondition);
    let envelope = tonic_types::Status::decode(rejected.details()).unwrap();
    assert_eq!(
        v1::Rejection::decode(envelope.details[0].value.as_slice())
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
    client.respond(authenticated(answer)).await.unwrap();
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

async fn prompt_runtime(
    root: &std::path::Path,
    definition: dokime_app::capabilities::prompts::PromptDefinition,
) -> Runtime {
    let mut declarations = harmos::sidecar::Registrar::new();
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
