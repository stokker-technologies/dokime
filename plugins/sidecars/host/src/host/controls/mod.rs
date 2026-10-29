/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "runtime")]
mod runtime;
#[cfg(all(feature = "runtime", unix))]
mod unix;
#[cfg(all(feature = "runtime", target_os = "windows"))]
mod windows;

/* === Re-Exports === */

#[cfg(feature = "runtime")]
pub use runtime::{State, guard_connection, require_administrator};
#[cfg(all(feature = "runtime", unix))]
pub use unix::{administrator, connected_adapters, read_radio};
#[cfg(all(feature = "runtime", target_os = "windows"))]
pub use windows::{administrator, connected_adapters, read_radio};

/* === Imports === */

use crate::domain::controls::ControlState;
use dokime::harmos::{
    operations::{OperationPolicy, RetryPolicy},
    state::{Invertible, Transaction},
};
use std::time::Duration;

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(30),
    attempt_timeout: Duration::from_secs(30),
    retries: RetryPolicy::Never,
};

// An Inverse May First Settle An Already Owned Forward Attempt.
pub const RESTORE_POLICY: OperationPolicy = OperationPolicy {
    timeout: POLICY.timeout.saturating_mul(2),
    attempt_timeout: POLICY.attempt_timeout.saturating_mul(2),
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::operation(id = "host.network.radio.set", response = dokime::operations::effects::EffectResult, policy = POLICY, visibility = Public,
    resources = [State, crate::Workers])]
#[derive(Clone, PartialEq)]
pub struct RadioSet {
    pub radio: String,
    pub state: String,
    pub allow_disconnect: bool,
}

#[dokime::operation(id = "host.network.adapter.set", response = dokime::operations::effects::EffectResult, policy = POLICY, visibility = Public,
    resources = [State, crate::Workers])]
#[derive(Clone, PartialEq)]
pub struct AdapterSet {
    pub adapter_id: String,
    pub enabled: bool,
    pub allow_disconnect: bool,
}

// Only The Backend Root Calls An Inverse; Other Plugins Cannot Invoke A Host Operation.
#[dokime::operation(id = "host.controls.restore", response = dokime::operations::effects::EffectResult, policy = RESTORE_POLICY, visibility = Host,
    resources = [State, crate::Workers])]
#[derive(Clone, PartialEq)]
pub struct Restore {
    pub state: ControlState,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Transaction<ControlState> for RadioSet {
    type Error = String;
    fn check(&self, state: &ControlState) -> Result<(), String> {
        if !matches!(self.radio.as_str(), "wifi" | "bluetooth" | "cellular") || !matches!(self.state.as_str(), "on" | "off") {
            return Err("radio must be wifi, bluetooth or cellular; state must be on or off".into());
        }
        if state.radios.is_empty() || state.radios.iter().any(|radio| radio.kind != self.radio) {
            return Err("the selected radio is unavailable".into());
        }
        Ok(())
    }
    fn apply(&self, state: &mut ControlState) {
        for radio in &mut state.radios {
            radio.on = self.state == "on";
        }
    }
}
impl Invertible<ControlState> for RadioSet {
    type Inverse = Restore;
    fn inverse(&self, before: &ControlState) -> Restore {
        Restore { state: before.clone() }
    }
}

impl Transaction<ControlState> for AdapterSet {
    type Error = String;
    fn check(&self, state: &ControlState) -> Result<(), String> {
        if state.adapters.len() != 1 || state.adapters[0].id != self.adapter_id {
            return Err("adapter identity was not resolved".into());
        }
        Ok(())
    }
    fn apply(&self, state: &mut ControlState) {
        state.adapters[0].enabled = self.enabled;
    }
}
impl Invertible<ControlState> for AdapterSet {
    type Inverse = Restore;
    fn inverse(&self, before: &ControlState) -> Restore {
        Restore { state: before.clone() }
    }
}

impl Transaction<ControlState> for Restore {
    type Error = String;
    fn check(&self, _: &ControlState) -> Result<(), String> {
        Ok(())
    }
    fn apply(&self, state: &mut ControlState) {
        *state = self.state.clone();
    }
}
impl Invertible<ControlState> for Restore {
    type Inverse = Restore;
    fn inverse(&self, before: &ControlState) -> Restore {
        Restore { state: before.clone() }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::controls::{AdapterState, RadioState};
    #[test]
    fn native_change_inverses() {
        let radio = ControlState {
            radios: vec![
                RadioState {
                    id: "r1".into(),
                    kind: "wifi".into(),
                    on: true,
                },
                RadioState {
                    id: "r2".into(),
                    kind: "wifi".into(),
                    on: false,
                },
            ],
            ..Default::default()
        };
        let set = RadioSet {
            radio: "wifi".into(),
            state: "off".into(),
            allow_disconnect: false,
        };
        set.check(&radio).unwrap();
        let inverse = set.inverse(&radio);
        let mut changed = radio.clone();
        set.apply(&mut changed);
        assert!(changed.radios.iter().all(|radio| !radio.on));
        inverse.apply(&mut changed);
        assert_eq!(changed, radio);
        let adapter = ControlState {
            adapters: vec![AdapterState { id: "nic".into(), enabled: true }],
            ..Default::default()
        };
        let set = AdapterSet {
            adapter_id: "nic".into(),
            enabled: false,
            allow_disconnect: false,
        };
        set.check(&adapter).unwrap();
        let inverse = set.inverse(&adapter);
        let mut changed = adapter.clone();
        set.apply(&mut changed);
        assert!(!changed.adapters[0].enabled);
        inverse.apply(&mut changed);
        assert_eq!(changed, adapter);
    }
}
