# Screenshots Task Tasks

## 1. Implementation

- [x] 1.1 Add the public `screenshots` task with `--scene` and `--check`, as one linear script on the desktop story harness.
- [x] 1.2 Build the state by one fixed journey on a fresh root under the placeholder identity, with the bench plugin.
- [ ] 1.3 Capture each scene in the light and the dark theme at its viewport and density, and write the manifest.
- [x] 1.4 Replace the command-per-scene module and its checks; delete the old images and hand-capture instructions.
- [ ] 1.5 Run the journey on the virtual clock once the host clock is adopted.
- [x] 1.6 Update the README, docs/tooling.md and the product guide.

## 2. Final Validation

- [ ] 2.1 Capture on a workstation and review every image.

## Capture Blockers

- The Windows journey reaches running captures, but the bench execution fails during
  finalization: `dokime.execution.evidence` exceeds its 30-second operation deadline.
  Backend shutdown subsequently exceeds its 25-second runtime stop deadline. The
  smaller finite journey reproduces this; no incomplete image set is published.
- Task 1.3 remains open: the current bench plugin declares no reference bands, and
  the current Library is a flat template table with a parameter dialog, without
  the specified expanded suite and template step timeline. These product owners
  must resolve the scene prerequisites before the captures can satisfy the table.
- Task 2.1 remains open. Failed-run diagnostics are under `target/screenshots/`;
  regenerated images and their manifest are not committed until the full journey
  and visual review succeed. Task 1.5 remains deferred to the host-clock wave.

## Workflow Follow-Up

- The owner captures and commits the images as a chore; the site switches to the new file names.
