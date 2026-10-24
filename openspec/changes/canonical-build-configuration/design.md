# Canonical Build Design

## Parallel Producer Decision

Product and Tests now use separate parallel provider jobs, superseding single-job consolidation. Independent feature sets are permitted, but CI remains canonical-only and extended configurations remain local opt-in. Tests Build must take no longer than Product Build on the same candidate/platform/profile; measured breaches require investigation and repair of redundant compilation, feature sprawl or testing structure, not silent coverage removal or raised timeouts. Preserve actual artifact dependencies and report relocated work in the full critical path. Repository owner must reconcile obsolete single-job clauses before closure. The preserved baseline pipeline completed with provider timeout on Windows; it was not cancelled by an agent.

## Superseding User Decision

All provider CI contexts cover only the canonical shipping configuration. Fault-injection, simulation and other noncanonical feature/target compatibility coverage are local-only, explicitly requested extended validation. No multi-archive extension or provider compatibility matrix is required. Preserve shipping behavior and all test sources. Keep canonical-capable tests in CI rather than excluding whole packages for a few instrumented cases. Repository-owned build:rust:compatibility is a local compiler-proof helper; test:rust:extended is the explicit optional local behavioral suite and may arrange its own needed builds. Ordinary installers and CI must never invoke extended tasks implicitly. Document exact coverage moved to extended. The baseline pipeline completed without cancellation or a superseding push by the agent.

## Locked Decisions

Clippy runs before compilation with matching feature, target and profile settings and child-scoped warning denial. Product and test compilation may use separate Cargo/nextest invocations sharing the same Cargo directory; test harnesses and incompatible units remain separate. Separate invocations do not promise a single rustc invocation. Installer tasks request Product only. Test and Package consume exported artifacts without compilation.

Mise owns canonical Product and Tests configurations per platform/profile. Avoid a new generic framework or duplicating feature lists in CI. Keep meaningful fault-injection tests isolated where present; never silently ship test instrumentation. Compatibility checks retain their existing coverage only in the explicit local extended suite, including all-features Clippy. Ordinary checks are not silently removed.

## Ownership And Delivery

The existing dokime owner implements in its current worktree, updates the inventory, and owns final local proof after source closure. Root coordinates Hub dependencies. CI 40054181306 completed and supplies the recorded baseline evidence. Publish the new candidate only after the final applicable local validation. Changes require protected review into dev. Do not publish releases.


## Source Inventory

- `mise.toml` owns shipping feature declarations, Product/Tests/compile-contract recipes, the new compatibility task and explicit local extended ordering.
- Mirrored `build/rust`, `build/typescript.py` and `build/python.py` owners compile ordered native hosts, frontend, desktop and independent Tests; direct Product consumers replace copied client artifacts. The existing single archive/lane contract remains.
- `crates/dokime-server/Cargo.toml`, runtime test declarations and their support modules own the real fault-injection boundary. Preserve test cases and injected secret-store/backend semantics.
- `stkr.toml`, rendered root workflows and embedded starter outputs adopt canonical shared guidance only after validated Hub handoff, always without provider credentials.
- README and maintained CI/build requirements describe canonical configurations, compatible reuse and local-only extended compatibility coverage. Targeted proof includes actual Cargo graph selection, existing test enumeration and artifact consumers before final full native validation.

## Ordered Shipping Configuration

Mise declares the desktop embedded-protocol feature and release media feature once. The early non-GUI native producer omits only the desktop switch because Cargo refuses features of an excluded package. After frontend preparation, the final desktop producer selects only dokime-gui with its shipping configuration. Tests may omit frontend embedding when test targets do not consume it; required real desktop journeys still consume the shipping GUI. Product exports remain separate and contain no fault injection. The explicit local extended task enables fault injection; ordinary Tests has no instrumentation.

## Exact Extended Test Inventory

The runtime keeps its canonical integration and acceptance targets enabled with `host`. Only `cycle_restart` (two crash/child cases), `runtime::secrets` (one shared-store case), the FTP module (two shared-store cases), `local_secrets_state` and `undeclared_records_settle` require the fault feature. Backend decoration moves into its own feature-gated `fixture_backends_decorated` case; `node_identity_restart` still runs canonically. Server support injects its test keyring only in the explicit server fault feature; its canonical tests remain registered. Storage-publication tests using ordinary injected storage interfaces stay in CI.

The local extended task may rebuild its profile's single Tests export with instrumentation, then consumes it through existing Rust lanes. Product exports stay separate. Ordinary CI rebuilds the canonical Tests export; no variant framework, duplicate CI runs or hidden instrumentation is introduced.

## Retired Standalone Starter Verification

The user explicitly removes the costly nested generated-starter verification, superseding compiler-cache reuse and fixture-stage migration for that test. Delete build/starter.py, its exclusive build/dependencies.py vendor/wheelhouse helper, the three wheel-selection/download cases in test_starter_dependencies.py and the Tests producer calls. Do not retain this proof in extended or another hidden task. The shipped cargo-dokime new templates and fast generator behavior tests remain. No claim of retained nested standalone compilation coverage is made.

