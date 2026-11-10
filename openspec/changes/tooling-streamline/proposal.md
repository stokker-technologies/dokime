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
- **CI workflows call tasks,** as the CI Shape section below describes.

## CI Shape

The workflows are 10,216 lines today (CI 4,521, Nightly 4,689, Release 925).
Almost none of that is logic: checkout, setup, cache and artifact steps
written out per job and per operating system are 73 percent of it, and
conditions another 2,758 lines. The target is about 1,100 to 1,550 lines.

- **One job template per kind of work, as a matrix.** Checks run over
  languages, tests over languages and layers, builds and packages over
  platforms. A platform is a matrix value, not a second copy of the steps.
- **A job is: check out, set up mise, restore caches, download the build it
  consumes, run one task, upload results.** The download is the one addition
  to the plain shape; without it every consumer would compile again.
- **Tasks own what the workflows and their scripts do today:** archiving and
  extracting builds, the summaries, the Nightly freshness check and report,
  the release review and green-commit check, signing order, and the mirror
  push. The workflow keeps event routing, secrets, permissions, caches,
  artifacts, concurrency and attestation.
- **Duplicates go:** the two identical archive and extract scripts, the
  cache-summary steps that usually print nothing, and the Windows disk
  counters, which are a machine diagnostic.
- **Jobs for removed things go with them:** the native wheel, stub and
  fixture jobs, replaced by one proof of the pure Python package.

```yaml
check:
  strategy:
    matrix:
      language: [rust, python, typescript]
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: ./.github/actions/setup
    - run: mise run check -- --language ${{ matrix.language }}
    - uses: actions/upload-artifact@v4
      if: always()
      with:
        name: check-${{ matrix.language }}
        path: target/results
```

## Settled

- Tooling is duplicated between Dokime and Harmos on purpose: each project is
  an independent module with its own copy. The copies are kept harmonized,
  with the same task names, arguments, module layout and behaviour, and a
  fix made in one is made in the other.
- Impact planning is removed. Every pull request runs every job, and the
  dependency graph kept in the workflow, a script and `tools/python` is
  deleted.
- The composite actions stay in each repository, reduced to the setup
  action. The duplication between products is accepted, since shared
  tooling no longer renders CI.
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
