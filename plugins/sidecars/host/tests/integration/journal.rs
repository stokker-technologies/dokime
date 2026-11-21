/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Effect Journal Stories

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Mutex;

// Third Party Imports

// Local Imports
use dokime::{Error, Result};
use plugin_host::adapters::Adapter;
use plugin_host::controls::{Change, Journal, Native, State};
use plugin_host::domain::controls::{ControlState, Radio};
use plugin_host::journal::{self, Effects, Saved, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The Host's Durable Plugin State: One Value and Revision That Outlive Every Journal Over It.
#[derive(Default)]
struct Durable {
    saved: Mutex<Option<(Effects, u64)>>,
    conflicts: Mutex<Vec<Effects>>,
}

/// A Machine That Only Records the Inverses Restoration Writes.
#[derive(Default)]
struct Restorer {
    failing: Option<String>,
    restored: Mutex<Vec<String>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store for Durable {
    type Revision = u64;

    async fn load(&self) -> Result<Saved<u64>> {
        let saved = self.saved.lock().unwrap();
        let (value, revision) = saved.clone().unwrap_or_default();
        Ok(Saved { value, revision })
    }

    async fn save(&self, value: Effects, expected: u64) -> Result<bool> {
        let mut saved = self.saved.lock().unwrap();
        if let Some(value) = self.conflicts.lock().unwrap().pop() {
            let next = saved.as_ref().map_or(1, |(_, revision)| revision + 1);
            *saved = Some((value, next));
        }
        let current = saved.as_ref().map_or(0, |(_, revision)| *revision);
        if current != expected {
            return Ok(false);
        }
        *saved = Some((value, current + 1));
        Ok(true)
    }
}

impl Native for Restorer {
    async fn administrator(&self) -> Result<bool> {
        Ok(true)
    }
    async fn read(&self, _: &Change) -> Result<ControlState> {
        Err(Error::Invalid("Restoration Reads Nothing".into()))
    }
    async fn apply(&self, state: &ControlState, restoring: bool) -> Result<()> {
        assert!(restoring);
        let plan = state.power_plan.clone().unwrap_or_default();
        if self.failing.as_deref() == Some(plan.as_str()) {
            return Err(Error::Invalid("Plan Locked".into()));
        }
        self.restored.lock().unwrap().push(plan);
        Ok(())
    }
    async fn connected(&self, _: u32) -> Result<Vec<Adapter>> {
        Ok(Vec::new())
    }
    fn radio_affects(&self, _: Radio, _: &Adapter) -> Result<bool> {
        Ok(false)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn plan(identifier: &str) -> ControlState {
    ControlState {
        power_plan: Some(identifier.into()),
        ..Default::default()
    }
}

async fn outstanding(store: &Durable) -> Vec<(String, Option<String>)> {
    let entries = journal::Journal::new(store).outstanding().await.unwrap();
    entries.into_iter().map(|(target, inverse)| (target, inverse.power_plan)).collect()
}

/* === Tests === */

#[tokio::test]
async fn a_recorded_inverse_outlives_its_journal_until_it_is_cleared() {
    let store = Durable::default();
    assert!(journal::Journal::new(&store).record("host.power.plan", &plan("balanced")).await.unwrap());

    assert_eq!(outstanding(&store).await, [("host.power.plan".to_owned(), Some("balanced".to_owned()))]);
    journal::Journal::new(&store).clear("host.power.plan").await.unwrap();
    assert!(outstanding(&store).await.is_empty());
}

#[tokio::test]
async fn a_second_record_for_a_target_keeps_the_first_inverse() {
    let store = Durable::default();
    let journal = journal::Journal::new(&store);
    assert!(journal.record("host.power.plan", &plan("balanced")).await.unwrap());
    assert!(journal.record("host.radio.wifi", &plan("radio")).await.unwrap());
    assert!(!journal.record("host.power.plan", &plan("performance")).await.unwrap());

    assert_eq!(
        outstanding(&store).await,
        [("host.power.plan".to_owned(), Some("balanced".to_owned())), ("host.radio.wifi".to_owned(), Some("radio".to_owned()))]
    );
}

#[tokio::test]
async fn start_restores_outstanding_inverses_newest_first_and_keeps_a_failed_one() {
    let store = Durable::default();
    for (target, inverse) in [("first", "a"), ("second", "b"), ("third", "c")] {
        journal::Journal::new(&store).record(target, &plan(inverse)).await.unwrap();
    }

    let restorer = Restorer {
        failing: Some("a".into()),
        ..Restorer::default()
    };
    assert!(State::default().restore(&restorer, &journal::Journal::new(&store)).await.is_err());
    assert_eq!(restorer.restored.into_inner().unwrap(), ["c", "b"]);
    assert_eq!(outstanding(&store).await, [("first".to_owned(), Some("a".to_owned()))]);

    let restorer = Restorer::default();
    State::default().restore(&restorer, &journal::Journal::new(&store)).await.unwrap();
    assert_eq!(restorer.restored.into_inner().unwrap(), ["a"]);
    assert!(outstanding(&store).await.is_empty());
}

#[tokio::test]
async fn revision_conflicts_reread_without_losing_other_targets_or_the_first_inverse() {
    let store = Durable::default();
    let concurrent = Durable::default();
    journal::Journal::new(&concurrent).record("other", &plan("original")).await.unwrap();
    let conflicting = concurrent.load().await.unwrap().value;
    store.conflicts.lock().unwrap().push(conflicting);
    assert!(journal::Journal::new(&store).record("mine", &plan("balanced")).await.unwrap());
    assert_eq!(outstanding(&store).await, [("other".into(), Some("original".into())), ("mine".into(), Some("balanced".into()))]);

    let concurrent = Durable::default();
    journal::Journal::new(&concurrent).record("mine", &plan("first")).await.unwrap();
    let conflicting = concurrent.load().await.unwrap().value;
    store.conflicts.lock().unwrap().push(conflicting);
    journal::Journal::new(&store).clear("other").await.unwrap();
    assert_eq!(outstanding(&store).await, [("mine".into(), Some("first".into()))]);
}

#[tokio::test]
async fn a_conflicting_first_record_keeps_the_concurrent_inverse() {
    let store = Durable::default();
    let concurrent = Durable::default();
    journal::Journal::new(&concurrent).record("same", &plan("original")).await.unwrap();
    let conflicting = concurrent.load().await.unwrap().value;
    store.conflicts.lock().unwrap().push(conflicting);
    assert!(!journal::Journal::new(&store).record("same", &plan("later")).await.unwrap());
    assert_eq!(outstanding(&store).await, [("same".into(), Some("original".into()))]);
}
