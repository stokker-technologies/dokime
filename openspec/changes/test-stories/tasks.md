# Test Stories Tasks

## 1. Contract

- [ ] 1.1 Recheck the locations in `design.md` against the current tree, measure the slowest tests again and lock the inventory per crate.

## 2. Shared Modules

- [ ] 2.1 Add the backend lifecycle, providers and catalogs, and events and storage readback modules for the server and runtime tests and move existing tests onto them.
- [ ] 2.2 Merge the server and desktop wire support and move the desktop connection fixtures into shared support.

## 3. Stories

- [ ] 3.1 Split the driver files listed in `design.md` into one story per file.
- [ ] 3.2 Shorten the slowest runtime stories by sharing one startup per story.

## 4. Workloads and Time

- [ ] 4.1 Move the large transfer, archive, admission, sampling and durability workloads to benchmarks and keep the smallest boundary-crossing case as the test.
- [ ] 4.2 Replace every real-time dependency listed in `design.md` with the virtual or paused clock or an awaited event, keeping hang guards as guards.
- [ ] 4.3 Replace wall-clock fixture timestamps with fixed values.

## 5. Removal

- [ ] 5.1 Remove the tests listed in `design.md`, moving any useful check into a story.

## 6. Final Validation

- [ ] 6.1 Run the affected unit and integration lanes on Linux and Windows and record test counts, lines and lane times before and after.

## Workflow Follow-Up

- Tests of the plugin, execution and remote access areas are rewritten under this convention in `grpc-plugins` and `remote-access-simplification`.
- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
