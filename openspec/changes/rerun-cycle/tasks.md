## Implementation

- [x] 1.1 Add the `RerunCycle` operation, `RerunCycleReply` and `RerunCaseNotice` wire messages and `CycleSummary.rerun_of` (dokime-operations, dokime-domain).
- [x] 1.2 Add `RuntimeHandle::rerun_cycle`, resolving cases against installed suites and recording the `rerun_of` metadata attribute (dokime-runtime).
- [x] 1.3 Serve `RerunCycle` beside `CreateCycle`, map `rerun_of` into summaries and cover the route in the authorization list (dokime-server).
- [x] 1.4 Expose rerun in the Rust client, CLI `cycle rerun` and Python `Client.rerun_cycle` with stubs.
- [x] 1.5 Regenerate the TypeScript client; add Rerun Cycle to the cycle menu, the skipped detail in the Control card and "Rerun of" lineage (dokime-gui).
- [x] 1.6 Add deterministic runtime integration tests (sealed rerun, missing case, no surviving case) and GUI unit tests (menu availability, skipped detail).
- [x] 1.7 Document Rerun Cycle in docs/product-guide.md and docs/gui.md; review README.

## Final Validation and Delivery

- [x] 2.1 Run the owner-specified focused all-targets check, focused Rust tests, `test:typescript:unit`, `format` and `stkr project check`.
- [x] 2.2 Obtain CLEAN from the owner-specified scan and prepare the candidate for authorized PR delivery and specification retirement.

## Workflow Follow-Up

Copy Cycle overlaps Rerun Cycle in the cycle menu; retiring it is an owner decision outside this change.
