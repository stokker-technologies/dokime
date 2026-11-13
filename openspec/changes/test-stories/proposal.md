# Test Stories

Status: agreed with the owner on 2026-11-10; implementation has not started
and waits for the owner's go. Independent of `grpc-plugins` and
`remote-access-simplification`, which rewrite the tests of the areas they
change under the same convention.

## Why

Several areas carry as many lines of test as of source. Two audits found
little duplicated coverage. The volume comes from repeated setup, driver files that dispatch unrelated
scenarios, workloads sized like benchmarks, and tests that depend on real time.

## What Changes

- **One convention.** A test at integration level and above follows one user
  journey, one story per file. It reads like a test template: top to bottom,
  few or no indentation levels, a short comment naming each stage, and every
  line an action the user takes or a check of what they then see.
- **Shared modules own the mechanics:** backend lifecycle, remote access,
  wire calls, providers and catalogs, events and storage readback. A shared
  module never chooses a scenario or hides an assertion.
- **Driver files that dispatch several scenarios are split** into one story
  per file.
- **Large workloads move to benchmarks.** A correctness test uses the
  smallest input that crosses the boundary it proves.
- **No test depends on real time.** A test runs on the host's virtual clock
  or a paused clock, or waits for an event. Fixture timestamps are fixed
  values, not the wall clock. A bound that only stops a hung test stays as a
  guard.
- **Tests that prove little are removed,** with any useful check moved into a
  story.

`design.md` lists the concrete groups and locations the audits found.

## Impact

- Test trees of `dokime-server`, `dokime-runtime`, the desktop shell, the
  interface and the tools. No production behaviour changes.
- Depends on the host-owned clock from `grpc-plugins` for stories that wait
  inside a plugin.
