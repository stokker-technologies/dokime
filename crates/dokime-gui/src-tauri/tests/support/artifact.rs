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
use dokime_runtime::RuntimeHandle;
use dokime_runtime::artifacts::ArtifactUploadInput;
use dokime_runtime::definition::{TestCaseTemplate, TestSuite};
use dokime_runtime::{Identifier, Metadata, RecordScope, TemplateId};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// Runs One Cycle That Uploads An Artifact And Publishes Its Identifier.
pub struct Provider(tokio::sync::watch::Sender<Option<String>>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Default for Provider {
    fn default() -> Self {
        Self(tokio::sync::watch::Sender::new(None))
    }
}

impl dokime_runtime::ExecutionFixture for Provider {
    fn run_cycle<'a>(
        &'a self, context: dokime_runtime::ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a dokime_runtime::domains::ExecutionProviderTarget,
    ) -> dokime_runtime::ExecutionFixtureFuture<'a> {
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
            let artifact_id = upload.finish().await.unwrap().artifact_id.to_string();
            self.0.send_replace(Some(artifact_id));
            use dokime_runtime::transport::{ProviderCycleOutcome, ProviderExecutionOutcome, ProviderStatusOutcome};
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
pub async fn seed(runtime: &RuntimeHandle, provider: &Provider) -> String {
    let metadata = || Metadata::new(Identifier::new("rpc.artifact").unwrap(), "Artifact fixture", None);
    let template = TestCaseTemplate {
        test_case_template_id: dokime_runtime::model::identity::new_id::<TemplateId>(),
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
    let suite = TestSuite { metadata: metadata(), tags: vec![], parameter_groups: vec![], test_case_templates: vec![template] };
    runtime
        .set_suites(
            "provider:fixture",
            dokime_runtime::definition::SuiteSet::new([dokime_runtime::domains::TestSuiteDefinition {
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
    assert_eq!(receipt.status, dokime_runtime::runtime::command::CommandStatus::Accepted);
    // The Provider Publishes The Artifact Once Its Upload Finished
    let mut published = provider.0.subscribe();
    published.wait_for(Option::is_some).await.expect("the provider outlives its cycle").clone().unwrap()
}

/// Creates an empty cycle and appends every case of the registered suite.
pub async fn registered_cycle(runtime: &RuntimeHandle, suite_id: &str, metadata: Metadata) -> dokime_runtime::TestCycleId {
    let suite =
        runtime.test_suites().await.into_iter().find(|entry| entry.suite_id == suite_id).expect("fixture suite is registered").suite;
    let options = dokime_runtime::definition::CaseMaterializationOptions {
        force: true,
        max_cases: dokime_runtime::definition::DEFAULT_MATERIALIZED_CASE_CAP,
        registration_id: dokime_runtime::model::identity::new_id(),
        first_label_number: 1,
    };
    let cases = suite
        .test_case_templates
        .iter()
        .flat_map(|template| {
            dokime_runtime::definition::materialize_template_cases(suite_id, template, Default::default(), options.clone())
                .unwrap()
                .test_cases
        })
        .map(|mut case| {
            case.suite = Some(suite.metadata.clone());
            case
        })
        .collect();
    let cycle_id = runtime
        .create_cycle(dokime_runtime::transactions::CycleDefinition { metadata, continue_on_failure: false }, Default::default())
        .await
        .unwrap();
    runtime.append_cycle_cases(cycle_id.clone(), cases, Default::default()).await.unwrap();
    cycle_id
}

/* === Private Functions === */

/* === Tests === */
