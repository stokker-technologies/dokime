/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[allow(dead_code)]
#[path = "../../../../src/commands/execution/recovery.rs"]
mod implementation;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use dokime::domain::model::identity::{TestCycleId, TestExecutionId};
use implementation::Args;

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
fn conflicts_are_refused_before_connection_or_mutation() {
    let directory = tempfile::tempdir().unwrap();
    let execution = TestExecutionId::from_uuid("00000000-0000-0000-0000-000000000001").unwrap();
    let cycle = "00000000-0000-0000-0000-000000000002";
    let args = Args::owned_destination(cycle.into(), directory.path().to_owned());
    let prepared = args.prepare(&execution).unwrap().unwrap();
    std::fs::write(&prepared.execution_destinations()[0].1, b"previous execution").unwrap();
    assert!(args.prepare(&execution).is_err());
    assert_eq!(std::fs::read(&prepared.execution_destinations()[0].1).unwrap(), b"previous execution");

    std::fs::remove_file(&prepared.execution_destinations()[0].1).unwrap();
    std::fs::write(prepared.cycle_destination(), b"previous cycle").unwrap();
    assert!(args.prepare(&execution).is_err());
    assert_eq!(std::fs::read(prepared.cycle_destination()).unwrap(), b"previous cycle");
    assert!(!prepared.execution_destinations()[0].1.exists());
}

#[test]
fn append_recovery_checks_every_requested_destination_and_cycle_owner() {
    let directory = tempfile::tempdir().unwrap();
    let cycle = TestCycleId::from_seed("owned append");
    let first = TestExecutionId::from_seed("first append");
    let second = TestExecutionId::from_seed("second append");
    let args = Args::owned_destination(cycle.to_string(), directory.path().to_owned());
    let executions = [first.clone(), second];
    let plan = args.prepare_append(&cycle, &executions).unwrap().unwrap();
    assert_eq!(plan.execution_destinations().len(), 2);
    assert!(args.prepare_append(&TestCycleId::from_seed("foreign"), &executions).is_err());
    assert!(args.prepare_append(&cycle, &[first.clone(), first]).is_err());

    std::fs::write(&plan.execution_destinations()[1].1, b"previous second execution").unwrap();
    assert!(args.prepare_append(&cycle, &executions).is_err());
    assert_eq!(std::fs::read(&plan.execution_destinations()[1].1).unwrap(), b"previous second execution");
    assert!(!plan.execution_destinations()[0].1.exists());
    assert!(!plan.cycle_destination().exists());
}
