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
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
// resource records project into instance groups without mutating parameters.

#[tokio::test]
async fn resource_records_absent() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;

    register_custom_suite(&runtime, custom_suite()).await;
    let test_cycle_id = start_cycle(&runtime, custom_suite(), false).await;
    runtime.run_cycle_cases(test_cycle_id.clone(), CommandContext::default()).await;
    let cycle = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Completed).await;
    let execution = cycle.test_executions.first().unwrap();
    let execution_scope = execution.scope.clone();

    let mut updated = RecordPublishRequest::new(identifier("resource.traceability.updated"), RecordSeverity::Info);
    updated.scope = Some(execution_scope.clone());
    updated.details = BTreeMap::from([
        ("resource_id".to_owned(), Value::from("tracking_resource")),
        ("resource_name".to_owned(), Value::from("tracking_resource")),
        ("resource_kind".to_owned(), Value::from("tracking_tool")),
        (
            "traceability".to_owned(),
            Value::Object(BTreeMap::from([
                ("tracking_driver_version".to_owned(), Value::from("2026.5.99")),
                ("serial_number".to_owned(), Value::from("SIM-PLT400-0001")),
                ("production_number".to_owned(), Value::Null),
                ("firmware_version".to_owned(), Value::from("R4.0.462")),
            ])),
        ),
        ("device_info".to_owned(), Value::Object(BTreeMap::from([("device_type".to_owned(), Value::from("Layout Tool"))]))),
    ]);
    let mut relay = RecordPublishRequest::new(identifier("resource.opened"), RecordSeverity::Info);
    relay.scope = Some(execution_scope.clone());
    relay.details = BTreeMap::from([
        ("resource_name".to_owned(), Value::from("relay")),
        ("resource_kind".to_owned(), Value::from("relay")),
        ("traceability".to_owned(), Value::Object(BTreeMap::from([("serial_number".to_owned(), Value::from("HUR-123"))]))),
    ]);

    runtime.publish_record(updated).await.unwrap();
    runtime.publish_record(relay).await.unwrap();
    // The same resource opened twice is two instances; closing one does not
    // renumber the next.
    publish_tooling_resource(&runtime, &execution_scope, "resource.opened", Some("TOOL-1")).await;
    publish_tooling_resource(&runtime, &execution_scope, "resource.closed", None).await;
    publish_tooling_resource(&runtime, &execution_scope, "resource.opened", Some("TOOL-2")).await;

    let evidence = runtime.evidence_snapshot().await.unwrap();
    let cycles = runtime.cycles().await;
    let grouped = |prefix: &str| {
        evidence
            .traceability
            .iter()
            .flat_map(|snapshot| snapshot.entries.iter())
            .filter(|entry| entry.source.as_deref().is_some_and(|source| source.starts_with(prefix)))
            .map(|entry| (entry.metadata.identifier.as_str(), &entry.value))
            .collect::<BTreeMap<_, _>>()
    };

    // The test method reads as the template's own identifier, not its UUID.
    let definition = grouped("traceability.test_definition");
    assert_eq!(
        definition.get("execution.template"),
        Some(&&Value::from(custom_suite().test_case_templates[0].metadata.identifier.as_str()))
    );

    let devices = grouped("traceability.devices.");
    assert_eq!(devices.get("resource.tracking_resource_1.device_info.device_type"), Some(&&Value::from("Layout Tool")));

    let resources = grouped("traceability.resources.");
    assert_eq!(resources.get("resource.tracking_resource_1.traceability.tracking_driver_version"), Some(&&Value::from("2026.5.99")));
    assert_eq!(resources.get("resource.tracking_resource_1.traceability.serial_number"), Some(&&Value::from("SIM-PLT400-0001")));
    assert!(!resources.contains_key("resource.tracking_resource_1.traceability.production_number"));
    assert_eq!(resources.get("resource.relay_1.traceability.serial_number"), Some(&&Value::from("HUR-123")));
    assert_eq!(resources.get("resource.tooling_1.name"), Some(&&Value::from("Tooling Session #1")));
    assert_eq!(resources.get("resource.tooling_1.traceability.serial_number"), Some(&&Value::from("TOOL-1")));
    assert_eq!(resources.get("resource.tooling_2.name"), Some(&&Value::from("Tooling Session #2")));
    assert_eq!(resources.get("resource.tooling_2.traceability.serial_number"), Some(&&Value::from("TOOL-2")));

    // None of it wrote a parameter on the execution.
    for key in ["resource.tracking_resource_1.traceability.serial_number", "resource.relay_1.traceability.serial_number"] {
        assert!(!cycles[0].test_executions[0].output_parameters.contains_key(key));
    }
}

async fn publish_tooling_resource(runtime: &RuntimeHandle, scope: &RecordScope, record_type: &str, serial_number: Option<&str>) {
    let mut request = RecordPublishRequest::new(identifier(record_type), RecordSeverity::Info);
    request.scope = Some(scope.clone());
    request.details = BTreeMap::from([
        ("resource_id".to_owned(), Value::from("tooling")),
        ("resource_name".to_owned(), Value::from("Tooling")),
        ("resource_kind".to_owned(), Value::from("tooling")),
    ]);
    if let Some(serial_number) = serial_number {
        request
            .details
            .insert("traceability".to_owned(), Value::Object(BTreeMap::from([("serial_number".to_owned(), Value::from(serial_number))])));
    }

    runtime.publish_record(request).await.unwrap();
}

/* === Tests === */
