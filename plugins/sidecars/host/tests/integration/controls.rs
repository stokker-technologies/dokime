/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Control Stories: Inverse State, Journal Ordering and the Connection Guard Over a Fake Machine

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Mutex;

// Third Party Imports
use prost_types::Timestamp;

// Local Imports
use dokime::templates::Environment;
use dokime::{Error, Result, Visibility};
use plugin_host::adapters::{ADAPTERS, Adapter, AdapterKind};
use plugin_host::controls::{
    ADAPTER_SET, AdapterSet, Caller, Change, Journal, Native, PLAN_SET, PlanSet, RADIO_SET, RESTORE, RadioSet, State, cuts_connection,
};
use plugin_host::domain::controls::{AdapterState, ControlState, Radio, RadioState};
use plugin_host::power::PLAN_GET;

/* === Constants === */

const BACKEND_PID: u32 = 7;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// A Machine, Journal and Caller That Record Every Call in Order, Never Touching the Host.
#[derive(Default)]
struct Fake {
    before: ControlState,
    administrator: bool,
    in_step: bool,
    journal_unavailable: bool,
    failing_write: bool,
    failing_restore: bool,
    connected: Vec<Adapter>,
    calls: Mutex<Vec<String>>,
    written: Mutex<Vec<ControlState>>,
    journal: Mutex<Vec<(String, ControlState)>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fake {
    fn record_call(&self, call: impl Into<String>) {
        self.calls.lock().unwrap().push(call.into());
    }

    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

impl Native for Fake {
    async fn administrator(&self) -> Result<bool> {
        self.record_call("administrator");
        Ok(self.administrator)
    }

    async fn read(&self, _: &Change) -> Result<ControlState> {
        self.record_call("read");
        Ok(self.before.clone())
    }

    async fn apply(&self, state: &ControlState, restoring: bool) -> Result<()> {
        self.record_call(if restoring { "restore" } else { "write" });
        self.written.lock().unwrap().push(state.clone());
        let refused = if restoring { self.failing_restore } else { self.failing_write };
        if refused {
            return Err(Error::Invalid("write refused".into()));
        }
        Ok(())
    }

    async fn connected(&self, pid: u32) -> Result<Vec<Adapter>> {
        self.record_call(format!("connected {pid}"));
        Ok(self.connected.clone())
    }

    fn radio_affects(&self, radio: Radio, adapter: &Adapter) -> Result<bool> {
        Ok(radio == Radio::Wifi && adapter.is(AdapterKind::Wifi))
    }
}

impl Journal for Fake {
    async fn check(&self) -> Result<()> {
        if self.journal_unavailable {
            return Err(Error::Invalid("journal unavailable".into()));
        }
        Ok(())
    }

    async fn record(&self, target: &str, inverse: &ControlState) -> Result<bool> {
        self.record_call(format!("record {target}"));
        if self.journal_unavailable {
            return Err(Error::Invalid("journal unavailable".into()));
        }
        let mut journal = self.journal.lock().unwrap();
        if journal.iter().any(|(recorded, _)| recorded == target) {
            return Ok(false);
        }
        journal.push((target.into(), inverse.clone()));
        Ok(true)
    }

    async fn clear(&self, target: &str) -> Result<()> {
        self.record_call(format!("clear {target}"));
        self.journal.lock().unwrap().retain(|(recorded, _)| recorded != target);
        Ok(())
    }

    async fn outstanding(&self) -> Result<Vec<(String, ControlState)>> {
        Ok(self.journal.lock().unwrap().clone())
    }
}

impl Caller for Fake {
    async fn now(&self) -> Result<Timestamp> {
        Ok(Timestamp { seconds: 1, nanos: 0 })
    }

    async fn environment(&self) -> Result<Environment> {
        self.record_call("environment");
        Ok(Environment {
            backend_pid: BACKEND_PID,
            in_step: self.in_step,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn radios(states: &[bool]) -> ControlState {
    let radios = states
        .iter()
        .enumerate()
        .map(|(index, on)| RadioState {
            id: format!("radio-{index}"),
            kind: Radio::Wifi.as_str().into(),
            on: *on,
        })
        .collect();
    ControlState { radios, ..Default::default() }
}

fn adapter_state(enabled: bool) -> ControlState {
    ControlState {
        adapters: vec![AdapterState { id: "nic".into(), enabled }],
        ..Default::default()
    }
}

fn plan_state(plan: &str) -> ControlState {
    ControlState {
        power_plan: Some(plan.into()),
        ..Default::default()
    }
}

fn adapter(id: &str, kind: AdapterKind) -> Adapter {
    Adapter {
        id: id.into(),
        name: id.into(),
        kind: kind.as_str().into(),
        ..Default::default()
    }
}

fn wifi(state: &str) -> Change {
    let request = RadioSet {
        radio: "wifi".into(),
        state: state.into(),
        allow_disconnect: false,
    };
    Change::radio(request).unwrap()
}

fn disable_nic(allow_disconnect: bool) -> Change {
    Change::adapter(AdapterSet {
        adapter_id: "nic".into(),
        enabled: false,
        allow_disconnect,
    })
}

fn plan(plan_id: &str) -> Change {
    Change::plan(PlanSet { plan_id: plan_id.into() })
}

/* === Tests === */

#[tokio::test]
async fn a_radio_change_records_its_inverse_before_writing() {
    // Arrange a Machine With One Radio On and One Off
    let fake = Fake {
        before: radios(&[true, false]),
        ..Default::default()
    };

    // Switch Wi-Fi Off
    let state = State::default();
    state.execute(wifi("off"), &fake, &fake, &fake).await.unwrap();

    // The Journal Holds the Prior State Before the Write and Keeps It Outstanding
    assert_eq!(fake.calls(), ["read", "environment", "connected 7", "record host.radio.wifi", "write"]);
    assert_eq!(*fake.journal.lock().unwrap(), [("host.radio.wifi".to_owned(), radios(&[true, false]))]);
    assert_eq!(fake.written.lock().unwrap()[0], radios(&[false, false]));
}

#[tokio::test]
async fn an_unavailable_journal_refuses_before_any_write() {
    let fake = Fake {
        before: plan_state("balanced"),
        journal_unavailable: true,
        ..Default::default()
    };

    let state = State::default();
    assert!(state.execute(plan("performance"), &fake, &fake, &fake).await.is_err());

    assert!(fake.calls().is_empty());
}

#[tokio::test]
async fn a_failed_write_restores_at_once_and_clears_its_own_record() {
    let fake = Fake {
        before: radios(&[false]),
        failing_write: true,
        ..Default::default()
    };

    let state = State::default();
    assert!(state.execute(wifi("on"), &fake, &fake, &fake).await.is_err());

    assert_eq!(fake.calls(), ["read", "record host.radio.wifi", "write", "restore", "clear host.radio.wifi"]);
    assert_eq!(fake.written.lock().unwrap()[1], radios(&[false]));
    assert!(fake.journal.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_later_change_keeps_the_earliest_inverse() {
    // The First Change Records the Original Plan
    let fake = Fake {
        before: plan_state("balanced"),
        ..Default::default()
    };
    let state = State::default();
    state.execute(plan("performance"), &fake, &fake, &fake).await.unwrap();

    // A Second Change, Failing, Restores Its Own Prior State Without Clearing the Original
    let fake = Fake {
        before: plan_state("performance"),
        failing_write: true,
        journal: Mutex::new(fake.journal.lock().unwrap().clone()),
        ..Default::default()
    };
    assert!(state.execute(plan("saver"), &fake, &fake, &fake).await.is_err());
    assert_eq!(*fake.journal.lock().unwrap(), [("host.power.plan".to_owned(), plan_state("balanced"))]);
    assert!(!fake.calls().iter().any(|call| call.starts_with("clear")));
}

#[tokio::test]
async fn a_change_already_in_place_touches_neither_journal_nor_machine() {
    let fake = Fake {
        before: radios(&[false]),
        ..Default::default()
    };

    let state = State::default();
    state.execute(wifi("off"), &fake, &fake, &fake).await.unwrap();

    assert_eq!(fake.calls(), ["read"]);
}

#[tokio::test]
async fn disabling_the_backend_adapter_needs_an_override_from_a_step() {
    // A Request Flag Alone Does Not Override the Guard
    let fake = Fake {
        before: adapter_state(true),
        administrator: true,
        connected: vec![adapter("nic", AdapterKind::Ethernet)],
        ..Default::default()
    };
    let state = State::default();
    assert!(state.execute(disable_nic(true), &fake, &fake, &fake).await.is_err());
    assert!(!fake.calls().iter().any(|call| call.starts_with("record")));

    // Inside a Step the Override Skips the Guard
    let fake = Fake {
        before: adapter_state(true),
        administrator: true,
        in_step: true,
        connected: vec![adapter("nic", AdapterKind::Ethernet)],
        ..Default::default()
    };
    state.execute(disable_nic(true), &fake, &fake, &fake).await.unwrap();
    assert_eq!(fake.calls(), ["administrator", "read", "environment", "record host.adapter.nic", "write"]);
    assert_eq!(fake.written.lock().unwrap()[0], adapter_state(false));
}

#[tokio::test]
async fn an_adapter_change_needs_administrator_before_reading() {
    let fake = Fake {
        before: adapter_state(true),
        ..Default::default()
    };

    let state = State::default();
    assert!(state.execute(disable_nic(false), &fake, &fake, &fake).await.is_err());

    assert_eq!(fake.calls(), ["administrator"]);
}

#[tokio::test]
async fn restoration_writes_outstanding_inverses_latest_first() {
    let fake = Fake {
        journal: Mutex::new(vec![("host.radio.wifi".into(), radios(&[true])), ("host.power.plan".into(), plan_state("balanced"))]),
        ..Default::default()
    };

    let state = State::default();
    state.restore(&fake, &fake).await.unwrap();

    assert_eq!(fake.calls(), ["restore", "clear host.power.plan", "restore", "clear host.radio.wifi"]);
    assert_eq!(*fake.written.lock().unwrap(), [plan_state("balanced"), radios(&[true])]);
}

#[tokio::test]
async fn a_failed_restoration_stays_outstanding() {
    let fake = Fake {
        failing_restore: true,
        journal: Mutex::new(vec![("host.power.plan".into(), plan_state("balanced"))]),
        ..Default::default()
    };

    let state = State::default();
    assert!(state.restore(&fake, &fake).await.is_err());

    assert_eq!(*fake.journal.lock().unwrap(), [("host.power.plan".to_owned(), plan_state("balanced"))]);
}

#[test]
fn a_change_refuses_a_read_that_does_not_describe_its_target() {
    // No Radio, or a Radio of Another Kind
    assert!(wifi("off").after(&ControlState::default()).is_err());
    let bluetooth = ControlState {
        radios: vec![RadioState {
            id: "bt".into(),
            kind: Radio::Bluetooth.as_str().into(),
            on: true,
        }],
        ..Default::default()
    };
    assert!(wifi("off").after(&bluetooth).is_err());

    // An Unresolved Adapter and a Host Without Power Plans
    assert!(disable_nic(false).after(&ControlState::default()).is_err());
    assert!(plan("performance").after(&ControlState::default()).is_err());

    // A Plan Change Keeps Every Other Target
    assert_eq!(plan("performance").after(&plan_state("balanced")).unwrap(), plan_state("performance"));
}

#[test]
fn radio_requests_name_a_known_radio_and_power() {
    for radio in Radio::ALL {
        for state in ["on", "off"] {
            let request = RadioSet {
                radio: radio.as_str().into(),
                state: state.into(),
                allow_disconnect: false,
            };
            assert!(Change::radio(request).is_ok());
        }
    }
    let unknown_radio = RadioSet {
        radio: "satellite".into(),
        state: "on".into(),
        allow_disconnect: false,
    };
    assert!(Change::radio(unknown_radio).is_err());
    let unknown_power = RadioSet {
        radio: "wifi".into(),
        state: "dim".into(),
        allow_disconnect: false,
    };
    assert!(Change::radio(unknown_power).is_err());
}

#[test]
fn only_the_targeted_connection_counts_as_cut() {
    let connected = [adapter("nic", AdapterKind::Ethernet), adapter("wlan", AdapterKind::Wifi)];
    let affects = |radio: Radio, adapter: &Adapter| Ok(radio == Radio::Wifi && adapter.is(AdapterKind::Wifi));

    assert!(cuts_connection(&disable_nic(false), &connected, affects).unwrap());
    assert!(cuts_connection(&wifi("off"), &connected, affects).unwrap());
    assert!(!cuts_connection(&disable_nic(false), &connected[1..], affects).unwrap());

    let bluetooth = Change::radio(RadioSet {
        radio: "bluetooth".into(),
        state: "off".into(),
        allow_disconnect: false,
    })
    .unwrap();
    assert!(!cuts_connection(&bluetooth, &connected, affects).unwrap());
}

#[test]
fn reads_and_controls_are_public_while_only_the_host_restores() -> Result<()> {
    let declarations = plugin_host::declarations()?;
    let visibility = |id: &str| declarations.operations.iter().find(|operation| operation.id == id).map(|operation| operation.visibility());

    for id in [ADAPTERS, PLAN_GET, RADIO_SET, ADAPTER_SET, PLAN_SET] {
        assert_eq!(visibility(id), Some(Visibility::Public), "{id}");
    }
    assert_eq!(visibility(RESTORE), Some(Visibility::Protected));
    Ok(())
}