## Native Linux Picker Repair

Run 40054181306 fails its first owned-startup story while choosing a real plugin file. The helper locates the owned GTK dialog but repeatedly sends Return directly to its top-level window using XSendEvent. xdotool documents that explicitly targeted keyboard events differ from normal XTEST delivery and may be refused by applications. Focus the verified dialog and deliver keyboard input through XTEST instead, retaining the dialog identity and closure assertion. Log the actual focus and selection phases. The GTK realization warnings alone do not prove the root cause; required native Linux execution must establish the repair. No test deadline, consent assertion or picker boundary changes.

Primary API contract: https://github.com/jordansissel/xdotool/blob/main/xdotool.pod#sendevent-notes

## Baseline Cost Evidence

The untouched 4d5e930 pipeline shows Linux Product 29m49 and Tests 47m53, with serial Native 79m33; Windows Product alone takes 74m57. Parallelism alone cannot meet the requested whole-pipeline ceiling of 45 minutes (target 30), nor establish Tests no slower than Product. Linux Product sccache reports 0 hits/1862 misses and Tests 3 hits/1808 misses despite the same GHA cache location. No warm-cache speedup is assumed. The largest measured Tests compilation is 23m37, compatibility about 18 minutes, and standalone starter work about five minutes. Canonical-only CI removes the approved extended matrix from this path; remaining Product, GUI, archive and artifact transport phases need separate timing. Significant architecture or product-feature changes require a human decision.

Linux Product phases measured 4m54 Clippy, 10m32 native build, 34s cargo-harmos installation, 5m45 GUI Clippy and 7m42 GUI build. Tests remote-cache read/write errors and forced recaches are zero; average write is 0.438s, with noncacheable crate-type (220) and multiple-input (157) requests. This establishes poor effective reuse, not a cache-backend outage or proven key defect. Cargo test targets, features, profiles and source identities must be compared before claiming cache improvements.

A normal/build Cargo-tree comparison establishes that broadening desktop compilation to the workspace adds host, FFT, sidecar and terminal features to its facade/runtime closure. This is not equivalent to the existing GUI shipping graph. Preserve the original package-scoped desktop selector; independent Tests configuration is permitted. No speed improvement is claimed from workspace-wide feature union.

## Mirrored Task Modules And Direct Artifact Consumers

Retire the build/product.py step dispatcher. Actual task implementations move to build/rust/__main__.py, build/rust/desktop.py, build/rust/tests.py, build/rust/compatibility.py and build/rust/bench.py, plus build/typescript.py and build/python.py. The touched Python wheel/lane owner moves from test/client.py to test/python.py; layer selection remains one shared implementation rather than forwarding modules. Shared profile/features/output and invocation mechanisms stay in build/configuration.py. Update mise commands/imports together; no forwarding modules or public task renames. Existing frontend/wheel/output helpers remain shared.

No Dokime fixture preparation remains after standalone verification retirement. Generated TypeScript consumers restore native Product generated/typescript directly; Python lanes install the any-platform Python Product wheel directly. Remove Tests copies and producer selector. Rust acceptance/system already consume the authoritative any wheel. Tests stamps only its archive/compiler/metadata and keeps scoped verification. Provider dependency selection must supply exactly these exports; no consumer compiles missing artifacts.

The user clarifies that the 45-minute ceiling and 30-minute target apply to warm-cache pipelines. Cold timings remain separate diagnostic evidence; the near-zero-hit baseline does not establish a warm-budget violation. Cache statistics and compatible subsequent execution must substantiate warm classification; no invented hit threshold applies.

## Explicit Artifact Inventory

Native build exports non-overlapping named outputs: runtime (bin/plugins), clients (generated/typescript), packaging (product/web). It consumes the independent any-platform Python Product, allowing CI to skip the already downloaded Python task within the complete local build aggregate. Independent Tests exports only tests (archive/compiler/metadata and stamp). Python Product exports its python tree including the tutorial. Rust/TypeScript/Python consumers declare their actual named artifacts; Python acceptance additionally receives the native runtime for its real host fixture. Generated-client checks receive only their language Product outputs. Native and Python Package outputs remain separate and release consumers select packaged artifacts, with Hub preserving native package legs and success-only gates. Bench exports remain explicit for existing optional benchmark consumers. No test receives a large nextest archive solely to import a Python wheel or generated TypeScript.

The shipped starter build:rust:tests recipe invokes the shared build.cargo archive CLI. This retires its old test.archive compiler call while retaining the actual generated starter build feature. Root and starter reader/stamp modules remain managed and receive canonical regeneration.

Python coverage likewise installs the declared any-platform wheel before instrumented execution, since the shared coverage path does not perform the ordinary Python lane's wheel installation. It retains its existing source scope and deadlines. Rust coverage uses the same nonembedding canonical test features rather than requiring frontend embedding.

## Completed Windows Baseline

