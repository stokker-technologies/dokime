/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Lifecycle Gates And Drain Invocation Counts For Integration Stories.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    sync::{LazyLock, Mutex},
};

// Third Party Imports

// Local Imports
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

static REPORT_GATES: LazyLock<Mutex<BTreeMap<TestExecutionId, ReportGate>>> = LazyLock::new(Default::default);

static REPORT_WAIT_GATES: LazyLock<Mutex<BTreeMap<TestExecutionId, ReportGate>>> = LazyLock::new(Default::default);

static SEAL_GATES: LazyLock<Mutex<BTreeMap<TestExecutionId, ReportGate>>> = LazyLock::new(Default::default);

static CHECKPOINT_GATES: LazyLock<Mutex<BTreeMap<TestExecutionId, ReportGate>>> = LazyLock::new(Default::default);

static REQUEST_GATES: LazyLock<Mutex<BTreeMap<TestExecutionId, ReportGate>>> = LazyLock::new(Default::default);

static MEMBER_CAPTURE_GATES: LazyLock<Mutex<BTreeMap<TestExecutionId, ReportGate>>> = LazyLock::new(Default::default);

static REPORT_JOB_GATES: LazyLock<Mutex<BTreeMap<TestExecutionId, ReportGate>>> = LazyLock::new(Default::default);

static STREAM_DRAIN_GATES: LazyLock<Mutex<BTreeMap<TestExecutionId, ReportGate>>> = LazyLock::new(Default::default);

static CAPTURE_SEAL_GATES: LazyLock<Mutex<BTreeMap<TestExecutionId, ReportGate>>> = LazyLock::new(Default::default);

static COUNTS: LazyLock<Mutex<BTreeMap<TestExecutionId, BTreeMap<&'static str, usize>>>> = LazyLock::new(Default::default);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct ReportGate {
    entered: tokio_util::sync::CancellationToken,
    release: tokio_util::sync::CancellationToken,
    fail: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ReportGate {
    pub async fn entered(&self) {
        self.entered.cancelled().await;
    }
    pub fn release(&self) {
        self.release.cancel();
    }
}

/* === Public Functions === */

pub fn hold_report(execution: &TestExecutionId, fail: bool) -> ReportGate {
    let gate = ReportGate {
        entered: tokio_util::sync::CancellationToken::new(),
        release: tokio_util::sync::CancellationToken::new(),
        fail,
    };
    REPORT_GATES.lock().unwrap().insert(execution.clone(), gate.clone());
    gate
}

pub(crate) async fn before_report(execution: &TestExecutionId) -> crate::runtime::Result<()> {
    let gate = REPORT_GATES.lock().unwrap().remove(execution);
    if let Some(gate) = gate {
        gate.entered.cancel();
        gate.release.cancelled().await;
        if gate.fail {
            return Err(crate::runtime::Error::Operation("injected report failure".into()));
        }
    }
    Ok(())
}

pub fn hold_report_wait(execution: &TestExecutionId) -> ReportGate {
    hold(&REPORT_WAIT_GATES, execution)
}

pub fn hold_execution_seal(execution: &TestExecutionId) -> ReportGate {
    hold(&SEAL_GATES, execution)
}

pub fn hold_execution_checkpoint(execution: &TestExecutionId) -> ReportGate {
    hold(&CHECKPOINT_GATES, execution)
}

pub fn hold_report_request(execution: &TestExecutionId) -> ReportGate {
    hold(&REQUEST_GATES, execution)
}

pub fn hold_member_capture(execution: &TestExecutionId) -> ReportGate {
    hold(&MEMBER_CAPTURE_GATES, execution)
}

pub fn hold_report_job(execution: &TestExecutionId) -> ReportGate {
    hold(&REPORT_JOB_GATES, execution)
}

pub fn hold_stream_drain(execution: &TestExecutionId) -> ReportGate {
    hold(&STREAM_DRAIN_GATES, execution)
}

pub fn hold_capture_seal(execution: &TestExecutionId) -> ReportGate {
    hold(&CAPTURE_SEAL_GATES, execution)
}

pub(crate) async fn before_capture_seal(execution: &TestExecutionId) {
    let gate = CAPTURE_SEAL_GATES.lock().unwrap().get(execution).cloned();
    if let Some(gate) = gate {
        gate.entered.cancel();
        gate.release.cancelled().await;
        CAPTURE_SEAL_GATES.lock().unwrap().remove(execution);
    }
}

pub(crate) async fn before_stream_drain(execution: &TestExecutionId) {
    enter(&STREAM_DRAIN_GATES, execution).await;
}

pub(crate) async fn before_report_job(execution: &TestExecutionId) {
    enter(&REPORT_JOB_GATES, execution).await;
}

pub(crate) async fn before_report_request(execution: &TestExecutionId) {
    enter(&REQUEST_GATES, execution).await;
}

pub(crate) async fn before_member_capture(execution: &TestExecutionId) {
    enter(&MEMBER_CAPTURE_GATES, execution).await;
}

pub(crate) async fn before_report_wait(execution: &TestExecutionId) {
    enter(&REPORT_WAIT_GATES, execution).await;
}

pub(crate) async fn before_seal_adoption(execution: &TestExecutionId) {
    enter(&SEAL_GATES, execution).await;
}

pub(crate) async fn before_execution_checkpoint(execution: &TestExecutionId) {
    enter(&CHECKPOINT_GATES, execution).await;
}

pub(crate) fn record(execution: &TestExecutionId, stage: &'static str) {
    *COUNTS.lock().unwrap().entry(execution.clone()).or_default().entry(stage).or_default() += 1;
}

pub fn drain_counts(execution: &TestExecutionId) -> BTreeMap<&'static str, usize> {
    COUNTS.lock().unwrap().remove(execution).unwrap_or_default()
}

/* === Private Functions === */

fn hold(gates: &Mutex<BTreeMap<TestExecutionId, ReportGate>>, execution: &TestExecutionId) -> ReportGate {
    let gate = ReportGate {
        entered: tokio_util::sync::CancellationToken::new(),
        release: tokio_util::sync::CancellationToken::new(),
        fail: false,
    };
    gates.lock().unwrap().insert(execution.clone(), gate.clone());
    gate
}

async fn enter(gates: &Mutex<BTreeMap<TestExecutionId, ReportGate>>, execution: &TestExecutionId) {
    let gate = gates.lock().unwrap().remove(execution);
    if let Some(gate) = gate {
        gate.entered.cancel();
        gate.release.cancelled().await;
    }
}

/* === Tests === */
