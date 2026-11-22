# Tooling Streamline

Status: entry-point dispatch, project-owned tooling and workflow matrices are implemented. Necessary hidden leaves remain; superseded
aliases and unused wrappers are removed. Independent of `grpc-plugins`, which removes the things several tasks serve.

## Why

Task dispatch, preparation and CI previously duplicated dependency and execution policy. Keep each policy with one owner.

## What Changes

- **A small set of entry points that need no knowledge of the languages in the project.** Language and layer are arguments.

  | Task | Arguments and Purpose |
  | --- | --- |
  | `prepare` | `[--refresh-lock]`; Synchronize Locked Python and Frontend Dependencies |
  | `generate` | `[--check]`; Generate Clients from the Current Compiled Contract Export |
  | `format` | `[--check] [--language rust\|python\|typescript\|toml]`; Format Selected Languages |
  | `lint` | `[--language rust\|python\|typescript]`; Lint Selected Languages |
  | `check` | `[--language rust\|python\|typescript\|toml] [--repository\|--bindings\|--supply\|--wasm] [--profile dev\|release]`; Format, Lint, Repository Checks and Generated Drift; Special Selections Run Only Their Check |
  | `build` | `[--profile dev\|release] [--artifact product\|tests\|bench\|fixtures] [--package NAME [--check]] [--features FEATURES] [--language rust\|python\|typescript] [--binary NAME]`; Build Products, Archives or One Cargo Package; Fixtures Selects Native Python Inputs |
  | `test` | `[--language rust\|python\|typescript] [--layer unit\|integration\|acceptance\|system] [--package NAME --target lib\|TEST_TARGET [--exact NAME]] [--features FEATURES] [--coverage] [--profile dev\|release] [--project sdk/python\|tools/python] [--select EXPRESSION] [--artifacts PATH] [--wheel [--bindings]] [--starter]`; Run Existing Lanes or One Cargo Target; Focused Cargo Tests Use Dev Context |
  | `bench` | `[--layer unit\|integration\|system] [--language rust\|python] [--binary NAME] [--select FILTER] [--refresh-baseline]`; Run Built Benchmarks |
  | `package` | `[--language rust\|python] [--profile dev\|release] [--target TARGET]`; Package Built Artifacts and Prepare Organization Resources |
  | `release:prepare` | Prepare the Release Branch |
  | `release:tag` | Tag the Release and Fast-Forward Main |
  | `release:rehearse` | `[--profile dev\|release] [--target TARGET] [--stage-only]`; Rehearse Release Packages |
  | `release:publish` | `[--dry] [--stage assets\|python\|finalize]`; Publish Release Stages or Run Publication Dry Runs |
  | `release:verify` | Verify Published Release Assets and Installers |
  | `dev` | Start the Desktop with an Owned Backend |

- **Incremental compilation is disabled by one variable set in CI** and nowhere else. Local builds use it.
- **The storage preflight is removed.** Free disk space is watched by the session that runs the builds, not by project tooling. The wrapper
  code that exists only for the preflight goes with it.
- **Preparation shrinks.** `prepare:bindings` and `prepare:generated` become `generate`, which no longer compiles a backend to export
  contracts once they are checked in. `prepare:organization` and `prepare:report` become steps of `build` and `package`; only the built-in
  default appearance is prepared for an ordinary build.
- **Unused and superseded tasks are deleted:** `test:rust:extended` and `build:rust:compatibility`, which nothing calls; the wheel, stub and
  native fixture tasks that go with the PyO3 SDK; `release:notices`, `release:sbom` and `release:publish:rust` as separate tasks;
  `housekeeping:stats`.
- **`tools/python` loses its duplicates and dead modules:** two copies of the policy helpers, two of the screenshot code, the unused TOML
  formatter and its source helper, compiler-message parsing in two places, and a task dependency graph kept in three places.
- **CI workflows call tasks,** as the CI Shape section below describes.

## CI Shape
- **One job template per kind of work, as a matrix.** Checks run over languages, tests over languages and layers, builds and packages over
  platforms. A platform is a matrix value, not a second copy of the steps.