Windows native job 120753615035 in run 40054181306 reaches its configured two-hour maximum (provider annotation), 02:27:51–04:28:11 UTC. Product succeeds in 74m57: native Clippy 13m36, native compilation 26m37, cargo-harmos installation 1m12, frontend about 17s, GUI Clippy 14m34 and GUI compilation 18m04. Product reports zero hits/1723 misses and 16 cache-write errors, with zero read errors/timeouts; it does not establish warm performance.

Tests spends 29m04 in all-features compatibility, 10m02 in no-default compatibility and 2m09 in WebAssembly compatibility (41m15 total), then is cancelled about 2m18 into the archive configuration's Clippy. Neither nextest archive completion nor nested starter work is reached. Its cache reports four hits/1093 misses and zero backend errors/timeouts. Sccache connection-reset errors coincide with cancellation and do not establish a separate compiler failure. The new source removes those legacy matrix passes from CI and parallelizes independent producers; it does not claim the cold measurement predicts the future warm critical path. Existing profiles/deadlines remain intact.

## Shipping Product Overlap And Cache Diagnostics

Native Product selects the workspace excluding GUI; desktop selects only dokime-gui with custom-protocol. Both use the same host target/dev profile and incremental-off setting, with no authored Rust flags. A normal/build Cargo-tree comparison finds 277 shared package identities: 227 with equal feature sets and 50 with differences. These are not identical compiler-unit counts: dependency closures, host/build contexts and check versus code generation matter. Runtime host/FFT/storage versus client-native, Windows networking/WMI versus UI/WebView, Harmos FFT and TLS-provider features differ. No shipping feature widening or matching Clippy removal is authorized; no obvious identical invocation can safely be deleted.

The completed Windows cache namespace/prefix, read-write mode and V2 transport agree across Product/Tests. Provider cache entries confirm successful writes on the PR merge ref up to cancellation. Repository usage at inspection is about 10.12 GiB, which suggests possible eviction pressure but does not prove the cause of 16 Product write errors. The canonical cache server reader discards non-startup stderr, losing backend-error details; the diagnostic artifact contains only compiler messages. The old run has no backend-error details. The final canonical wrapper adds successful-write totals and fixed sanitized write-error categories for subsequent runs, without keys, policy or compilation changes. Existing build statistics are sufficient for the fresh candidate; investigate more deeply only if a repeated compatible candidate cannot warm. No speculative logger infrastructure, key changes, cache deletion, profile changes or claimed warm speedup follow from these observations.

The authored Python lane owner validates the exact any-platform wheel and delegates execution to the shared runner, which already installs selected wheels. Remove the duplicate explicit uv installation from the authored owner; retain the shared ambiguity/missing checks and normal dependency integrity.

## Canonical Regeneration And Final Local Proof

Adopted immutable Hub 68027292048F76B05FEB031CB1104438D9D16BBBCC6B92A13B53B68E0BE01450 with provider credentials disabled and project update --no-provider. Root and embedded starter outputs reconcile without conflicts, with authored task owners preserved. Existing package-local ruff.toml owns width 140 and extends authored pyproject settings; no duplicate width setting is added. Generated ordinary starter builds remain canonical, with only the approved explicit local compatibility helper retaining broad selectors.

Final available-platform proof is one sequential ordinary Windows sequence in the assigned worktree, Cargo jobs limited to two: Check, Product, Tests/compile contracts, Rust unit/integration/acceptance/system, Python unit/integration/acceptance, TypeScript unit/integration/native desktop system, generated-client checks, Package and clean-source Dry. Use a fresh normal tempfile-owned pytest basetemp where needed to isolate the already recorded stale administrator-created pytest-current residue; no elevation or TEMP redirection. Native Linux proof, including the picker repair, remains required provider validation. Bench timing and local extended compatibility remain optional explicit follow-up, not ordinary CI gates.

During final Check, repair six owned TOML files with the actual repository column_width=140 option, then rerun the failing Check lane. A separately discovered canonical Windows ci.build stdio defect reopens only wrapper adoption; parent explicitly authorizes continuing unaffected direct mise compilation/testing, with wrapper regeneration and focused proof required before publication. No managed output is hand-formatted.

Canonical wrapper repair adopted from immutable Hub 779DBD2E9C6D00CB809F6BB0FC0363726E33E4408997D11A1A1F7936FA249780 without provider credentials. Root and starter regeneration changes only the Cargo helper and CI artifact/build modules. Native Windows wrapper proof preserves stdout, stderr and exit 7; affected import lint passes. Source is complete again; required failed/unreached Check groups and remaining final proof stay open.

The retired standalone starter was the sole consumer of staged native workspace libraries. Remove its library selector, copying and named packaging export; registry library publication and executable/plugin/installer outputs remain unchanged. Retire the existing owned staging directory with the normal safe staging cleanup. Adopt immutable Hub 508F8CA7E8F2F25ACCA0FC242A36BD66F2A7678BB7C213BEBB35C4BF11863DEA for bounded cache diagnostics and refresh affected embedded starter/Product/package bytes; retain completed application and UI proof.
