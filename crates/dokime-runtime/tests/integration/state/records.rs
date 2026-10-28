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
use harmos::state::Transaction;
// Local Imports
use dokime_runtime::domains::evidence::RecordPublishRequest;
use dokime_runtime::records::RecordSeverity;
use dokime_runtime::transactions::PublishRecord;
use dokime_runtime::{Identifier, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn plugin_state(declarations: &[(&str, &str)]) -> dokime_runtime::state::RuntimeState {
    let mut state = dokime_runtime::state::RuntimeState::new(dokime_runtime::model::identity::new_id());
    let bundles = declarations
        .iter()
        .map(|(source, record_type)| dokime_runtime::state::BundleCatalog {
            package: (*source).to_owned(),
            path: "fixture.dokime".into(),
            installation: None,
            content_digest: "fixture-digest".to_owned(),
            plugins: vec![dokime_runtime::InstalledPlugin {
                source: (*source).to_owned(),
                enabled: true,
                record_contracts: vec![dokime_runtime::records::RecordContract::declared(*record_type, "Fixture record")],
                ..Default::default()
            }],
        })
        .collect::<Vec<_>>();
    state.reconcile_providers(&bundles).expect("the fixture catalog is admitted");
    state
}

/* === Tests === */
// a plugin may only narrate the records its own catalog declares.
#[test]
fn plugin_may_declares() {
    let state = plugin_state(&[("bench", "unit.connected"), ("gateway", "gateway.clamped")]);
    let publish = |owner: &str, record_type: &str| {
        PublishRecord::sidecar(
            RecordPublishRequest::new(Identifier::new(record_type).unwrap(), RecordSeverity::Info),
            owner.to_owned(),
            Timestamp::new("2026-09-17T10:00:00Z"),
        )
    };

    publish("bench", "unit.connected").check(&state).expect("a plugin narrates what it declared");
    // Somebody else's declaration is not a licence to publish it.
    assert!(publish("bench", "gateway.clamped").check(&state).is_err());
    // Nor is a type nobody declared at all.
    assert!(publish("bench", "bench.invented").check(&state).is_err());
    // Nor is a source the catalog does not know.
    assert!(publish("ghost", "unit.connected").check(&state).is_err());
}

#[test]
fn admission_order_publishers() {
    let mut state = plugin_state(&[("bench", "unit.connected"), ("gateway", "gateway.clamped")]);
    let publish = |owner: &str, record_type: &str| {
        PublishRecord::sidecar(
            RecordPublishRequest::new(Identifier::new(record_type).unwrap(), RecordSeverity::Info),
            owner.to_owned(),
            Timestamp::new("2026-09-17T10:00:00Z"),
        )
    };

    assert_eq!(state.record_count, 0);
    for (index, (owner, kind)) in
        [("gateway", "gateway.clamped"), ("bench", "unit.connected"), ("gateway", "gateway.clamped")].into_iter().enumerate()
    {
        let transaction = publish(owner, kind);
        let (_, members) = harmos::state::Records::collect(&transaction, &state).unwrap();
        assert_eq!(members.len(), 1);
        let fact = members[0].0.as_ref().expect("sidecar record source");
        let envelope: serde_json::Value = serde_json::from_slice(&fact.payload).unwrap();
        let captured: dokime_domain::state::records::CapturedRecord = serde_json::from_value(envelope["payload"].clone()).unwrap();
        assert_eq!(captured.ordinal, index as u64 + 1);
        assert!(captured.record.scope.is_none());
        assert!(captured.scopes.is_empty());
        transaction.apply(&mut state);
        assert_eq!(state.records.through, captured.ordinal);
    }

    // Three admissions, three positions, assigned by the fold and not by any
    // sequence the publishers chose. Interleaving publishers changes nothing:
    // one fold admits them one after another, in the order it received them.
    assert_eq!(state.record_count, 3);
}