- **A job is: check out, set up mise, restore caches, download the build it consumes, run one task, upload results.** The download is the
  one addition to the plain shape; without it every consumer would compile again.
- **Tasks own what the workflows and their scripts do today:** archiving and extracting builds, the summaries, the Nightly freshness check
  and report, the release review and green-commit check, signing order, and the mirror push. The workflow keeps event routing, secrets,
  permissions, caches, artifacts, concurrency and attestation.
- **Duplicates go:** the two identical archive and extract scripts, the cache-summary steps that usually print nothing, and the Windows disk
  counters, which are a machine diagnostic.
- **Jobs for removed things go with them:** the native wheel, stub and fixture jobs, replaced by one proof of the pure Python package.

```yaml check: strategy: matrix: language: [rust, python, typescript] runs-on: ubuntu-latest steps: - uses: actions/checkout@v4 - uses:
./.github/actions/setup - run: mise run check -- --language ${{ matrix.language }} - uses: actions/upload-artifact@v4 if: always() with:
name: check-${{ matrix.language }} path: target/results ```

## Settled

- Repository automation remains project-owned. The generator-adoption owner decision supersedes copied SDK tooling: `harmos-tools` is
  consumed from the same immutable revision as Cargo's carrier dependencies. Shared tooling owns descriptor analysis, four SDK renderers and
  C# wire projection; product code supplies explicit configuration and task entry points. Other project-owned automation copies remain
  harmonized until their owners decide otherwise.
- The owner decision on 2026-11-16 restores impact planning through one hidden `ci:impact` task. Pure `classify(paths)` and `plan(classes,
  event)` functions own job selection and matrices; workflows route from those outputs. Markdown, docs and OpenSpec changes select only
  documentation proof. Mixed changes union selections, and consumers include all required artifact producers. Unknown or incomplete
  comparisons run every event-eligible job. Required accepts success or explicit planned skips and rejects planner failure, missing results,
  cancellation, failures and unexpected skips. Dev has only quick checks and product builds; Nightly and release validation remain complete.
- The composite actions stay in each repository, reduced to the setup action. The duplication between products is accepted, since shared
  tooling no longer renders CI.
- Of the three compatibility configurations, only the WebAssembly check of the domain crate is kept, as one CI job. The all-features and
  no-default-features checks are dropped with the tasks that ran them.
- The compiler cache stays for local builds. With incremental compilation on it still serves dependencies and fresh checkouts.
- Compiler-cache wrappers, release scripts and CI helpers are project-owned copies. Fix them in each project; Hub templates no longer own or
  render them. No sibling checkout or Hub executable is a tooling dependency.

## Remaining Integration
The entry-point interface in the table is the target. Current `mise.toml` still contains necessary hidden language/layer and recurring
`ci:*` operations. The native Python/PyO3 package and fixture jobs are removed; current wheel/stub checks verify the pure package. Keep
necessary hidden leaves without restoring superseded aliases. Focused Cargo selection accepts `--features`. CI setup provisions Python
before native bootstrap and guards Windows APIs by platform. The installable `uv` pin is 0.13.7; no temporary environment override is part
of the contract. Linux product-build and all-platform entry-point proof remain open. A machine Hub check rejecting this task model is an
upstream tooling obligation, not permission to restore the old model.

## Pinned Protocol Inputs and Bootstrap

The workspace `harmos` dependency revision is the only library pin. Dev and nightly generation export `proto` from that git revision with
Buf into ignored `target/protocol`, alongside freshly staged product inputs. No library proto source is tracked by the product. Rust, C#,
C++, Python and Buf consume that resolved inventory. Check mode compares generated output without rewriting it. Pin projection has one owner
and precedes lock/tool-environment refresh and locked generator execution. SDK installation reuses the profile-aware wheel installer, which
also owns example-distribution preparation.

Release resolution from the library's published Buf Schema Registry label is follow-up work: the owner must provision its organization and
token, and the library release flow must publish the label. This delivery implements git inputs.

## Impact
- `mise.toml`, `tools/python`, `.github/workflows`, `docs/CONTRIBUTING.md` and the tooling guide. No product behaviour changes.