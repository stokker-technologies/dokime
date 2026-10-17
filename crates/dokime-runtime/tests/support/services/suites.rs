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

// Local Imports
use super::contracts::{minimal_tracking_contract, suite_telemetry_contracts, tracking_record_contracts, tracking_warning_contract};
use super::{metadata, value};
use dokime_runtime::RuntimeHandle;
use dokime_runtime::definition::{
    CriterionDefinition, ExecutionPhaseDefinition, ParameterDefinition, ParameterExpr, SuiteSet, TestCase, TestCaseTemplate, TestSuite,
};
use dokime_runtime::domains::TestSuiteDefinition;
use dokime_runtime::runtime::command::CommandContext;
use dokime_runtime::transactions::CycleDefinition;
use dokime_runtime::{Metadata, ParameterId, TemplateId, TestCycleId, Value, ValueType};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn empty_suite() -> TestSuite {
    fixture_suite(&[])
}

pub fn fixture_suite(results: &[&str]) -> TestSuite {
    TestSuite {
        metadata: metadata("fixture.suite", "Fixture Suite"),
        tags: vec!["fixture".to_owned()],
        parameter_groups: Vec::new(),
        test_case_templates: results.iter().enumerate().map(|(index, result)| case_template(index, result)).collect(),
    }
}

pub fn plt400_suite() -> TestSuite {
    plt400_suite_with_result("passed")
}

pub fn plt400_suite_with_result(result: &str) -> TestSuite {
    TestSuite {
        metadata: metadata("plt400.ract", "PLT400 RACT"),
        tags: vec!["plt400".to_owned()],
        parameter_groups: Vec::new(),
        test_case_templates: vec![case_template_with_identifier("plt400.ract.tracking", "Tracking", result)],
    }
}

pub fn plt400_diagnostics_suite() -> TestSuite {
    TestSuite {
        metadata: metadata("plt400.diagnostics", "PLT400 Diagnostics"),
        tags: vec!["plt400".to_owned()],
        parameter_groups: Vec::new(),
        test_case_templates: vec![case_template_with_identifier("plt400.diagnostics.tracking", "Diagnostics", "passed")],
    }
}

pub async fn register_python_suite(runtime: &RuntimeHandle, suite: TestSuite) {
    register_suite(runtime, python_registration("provider:testdb:plt400.ract", "execute_cycle", suite)).await;
}

pub async fn register_python_suite_pair(runtime: &RuntimeHandle, first: TestSuite, second: TestSuite) {
    runtime
        .set_suites(
            "provider:testdb",
            SuiteSet::new([
                python_registration("provider:testdb:plt400.ract", "execute_cycle", first),
                python_registration("provider:testdb:plt400.diagnostics", "run_diagnostics", second),
            ]),
        )
        .await
        .unwrap();
}

pub async fn register_custom_suite(runtime: &RuntimeHandle, suite: TestSuite) {
    register_suite(
        runtime,
        TestSuiteDefinition {
            suite_id: "provider:testdb:custom.suite".to_owned(),
            tags: Vec::new(),
            workflow: Some("run".to_owned()),
            suite,
            record_contracts: tracking_record_contracts(),
            telemetry_contracts: suite_telemetry_contracts(),
        },
    )
    .await;
}

pub fn case_template(index: usize, result: &str) -> TestCaseTemplate {
    case_template_with_identifier(&format!("fixture.template.{index}"), &format!("Fixture Template {index}"), result)
}

pub fn case_template_with_identifier(identifier: &str, label: &str, result: &str) -> TestCaseTemplate {
    TestCaseTemplate {
        test_case_template_id: dokime_runtime::model::identity::new_id::<TemplateId>(),
        metadata: metadata(identifier, label),
        tags: vec!["fixture".to_owned()],
        parameter_groups: Vec::new(),
        parameters: vec![ParameterDefinition {
            expansion: Default::default(),
            min: None,
            max: None,
            parameter_id: dokime_runtime::model::identity::new_id::<ParameterId>(),
            metadata: metadata("fixture.result", "Fixture Result"),
            group_id: None,
            value_type: ValueType::Text,
            unit: None,
            required: false,
            expression: ParameterExpr::Literal { value: Value::from(result) },
            runtime_writable: false,
            selection: None,
            list: None,
            extensions: Default::default(),
        }],
        files: Vec::new(),
        phases: vec![ExecutionPhaseDefinition {
            repeatable: false,
            step_id: dokime_runtime::model::identity::new_id::<dokime_runtime::StepId>(),
            metadata: metadata("fixture.phase", "Fixture Phase"),
        }],
        criteria: vec![CriterionDefinition {
            criterion_id: dokime_runtime::model::identity::new_id::<dokime_runtime::CriterionId>(),
            metadata: metadata("fixture.criterion", "Fixture Criterion"),
            expected: Some(Value::from(result)),
        }],
        continue_on_failure: false,
        prompt_definitions: Vec::new(),
        subscriptions: Vec::new(),
        document: None,
        attributes: BTreeMap::from([(
            "document".to_owned(),
            value(serde_json::json!({
                "metadata": {
                    "identifier": identifier,
                    "label": label,
                    "description": null
                },
                "sections": [{
                    "metadata": {
                        "identifier": "main",
                        "label": "Main",
                        "description": null
                    },
                    "cards": [{
                        "type": "plot",
                        "data": {
                            "plot": {
                                "metadata": {
                                    "identifier": "fixture.progress",
                                    "label": "Fixture Progress",
                                    "description": null
                                },
                                "plot_type": "time_series",
                                "stream": "progress",
                                "x": "timestamp",
                                "y": ["distance"],
                                "group": "Fixture",
                                "style": "line",
                                "x_scale": "linear",
                                "y_scale": "linear",
                                "live_window_s": 120.0
                            }
                        }
                    }, {
                        "type": "artifact_list"
                    }]
                }]
            })),
        )]),
    }
}

