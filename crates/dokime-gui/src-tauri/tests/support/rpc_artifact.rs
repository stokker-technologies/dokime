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
use dokime_app::RuntimeHandle;
use dokime_app::artifacts::ArtifactUploadInput;
use dokime_app::core::{Identifier, Metadata, RecordScope, TemplateId};
use dokime_app::definition::{TestCaseTemplate, TestSuite};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Default)]
pub(super) struct Provider(std::sync::Mutex<Option<String>>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl dokime_app::ExecutionFixture for Provider {
    fn run_cycle<'a>(
        &'a self,
        context: dokime_app::ExecutionFixtureContext,
        cycle: &'a dokime_app::execution::CycleView,
        _target: &'a dokime::domains::ExecutionProviderTarget,
    ) -> dokime_app::ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let execution = &cycle.test_executions[0];
            let upload = context
                .begin_artifact_upload(ArtifactUploadInput {
                    scope: RecordScope::default()
                        .with_cycle(cycle.test_cycle_id.clone())
                        .with_test_case(execution.test_case_id.clone())
                        .with_test_execution(execution.test_execution_id.clone()),
                    filename: "wire.bin".into(),
                    content_type: Some("application/octet-stream".into()),
                    size_bytes: 150_000,
                    sha256: None,
                })
                .await
                .unwrap();
            let bytes: Vec<_> = (0..150_000).map(|i| (i % 251) as u8).collect();
            for chunk in bytes.chunks(64 * 1024) {
                upload.send(chunk).await.unwrap();
            }
            *self.0.lock().unwrap() = Some(upload.finish().await.unwrap().artifact_id.to_string());
            use dokime_app::transport::{
                ProviderCycleOutcome, ProviderExecutionOutcome, ProviderStatusOutcome,
            };
            Ok(ProviderCycleOutcome {
                status: "passed".into(),
                dependency_imports: vec![],
                failures: vec![],
                provider: ProviderStatusOutcome {
                    status: "active".into(),
                    domain_registration_count: 1,
                    test_suite_count: 1,
                    test_suite_identifiers: vec!["rpc.artifact".into()],
                    error: None,
                },
                executions: vec![ProviderExecutionOutcome {
                    case_id: execution.test_case_id.to_string(),
                    test_execution_id: execution.test_execution_id.to_string(),
                    template_identifier: "rpc.artifact".into(),
                    label: "Artifact fixture".into(),
                    module: "fixture".into(),
                    status: "passed".into(),
                    phases: vec![],
                    records: vec![],
                    telemetry: vec![],
                    criteria: vec![],
                    failures: vec![],
                }],
            })
        })
    }
}

/* === Public Functions === */
pub(super) async fn seed(runtime: &RuntimeHandle, provider: &Provider) -> String {
    let metadata = || {
        Metadata::new(
            Identifier::new("rpc.artifact").unwrap(),
            "Artifact fixture",
            None,
        )
    };
    let template = TestCaseTemplate {
        test_case_template_id: TemplateId::new(),
        metadata: metadata(),
        tags: vec![],
        parameter_groups: vec![],
        parameters: vec![],
        files: vec![],
        document: None,
        phases: vec![],
        criteria: vec![],
        continue_on_failure: false,
        prompt_definitions: vec![],
        subscriptions: vec![],
        attributes: Default::default(),
    };
    let suite = TestSuite {
        metadata: metadata(),
        tags: vec![],
        parameter_groups: vec![],
        test_case_templates: vec![template],
    };
    runtime
        .set_suites(
            "provider:fixture",
            dokime_app::definition::SuiteSet::new([dokime::domains::TestSuiteDefinition {
                suite_id: "rpc.artifact".into(),
                tags: vec![],
                workflow: Some("execute_cycle".into()),
                suite,
                record_contracts: vec![],
                telemetry_contracts: vec![],
            }]),
        )
        .await
        .unwrap();
    let cycle_id = registered_cycle(runtime, "rpc.artifact", metadata()).await;
    let receipt = runtime.run_cycle_cases(cycle_id, Default::default()).await;
    assert_eq!(
        receipt.status,
        dokime_app::runtime::command::CommandStatus::Accepted
    );
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if let Some(id) = provider.0.lock().unwrap().clone() {
                break id;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap()
}

/// Creates an empty cycle and appends every case of the registered suite.
pub(super) async fn registered_cycle(
    runtime: &RuntimeHandle,
    suite_id: &str,
    metadata: Metadata,
) -> dokime_app::core::TestCycleId {
    let suite = runtime
        .test_suites()
        .await
        .into_iter()
        .find(|entry| entry.suite_id == suite_id)
        .expect("fixture suite is registered")
        .suite;
    let options = dokime::definition::CaseMaterializationOptions {
        force: true,
        ..Default::default()
    };
    let cases = suite
        .test_case_templates
        .iter()
        .flat_map(|template| {
            dokime::definition::materialize_template_cases(
                suite_id,
                template,
                Default::default(),
                options.clone(),
            )
            .unwrap()
            .test_cases
        })
        .map(|mut case| {
            case.suite = Some(suite.metadata.clone());
            case
        })
        .collect();
    let cycle_id = runtime
        .create_cycle(
            dokime_app::transactions::CycleDefinition {
                metadata,
                continue_on_failure: false,
            },
            Default::default(),
        )
        .await
        .unwrap();
    runtime
        .append_cycle_cases(cycle_id.clone(), cases, Default::default())
        .await
        .unwrap();
    cycle_id
}

/* === Private Functions === */

/* === Tests === */
