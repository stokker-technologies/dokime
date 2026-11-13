/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Round-Trip Frozen Origins and Refuse Mismatched Plugin Calls

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use prost_reflect::ReflectMessage;

// Local Imports
use dokime_protocol::v1::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn frozen_plugin_case_survives_transport_and_rejects_another_case() {
    // Materialize a Case and Freeze Its Provider Identity.
    let test_case = TestCase {
        steps: vec![Step {
            metadata: Some(Metadata {
                identifier: "measure".into(),
                ..Default::default()
            }),
            operation: "thermal.measure".into(),
            repeatable: false,
        }],
        streams: vec!["thermal.temperature".into()],
        ..Default::default()
    };
    let run = PluginRun {
        test_case: Some(test_case.clone()),
        plugin: "thermal".into(),
        template: "thermal.cycle".into(),
        digest: vec![7; 32],
        started_at: Some(prost_types::Timestamp { seconds: 1_700_000_000, nanos: 42 }),
    };
    let origin = Some(Origin {
        kind: Some(origin::Kind::PluginRun(run.clone())),
    });

    // Initialize, Step, Evaluate and Clean Up Through the Same Frozen Origin.
    let initialize = InitializeRequest {
        execution: "execution".into(),
        test_case: Some(test_case.clone()),
        origin: origin.clone(),
    };
    let initialize = InitializeRequest::decode(initialize.encode_to_vec().as_slice()).unwrap();
    assert_eq!(initialize.validate_origin().unwrap(), &run);
    let step = StepRequest {
        execution: "execution".into(),
        step: "measure".into(),
        test_case: Some(test_case.clone()),
        origin: origin.clone(),
    };
    let step = StepRequest::decode(step.encode_to_vec().as_slice()).unwrap();
    assert_eq!(step.validate_origin().unwrap(), &run);
    let criterion = CriterionRequest {
        execution: "execution".into(),
        criterion: "limit".into(),
        test_case: Some(test_case.clone()),
        origin: origin.clone(),
    };
    let criterion = CriterionRequest::decode(criterion.encode_to_vec().as_slice()).unwrap();
    assert_eq!(criterion.validate_origin().unwrap(), &run);
    let cleanup = CleanupRequest {
        execution: "execution".into(),
        termination: Termination::Interrupted as i32,
        test_case: Some(test_case.clone()),
        origin,
    };
    let cleanup = CleanupRequest::decode(cleanup.encode_to_vec().as_slice()).unwrap();
    assert_eq!(cleanup.validate_origin().unwrap(), &run);

    // Decode Using the Same Authoritative Descriptor as Host Registration.
    let dynamic = prost_reflect::DynamicMessage::decode(step.descriptor(), step.encode_to_vec().as_slice()).unwrap();
    assert_eq!(dynamic.transcode_to::<StepRequest>().unwrap(), step);

    // Refuse a Changed Case and Incomplete Identity Before Calling the Plugin.
    let changed = TestCase {
        streams: vec!["other".into()],
        ..test_case
    };
    assert!(
        InitializeRequest {
            test_case: Some(changed.clone()),
            ..initialize
        }
        .validate_origin()
        .is_err()
    );
    assert!(
        StepRequest {
            test_case: Some(changed.clone()),
            ..step.clone()
        }
        .validate_origin()
        .is_err()
    );
    assert!(
        CriterionRequest {
            test_case: Some(changed.clone()),
            ..criterion
        }
        .validate_origin()
        .is_err()
    );
    assert!(CleanupRequest { test_case: Some(changed), ..cleanup }.validate_origin().is_err());
    assert!(StepRequest { origin: None, ..step.clone() }.validate_origin().is_err());
    assert!(StepRequest { test_case: None, ..step.clone() }.validate_origin().is_err());
    assert!(StepRequest { execution: String::new(), ..step }.validate_origin().is_err());
    assert!(PluginRun { started_at: None, ..run.clone() }.validate().is_err());
    assert!(PluginRun { digest: Vec::new(), ..run }.validate().is_err());
}

#[test]
fn import_provenance_round_trips_without_a_plugin_and_cannot_drive_one() {
    // Freeze Script-Supplied Provenance Without Registering a Plugin.
    let imported = Import {
        metadata: Some(Metadata {
            identifier: "run42".into(),
            label: "Run 42".into(),
            ..Default::default()
        }),
        sources: vec![ImportSource {
            name: "readings.csv".into(),
            sha256: vec![9; 32],
            attached: true,
        }],
        importer: "Measurement Script".into(),
        sdk_version: "0.1.5".into(),
        processing: "Calibrated source readings".into(),
        imported_at: Some(prost_types::Timestamp { seconds: 1_700_000_001, nanos: 1 }),
        key: "run42".into(),
        predecessor: String::new(),
    };
    let origin = Origin {
        kind: Some(origin::Kind::Import(imported)),
    };
    let decoded = Origin::decode(origin.encode_to_vec().as_slice()).unwrap();
    assert_eq!(decoded, origin);

    // Refuse Plugin Calls Even If a Caller Supplies an Empty Case.
    let request = StepRequest {
        execution: "imported-execution".into(),
        step: "measure".into(),
        test_case: Some(TestCase::default()),
        origin: Some(decoded),
    };
    assert!(request.validate_origin().is_err());
    assert!(
        StepRequest {
            origin: Some(Origin::default()),
            ..request
        }
        .validate_origin()
        .is_err()
    );
}
