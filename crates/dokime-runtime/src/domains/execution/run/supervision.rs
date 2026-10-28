/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Decide Which Wakes Concern One Execution's Supervisor, So Global Commits Cost No Storage Queries.

use std::time::Duration;

use crate::RuntimeState;
use dokime_domain::model::execution::result::CyclePhase;
use dokime_domain::model::identity::{CommandId, TestCycleId};

// An Open Command Is Re-Polled At This Pace When The Run Reports No Evidence For It.
pub(super) const COMMAND_POLL: Duration = Duration::from_millis(200);

/// The Part Of A Committed State That One Execution's Supervisor Acts On.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Concern {
    cycle_active: bool,
    stopped: bool,
}

/// Owns Which Wakes Reach One Execution's Supervisor And Whether A Command Needs Reconciliation.
#[derive(Debug)]
pub(super) struct Supervision {
    concern: Concern,
    command_open: bool,
}

/// Names The Execution Whose Concern Is Read From Committed States.
pub(super) struct Subject<'a> {
    pub cycle: &'a TestCycleId,
    pub request: Option<&'a CommandId>,
}

/// What Woke A Supervisor: A Global Commit, Or A Change Signalled On Its Own Session.
#[derive(Clone, Copy, Debug)]
pub(super) enum Wake {
    Commit(Concern),
    Session { command_admitted: bool },
    Poll,
}

impl Concern {
    pub(super) fn of(state: &RuntimeState, subject: &Subject<'_>) -> Self {
        let cycle = state.cycles.get(subject.cycle);
        let cycle_active = cycle.is_some_and(|cycle| cycle.phase == CyclePhase::Active);
        let request = subject.request.and_then(|request| state.run_requests.get(request));
        let stopped = request.is_some_and(|request| request.summary.stop_requested);
        Self { cycle_active, stopped }
    }

    /// Whether This Concern Ends The Run: Its Cycle Left The Active Phase Or Its Request Was Stopped.
    pub(super) fn halts(self) -> bool {
        !self.cycle_active || self.stopped
    }
}

impl Supervision {
    /// A Run May Resume With A Durable Pending Command, So The First Reconciliation Queries Storage.
    pub(super) fn new(concern: Concern) -> Self {
        Self { concern, command_open: true }
    }

    pub(super) fn concern(&self) -> Concern {
        self.concern
    }

    /// Whether A Committed State Changed What This Execution Acts On; Unrelated Commits Are Filtered Here.
    pub(super) fn concerns(&self, concern: Concern) -> bool {
        concern != self.concern
    }

    /// Takes A Wake And Answers Whether The Supervisor Acts On It.
    pub(super) fn wake(&mut self, wake: Wake) -> bool {
        match wake {
            Wake::Commit(concern) => {
                let concerned = self.concerns(concern);
                self.concern = concern;
                concerned
            }
            Wake::Session { command_admitted } => {
                // A Command Admitted For This Execution Opens Reconciliation Until Its Receipt Settles.
                self.command_open |= command_admitted;
                true
            }
            Wake::Poll => true,
        }
    }

    /// Whether A Pending Command Still Needs Receipt Reconciliation.
    pub(super) fn command_open(&self) -> bool {
        self.command_open
    }

