/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use dokime::RuntimeHandle;
use dokime::artifacts::ArtifactUploadInput;
use dokime::definition::{TestCaseTemplate, TestSuite};
use dokime::{Identifier, Metadata, RecordScope, TemplateId};

pub struct Provider(tokio::sync::watch::Sender<Option<String>>);

impl Default for Provider {
    fn default() -> Self {
        Self(tokio::sync::watch::Sender::new(None))
    }
}

impl dokime::ExecutionFixture for Provider {
    fn run_cycle<'a>(
        &'a self, context: dokime::ExecutionFixtureContext, cycle: &'a dokime::domain::model::execution::result::CycleView,
        _target: &'a dokime::domains::ExecutionProviderTarget,
    ) -> dokime::ExecutionFixtureFuture<'a> {
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
                    size_bytes: 1_200_000,
                    sha256: None,
                })
                .await
                .unwrap();
            let bytes: Vec<_> = (0..1_200_000).map(|i| (i % 251) as u8).collect();
            for chunk in bytes.chunks(64 * 1024) {
                upload.send(chunk.to_vec()).await.unwrap();
            }
            let artifact_id = upload.finish().await.unwrap().artifact_id.to_string();
            self.0.send_replace(Some(artifact_id));
            use dokime::transport::{ProviderCycleOutcome, ProviderExecutionOutcome, ProviderStatusOutcome};
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

pub async fn seed(runtime: &RuntimeHandle, provider: &Provider) -> String {
    let metadata = || Metadata::new(Identifier::new("rpc.artifact").unwrap(), "Artifact fixture", None);
    let template = TestCaseTemplate {
        test_case_template_id: dokime::model::identity::new_id::<TemplateId>(),
        metadata: metadata(),
        tags: vec![],
        parameter_groups: vec![],
        parameters: vec![],
        files: vec![],
        document: None,
        phases: vec![],
        criteria: vec![],
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
            dokime::definition::SuiteSet::new([dokime::domains::TestSuiteDefinition {
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
    let (_cycle_id, executions) = registered_cycle(runtime, "rpc.artifact", metadata()).await;
    let receipt = runtime
        .start_execution(executions[0].clone(), dokime::domain::model::execution::ExecutionMode::Automatic, Default::default())
        .await;
    assert_eq!(receipt.status, dokime::runtime::command::CommandStatus::Accepted);
    // The Provider Publishes The Artifact Once Its Upload Finished
    let mut published = provider.0.subscribe();
    published.wait_for(Option::is_some).await.expect("the provider outlives its cycle").clone().unwrap()
}

pub async fn registered_cycle(
    runtime: &RuntimeHandle, suite_id: &str, metadata: Metadata,
) -> (dokime::TestCycleId, Vec<dokime::TestExecutionId>) {
    let suite =
        runtime.test_suites().await.into_iter().find(|entry| entry.suite_id == suite_id).expect("fixture suite is registered").suite;
    let options = dokime::definition::CaseMaterializationOptions {
        force: true,
        max_cases: usize::try_from(1_000).unwrap(),
        registration_id: dokime::model::identity::new_id(),
        first_label_number: 1,
    };
    let cases = suite
        .test_case_templates
        .iter()
        .flat_map(|template| {
            dokime::definition::materialize_template_cases(suite_id, template, Default::default(), options.clone()).unwrap().test_cases
        })
        .map(|mut case| {
            case.suite = Some(suite.metadata.clone());
            case
        })
        .collect::<Vec<_>>();
    let cycle_id = runtime
        .create_cycle(dokime::transactions::CycleDefinition { metadata, continue_automatically: false }, Default::default())
        .await
        .unwrap();
    let bindings = cases
        .iter()
        .map(|case| {
            let execution = dokime::model::identity::new_id::<dokime::TestExecutionId>();
            (execution, case.test_case_id.clone())
        })
        .collect::<Vec<_>>();
    runtime.register_cases(options.registration_id, cases).await.unwrap();
    runtime.append_executions(cycle_id.clone(), bindings.clone()).await.unwrap();
    (cycle_id, bindings.into_iter().map(|(execution, _case)| execution).collect())
}
