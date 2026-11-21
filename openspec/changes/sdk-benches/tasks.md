# Tasks

## 1. Preconditions

- [ ] 1.1 Confirm the plugin launch handshake change has landed on `dev` and resolve this change's source paths against the start-up and SDK run entries it introduced; verify by reading the landed SDK run entries in all four languages.
- [ ] 1.2 Confirm the plugin contract freeze has not been declared; if it has, raise the conflict with the owner before continuing.

## 2. Reference Plugins

- [ ] 2.1 Give Rust its reference plugin per design.md, seeded from `sdk/rust/examples/thermal.rs` (fixture `dokime-rust-thermal`), with the echo operation, the 128 load streams and the single-stream ceiling stream; verify `mise run build:host-fixtures -- --language rust` builds it and the thermal equivalence stories still pass.
- [ ] 2.2 Give Python the same reference plugin declarations, seeded from `sdk/python/examples/thermal` and `sdk/tests/fixtures/python/thermal`; verify `python_plugin_matches_rust_thermal_evidence` still passes.
- [ ] 2.3 Give C# the same reference plugin declarations, seeded from `sdk/tests/fixtures/csharp/SystemPlugin`; verify `mise run build:host-fixtures -- --language csharp` builds it and `csharp_plugin_matches_rust_thermal_evidence` still passes.
- [ ] 2.4 Give C++ the same reference plugin declarations, seeded from `sdk/tests/fixtures/cpp/thermal`; verify `mise run build:host-fixtures -- --language cpp` builds it and `cpp_plugin_matches_rust_thermal_evidence` still passes.

## 3. Driver and Guard

- [ ] 3.1 Add the `system-sdk` bench target with the catalog guard that compares each plugin's admitted catalog with the Rust reference plugin; verify a deterministic test that a plugin with one extra stream is refused with that stream's identifier and records no measurement.
- [ ] 3.2 Implement the fixed scenario and the round trip, throughput at 128 × 1 kHz, single-stream ceiling, start to ready, idle and loaded memory measures in the existing report row shape, with units in square brackets and `not_measured` rows for unmeasurable stages; verify `mise run bench -- --language rust --layer system --binary system-sdk --once` produces every case name for each available language.
- [ ] 3.3 Route `--binary system-sdk` and the language `--select` variants through `tools/python/src/dokime_tools/task/bench.py` and `test/benchmark.py`, and add the build of the bench target and its fixtures to `build:rust:bench`; verify the task selection with the existing tooling tests.

## 4. Baselines and Report

- [ ] 4.1 Restore and save the bench baseline cache per platform and job variant on `dev` (`.github/actions/setup/action.yml`, `tools/python/src/dokime_tools/ci/native.py`) so hosted runs compare with the previous `dev` run; verify a second hosted `dev` run reports `baseline_value` for every case.
- [ ] 4.2 Add the cross-language section to the Nightly report in `tools/python/src/dokime_tools/release/reports.py`, one table per measure with one column per language and one block per platform and no thresholds; verify with a fixture-driven report test.

## 5. Nightly Jobs

- [ ] 5.1 Add `Bench / SDK / <Language> (<platform>)` for Rust, Python, C# and C++ on `unix` and `windows` to `.github/workflows/nightly.yml`, reusing the `Build / Bench`, `Build / SDK / Python` and `Build / SDK / <Language> / Fixtures` archives, and add them to the `needs` of `Release / Report / Nightly` only; verify the job graph shows `Release / Publish / GitHub` independent of them.
- [ ] 5.2 Add the `Bench / Protocol` Criterion target for evidence, sample, plot and report messages (encode, decode, size) and its `Bench / Protocol (<platform>)` job; verify it appears in `benchmarks.json` and the release notes of a Nightly run.

## 6. Storage Bench After the Cutover

- [ ] 6.1 After the `operation-boundary` storage cutover lands, add `Bench / Storage` for writing an execution, sealing it and querying one million samples through the library's storage operations, and apply the owner's decision on `crates/dokime-runtime/benches/storage/` and its `bench:rust:storage` task; verify the new cases reach `benchmarks.json`.

## 7. Documentation and Final Validation

- [ ] 7.1 Report the documentation impact to the owner: `docs/project/tooling.md` bench tasks and jobs, the cross-language report, and the SDK guides that name the reference plugin.
- [ ] 7.2 After source closure, obtain hosted Nightly results for every `Bench / SDK / <Language>` and `Bench / Protocol` job on both platforms and report the measured rows, `not_measured` stages and regressions without claiming any pass or fail verdict.