    pub(super) fn reconciled(&mut self, open: bool) {
        self.command_open = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::execution::result::TestCycle;
    use dokime_domain::model::identity::{Identifier, Metadata, SessionId};
    use futures_util::FutureExt as _;
    use std::sync::Arc;

    const QUIET: Concern = Concern { cycle_active: true, stopped: false };

    // Counts Reconciliations Reached By The Supervisor's Wake Gate.
    fn deliver(supervision: &mut Supervision, wake: Wake, settles: bool) -> u64 {
        if !supervision.wake(wake) || !supervision.command_open() {
            return 0;
        }
        supervision.reconciled(!settles);
        1
    }

    // Each Execution Sees Its Own Commands: Admission, One Open Progress Page, Then Settlement.
    fn own_command(supervision: &mut Supervision) -> u64 {
        let admitted = deliver(supervision, Wake::Session { command_admitted: true }, false);
        let progressed = deliver(supervision, Wake::Session { command_admitted: false }, true);
        admitted + progressed + deliver(supervision, Wake::Poll, false)
    }

    #[test]
    fn global_commits_cost_only_own_command_reconciliations() {
        const EXECUTIONS: usize = 16;
        const COMMITS: usize = 10_000;
        const OWN_COMMANDS: usize = 3;
        const SPACING: usize = COMMITS / OWN_COMMANDS;
        let mut supervisors: Vec<Supervision> = (0..EXECUTIONS).map(|_| Supervision::new(QUIET)).collect();
        let mut reconciliations = vec![0u64; EXECUTIONS];
        for (index, supervision) in supervisors.iter_mut().enumerate() {
            reconciliations[index] += deliver(supervision, Wake::Poll, true);
        }
        for commit in 0..COMMITS {
            for (index, supervision) in supervisors.iter_mut().enumerate() {
                reconciliations[index] += deliver(supervision, Wake::Commit(QUIET), false);
                let own = commit % SPACING == index && commit / SPACING < OWN_COMMANDS;
                if !own {
                    continue;
                }
                reconciliations[index] += own_command(supervision);
            }
        }
        // One Resume Reconciliation Plus Two Per Own Command; The Global Commit Count Never Appears.
        let expected = 1 + 2 * OWN_COMMANDS as u64;
        assert!(reconciliations.iter().all(|&count| count == expected), "{reconciliations:?}");
    }

    #[test]
    fn stop_and_cycle_close_concern_the_execution() {
        let mut supervision = Supervision::new(QUIET);
        assert!(!supervision.wake(Wake::Commit(QUIET)));
        let stopped = Concern { stopped: true, ..QUIET };
        assert!(supervision.wake(Wake::Commit(stopped)) && supervision.concern().halts());
        let closed = Concern { cycle_active: false, ..QUIET };
        assert!(supervision.wake(Wake::Commit(closed)) && supervision.concern().halts());
        assert!(!QUIET.halts());
    }

    #[test]
    fn committed_payload_wakes_only_the_affected_supervisor() {
        const EXECUTIONS: usize = 16;
        const COMMITS: usize = 1_024;
        let mut state = RuntimeState::new(SessionId::from_seed("supervision"));
        let cycles = (0..EXECUTIONS).map(|index| TestCycleId::from_seed(format!("cycle-{index}"))).collect::<Vec<_>>();
        let at = crate::model::time::now_utc();
        for id in &cycles {
            let cycle = TestCycle {
                saved: None,
                test_cycle_id: id.clone(),
                session_id: None,
                metadata: Metadata::new(Identifier::new("supervision.fixture").unwrap(), "Supervision", None),
                lifecycle: Default::default(),
                phase: CyclePhase::Active,
                created_at: at.clone(),
                started_at: None,
                updated_at: at.clone(),
                execution_ids: Vec::new(),
                records: Default::default(),
                continue_automatically: false,
                result: None,
            };
            state.cycles.insert(id.clone(), cycle);
        }

        let snapshot = crate::queries::RuntimeStateSnapshot::new(Arc::new(state), 0);
        let (sender, _) = tokio::sync::watch::channel(snapshot);
        let mut receivers = cycles.iter().map(|_| sender.subscribe()).collect::<Vec<_>>();
        let mut supervisors = cycles.iter().map(|_| Supervision::new(QUIET)).collect::<Vec<_>>();
        let mut wakes = [0; EXECUTIONS];
        for revision in 1..=COMMITS {
            let owner = (revision - 1) % EXECUTIONS;
            let mut state = sender.borrow().state().clone();
            let cycle = state.cycles.get_mut(&cycles[owner]).unwrap();
            cycle.phase = if cycle.phase == CyclePhase::Active { CyclePhase::Sealing } else { CyclePhase::Active };
            sender.send_replace(crate::queries::RuntimeStateSnapshot::new(Arc::new(state), revision as u64));
            for index in 0..EXECUTIONS {
                let subject = Subject { cycle: &cycles[index], request: None };
                let concerned = receivers[index].wait_for(|snapshot| supervisors[index].concerns(Concern::of(snapshot.state(), &subject)));
                let observed = concerned.now_or_never();
                assert_eq!(observed.is_some(), index == owner);
                if let Some(snapshot) = observed {
                    let snapshot = snapshot.unwrap();
                    let concern = Concern::of(snapshot.state(), &subject);
                    supervisors[index].wake(Wake::Commit(concern));
                    wakes[index] += 1;
                }
            }
        }
        assert_eq!(wakes, [COMMITS / EXECUTIONS; EXECUTIONS]);
    }
}
