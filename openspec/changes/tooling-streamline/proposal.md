# Tooling Streamline

Status: direction agreed with the owner on 2026-11-10; the task table is a
proposal until the CI audit is in. Implementation has not started and waits
for the owner's go. Independent of `grpc-plugins`, which removes the things
several tasks serve.

## Why

`mise.toml` defines 77 tasks, and `tools/python` holds 77 files and about
12,650 lines behind them. Ten tasks have no caller. Preparation is spread
over six tasks, two of which do the same generation. Incremental compilation
is switched off in about 25 places. A storage preflight that deletes compiler
output runs in front of every build. The CI workflows are about 10,000 lines
that carry their own logic beside the tasks.

## What Changes

- **A small set of entry points that need no knowledge of the languages in
  the project.** Language and layer are arguments.

  | Task | Purpose |
  | --- | --- |
  | `prepare` | Install the locked development dependencies |
  | `generate` | Generate clients from the checked-in contracts; `--check` reports drift |
  | `format` | Format every language; `--check` |
  | `lint` | Lint every language |
  | `check` | Format check, lint, repository checks and generated drift |
  | `build` | Build the shipping product |
  | `test` | Run tests; `--language` and `--layer` select |
  | `bench` | Run benchmarks; `--layer` selects |
  | `package` | Assemble archives and installers; the organization pack is applied here |
  | `release:prepare`, `release:tag`, `release:rehearse`, `release:publish`, `release:verify` | The release lifecycle |
  | `dev` | Start the desktop with an owned backend |

- **Incremental compilation is disabled by one variable set in CI** and
  nowhere else. Local builds use it.
- **The storage preflight is removed.** Free disk space is watched by the
  session that runs the builds, not by project tooling. The wrapper code that
  exists only for the preflight goes with it.
- **Preparation shrinks.** `prepare:bindings` and `prepare:generated` become
  `generate`, which no longer compiles a backend to export contracts once
  they are checked in. `prepare:organization` and `prepare:report` become
  steps of `build` and `package`; only the built-in default appearance is
  prepared for an ordinary build.
- **Unused and superseded tasks are deleted:** `test:rust:extended` and
  `build:rust:compatibility`, which nothing calls; the wheel, stub and native
  fixture tasks that go with the PyO3 SDK; `release:notices`, `release:sbom`
  and `release:publish:rust` as separate tasks; `housekeeping:stats`.
- **`tools/python` loses its duplicates and dead modules:** two copies of the
  policy helpers, two of the screenshot code, the unused TOML formatter and
  its source helper, compiler-message parsing in two places, and a task
  dependency graph kept in three places.
- **CI workflows call tasks.** A job checks out, sets up mise, restores the
  cache, runs one task and uploads its results. The detail follows the CI
  audit.

## Open Points

1. The CI shape, from the audit in progress.

Settled with the owner:

- Of the three compatibility configurations, only the WebAssembly check of
  the domain crate is kept, as one CI job. The all-features and
  no-default-features checks are dropped with the tasks that ran them.
- The compiler cache stays for local builds. With incremental compilation on
  it still serves dependencies and fresh checkouts.
- Duplicates and dead modules that came from the shared templates are fixed
  there as well, so new projects do not start with them.

## Impact

- `mise.toml`, `tools/python`, `.github/workflows`, `CONTRIBUTING.md` and the
  tooling guide. No product behaviour changes.
