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
use crate::service_support::prelude::*;
use dokime_domain::model;
use {
    dokime_runtime::records::{RecordPublication, RecordSeverity},
    model::records::publication::RecordPublishRequest,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Publisher;

#[dokime_runtime::record(id = "fixture.sidecar.observed", label = "Observed")]
struct Observed {
    value: f64,
}

#[dokime_runtime::catalog]
enum Records {
    Observed(Observed),
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::builtins::lifecycle::Policy for Publisher {
    const INITIALIZATION_POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
        timeout: Duration::from_secs(120),
        attempt_timeout: Duration::from_secs(120),
        cancellation_timeout: std::time::Duration::ZERO,
        retries: harmos::operations::RetryPolicy::Never,
    };
    const SHUTDOWN_POLICY: harmos::operations::OperationPolicy = Self::INITIALIZATION_POLICY;
}

#[dokime_runtime::plugin(id = "fixture_records", requires(), contributions(dokime_runtime::Records<Records>), restart = "never")]
impl dokime_runtime::Plugin for Publisher {
    const CATALOGS: &'static [harmos::Catalog] = &[];
}

impl harmos::Lifecycle for Publisher {
    type Config = dokime_runtime::plugin::Configuration;
    type Error = dokime_runtime::Refusal;
    type Context = dokime_runtime::Supervisor;

    async fn prepare(_: Self::Config, _: &mut harmos::Setup<Self>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

/* === Public Functions === */

pub(super) fn configured(config: dokime_runtime::RuntimeConfig, directory: &std::path::Path) -> dokime_runtime::RuntimeConfig {
    dokime_runtime::testing::plugin::<Publisher>(config, directory).unwrap()
}

/* === Private Functions === */

/* === Tests === */

pub(super) async fn native_publications_order(owner: &mut Runtime) {
    let runtime = owner.handle();
    let caller = owner.fixture_operation_caller("fixture_records").unwrap();
    let mut records = runtime.subscribe_records();
    let mut request = RecordPublishRequest::new(Identifier::new("fixture.sidecar.observed").unwrap(), RecordSeverity::Info);
    request.details.insert("value".into(), dokime_runtime::Value::Number(7.0));
    request.scope = Some(dokime_runtime::RecordScope::default().with_cycle(dokime_runtime::model::identity::new_id::<TestCycleId>()));
    let timestamp = "2026-09-17T10:00:00Z";
    let first = RecordPublication {
        request_json: serde_json::to_string(&request).unwrap(),
        produced_at: timestamp.into(),
    };
    let mut forged = serde_json::to_value(&request).unwrap();
    forged["source"] = "foreign.owner".into();
    let mut undeclared = request.clone();
    undeclared.record_type = Identifier::new("undeclared.record").unwrap();
    let publications = [
        first.clone(),
        RecordPublication {
            request_json: serde_json::to_string(&forged).unwrap(),
            produced_at: timestamp.into(),
        },
        RecordPublication {
            request_json: serde_json::to_string(&undeclared).unwrap(),
            produced_at: timestamp.into(),
        },
        RecordPublication {
            request_json: "garbled".into(),
            produced_at: timestamp.into(),
        },
        RecordPublication {
            request_json: first.request_json,
            produced_at: String::new(),
        },
    ];
    let arrival = dokime_runtime::model::time::now_utc();
    // The host files what the plugin relays under the caller's own identity,
    // and refuses to the plugin, synchronously, what it cannot file: a forged
    // source, an undeclared record type and a payload that is no request.
    for (index, publication) in publications.into_iter().enumerate() {
        let filed = caller.call::<RecordPublication>(publication).await;
        assert_eq!(filed.is_ok(), matches!(index, 0 | 4), "publication {index}: {filed:?}");
    }
    let mut admitted = Vec::new();
    while admitted.len() < 2 {
        let entry = records.next().await.unwrap().unwrap();
        assert_eq!(entry.record_type.as_str(), "fixture.sidecar.observed");
        admitted.push(entry);
    }
    assert_eq!(admitted[0].timestamp.as_str(), timestamp);
    assert!(admitted[1].timestamp >= arrival);
    assert!(admitted[1].sequence > admitted[0].sequence);
    for entry in &admitted {
        assert_eq!(entry.source, "fixture_records");
        assert_eq!(entry.scope, None);
        assert_eq!(entry.details["value"], dokime_runtime::Value::Number(7.0));
    }
    drop(records);
}
