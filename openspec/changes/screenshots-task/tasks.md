# Screenshots Task Tasks

## 1. Implementation

- [x] 1.1 Add the public `screenshots` task with `--scene` and `--check`, as one linear script on the desktop story harness.
- [x] 1.2 Build the state by one fixed journey on a fresh root under the placeholder identity, with the bench plugin.
- [ ] 1.3 Capture each scene in the light and the dark theme at its viewport and density, and write the manifest.
- [x] 1.4 Replace the command-per-scene module and its checks; delete the old images and hand-capture instructions.
- [ ] 1.5 Run the journey on the virtual clock once the host clock is adopted.
- [x] 1.6 Update the README, docs/tooling.md and the product guide.
- [x] 1.7 Repair histogram geometry and prove all bench plot kinds in both themes through the GUI canvas interface test.
- [x] 1.8 Diagnose evidence admission and shutdown timeouts without changing operation deadlines; repair the journey or report the precise product wait.
- [x] 1.9 Declare bench reference bands and expose registered procedure steps in the expanded Library selection.

## 2. Final Validation

- [ ] 2.1 Capture on a workstation and review every image.

## Capture Blockers

The finite 1,000-sample bench execution fails `dokime.execution.evidence` at its
30-second operation deadline during final record settlement. In
`crates/dokime-runtime/src/domains/execution/run/session.rs`, `settle_records(true)`
logs publications flushed and subscriptions closed, but never reaches replaying
samples. The intervening awaits are `deliver_records()`, `fence_executions()` and
`settle_leaseholder_deliveries()`. Further instrumentation in the runtime owner is
needed to distinguish them; no deadline or workload is reduced to pass capture.
Backend shutdown then exceeds the 25-second runtime stop budget and exits with 5.
Diagnostics are retained under `target/screenshots/`.

Selected Touch capture also reproduces slow stop settlement: the host reports the
5-second stop budget expired, while `Execution.StopExecution` remains unanswered
long enough to exhaust the WebDriver's 30-second script guard. The images are
captured before requesting stop; the task still reports this teardown failure.

Finished plots, the saved report, checkpoint history and the mixed finished/active
Cycles scene require that successful execution and remain uncaptured. Tasks 1.3
and 2.1 remain open; `--check` must fail until every scene exists. Ready scenes are
retained with their manifest entries and reviewed independently.

The shared GUI fix also requires regeneration of the embedded runtime report at
`crates/dokime-runtime/dist-report/report.html`, outside the assigned GUI ownership.
The TypeScript generated-resource check reports that stale file until its owner
refreshes it. Task 1.5 remains deferred to adoption of the host virtual clock.

## Workflow Follow-Up

- The owner captures and commits the images as a chore; the site switches to the new file names.