pub fn materialized_case(suite_id: &str, suite: Option<&TestSuite>, identifier: &str, label: &str, result: &str) -> TestCase {
    TestCase {
        suite_id: suite_id.to_owned(),
        test_case_id: dokime_runtime::model::identity::new_id::<dokime_runtime::CaseId>(),
        test_case_template_id: suite.map_or_else(dokime_runtime::model::identity::new_id::<TemplateId>, |suite| {
            suite.test_case_templates[0].test_case_template_id.clone()
        }),
        suite: suite.map(|suite| suite.metadata.clone()),
        metadata: metadata(identifier, label),
        parameters: BTreeMap::from([("fixture.result".to_owned(), Value::from(result))]),
        generation_index: 0,
        label_number: 0,
    }
}

pub fn materialize_cases(suite_id: &str, suite: &TestSuite) -> Vec<TestCase> {
    let options = dokime_runtime::definition::CaseMaterializationOptions {
        force: true,
        max_cases: dokime_runtime::definition::DEFAULT_MATERIALIZED_CASE_CAP,
        registration_id: dokime_runtime::model::identity::new_id(),
        first_label_number: 1,
    };
    suite
        .test_case_templates
        .iter()
        .flat_map(|template| {
            dokime_runtime::definition::materialize_template_cases(suite_id, template, Default::default(), options.clone())
                .unwrap()
                .test_cases
                .into_iter()
                .map(|mut case| {
                    case.suite = Some(suite.metadata.clone());
                    case
                })
        })
        .collect()
}

pub async fn start_cycle(runtime: &RuntimeHandle, suite: TestSuite, continue_on_failure: bool) -> TestCycleId {
    let cases = if suite.test_case_templates.is_empty() {
        Vec::new()
    } else {
        let registration = runtime.test_suites().await.into_iter().find(|entry| entry.suite == suite).expect("fixture suite is installed");
        materialize_cases(&registration.suite_id, &registration.suite)
    };
    start_cases(runtime, metadata("cycle.start", "Start Cycle"), cases, continue_on_failure).await
}

pub async fn start_cases(runtime: &RuntimeHandle, metadata: Metadata, cases: Vec<TestCase>, continue_on_failure: bool) -> TestCycleId {
    let cycle_id = runtime.create_cycle(CycleDefinition { metadata, continue_on_failure }, CommandContext::default()).await.unwrap();
    if !cases.is_empty() {
        runtime.append_cycle_cases(cycle_id.clone(), cases, CommandContext::default()).await.unwrap();
    }
    cycle_id
}

pub fn custom_suite() -> TestSuite {
    TestSuite {
        metadata: metadata("custom.suite", "Custom Suite"),
        tags: Vec::new(),
        parameter_groups: Vec::new(),
        test_case_templates: vec![TestCaseTemplate {
            test_case_template_id: TemplateId::from_seed("evidence.fixture.template"),
            metadata: metadata("custom.template", "Custom Template"),
            tags: Vec::new(),
            parameter_groups: Vec::new(),
            parameters: vec![ParameterDefinition {
                expansion: Default::default(),
                min: None,
                max: None,
                parameter_id: ParameterId::from_seed("evidence.fixture.parameter"),
                metadata: metadata("fixture.result", "Fixture Result"),
                group_id: None,
                value_type: ValueType::Text,
                unit: None,
                required: false,
                expression: ParameterExpr::Literal { value: Value::from("passed") },
                runtime_writable: false,
                selection: None,
                list: None,
                extensions: Default::default(),
            }],
            files: Vec::new(),
            phases: vec![ExecutionPhaseDefinition {
                repeatable: false,
                step_id: dokime_runtime::StepId::from_seed("evidence.fixture.step"),
                metadata: metadata("custom.phase", "Custom Phase"),
            }],
            criteria: vec![CriterionDefinition {
                criterion_id: dokime_runtime::CriterionId::from_seed("evidence.fixture.criterion"),
                metadata: metadata("custom.criterion", "Custom Criterion"),
                expected: Some(Value::from("passed")),
            }],
            continue_on_failure: false,
            prompt_definitions: Vec::new(),
            subscriptions: Vec::new(),
            document: None,
            attributes: BTreeMap::new(),
        }],
    }
}

/* === Private Functions === */
fn python_registration(suite_id: &str, workflow: &str, suite: TestSuite) -> TestSuiteDefinition {
    TestSuiteDefinition {
        suite_id: suite_id.to_owned(),
        tags: vec!["plt400".to_owned()],
        workflow: Some(workflow.to_owned()),
        suite,
        record_contracts: vec![tracking_warning_contract()],
        telemetry_contracts: vec![minimal_tracking_contract()],
    }
}

async fn register_suite(runtime: &RuntimeHandle, definition: TestSuiteDefinition) {
    runtime.set_suites("provider:testdb", SuiteSet::new([definition])).await.unwrap();
}

/* === Tests === */
