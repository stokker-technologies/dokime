# Composition And Ownership

The existing server library owns boot, listener/runtime resources and bounded drain in a running session. CLI and GUI supply configuration and own process-global tracing, signal handling and presentation. Embedding must not install a second global subscriber or signal handler. Partial startup failures release acquired resources.

`dokime server` replaces `serve` forwarding and the separate server CLI crate. Cargo-subcommand development control, executable discovery, installer hooks and service examples use the same command, preserving JSON control and version checks. The server library's contract export remains owned by that library.

The GUI starts and stops its embedded session in process. Attached and profile-selected remote connections retain their authentication and origin guards. Normal shutdown drains sidecars and releases listener/store resources; the existing bounded whole-GUI force-quit behavior remains. Restart assertions identify session/resource replacement rather than a child PID.

Runtime composition accepts an explicit credential-store resource, separate from serialized settings. Production selects its existing OS-backed adapter; tests use the upstream in-memory store without global mutation, fault features, silent file fallback or a bespoke mocking framework. Existing secrets and secure adoption rules remain unchanged.

Product uses default dev/release; Tests and preceding Clippy use built-in test, including release graph contexts. Artifact paths preserve the graph profile. Packaging symbol policy remains separate from compiler flags. Contract tests consume the just-built archive and explicit graph namespace.

The shared test runner assigns child-only `STKR_TEST_PROFILE` from its parsed profile for ordinary tests and coverage. Archive contract execution supplies the same value explicitly; Rust fixture readers validate dev/release and default to dev for direct Cargo execution. Mise task environment tables do not reference usage arguments.

`test:rust:starter` maps to `dokime_tools.test.rust.starter`, declares `pipelines = ["nightly"]` and explicit Product input. It generates and compiles the real standalone consumer. Ordinary scaffold assertions remain; no unrelated suite consolidation is included.

## Report Browser Ownership

Native report journeys supply a test-scoped opener resource targeting an owned browser profile and debugging endpoint. The gate records exact report tab identities and closes those tabs in its final cleanup before deleting fixtures, including failure and timeout paths. Production keeps the system opener. No user browser enumeration, tab closure or process termination is permitted.

## Nightly Profiles

Nightly runs full ordinary validation with default release Product and Package outputs, unsigned downloadable installers, default unoptimized test-profile Tests and coverage, and default bench-profile benchmarks. It does not sign or publish.

## Source Ownership

The server library exposes one Session for boot, plugin admission/reload, listener ownership and bounded drain. CLI server composition owns dotenv, tracing, terminal/JSON output and signals. GUI composition retains its existing connection revision owner and supplies the embedded session; startup diagnostics use the shared application log rather than subprocess exit codes or stderr capture. Cancellation guards retain cleanup ownership during plugin initialization and session startup.

The nightly starter module owns its generated Git/Cargo boundary, standard Cargo vendoring, explicit candidate SDK/tool override and real standalone commands. It neutralizes machine/ancestor mise configuration and inherited task-skip directives, uses normal uv locking/cache integrity, and runs offline commands after dependency bootstrap. It may reuse compatible Cargo target units; this is not cold-cache proof. No custom Python wheel hash gate is restored.

Root README, CONTRIBUTING and the preserved product guide are prose inputs checked by repository policy. Embedded starter templates, crate documentation, generated resources and executable examples remain outside that narrow impact rule.

Linux desktop bootstrap owns its report browser dependency. It verifies an explicit browser override or installed Google Chrome, otherwise installs Chrome For Testing 156.0.8078.47 from Google's versioned public archive into target/tooling/desktop. Only desktop consumers install its runtime libraries; other jobs remain unaffected. This pinned Stable version comes from Google's [published release metadata](https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json). Browser selection is passed to both real report-rendering paths; no report assertions are removed.

## Runtime Artifact Inputs

Product keeps the same compiler outputs and default Cargo profiles. Named transfers separate CLI, GUI, generator, plugins, generated clients and packaging metadata rather than transferring every executable to each consumer. Linux filenames have no suffix; Windows filenames use .exe.

| Consumer | Product Inputs |
| --- | --- |
| Rust Acceptance | CLI, generator, plugins, generated clients |
| Rust System | CLI, plugins |
| Rust Coverage | CLI, GUI, generator, plugins, generated clients |
| Nightly Starter | Generator |
| TypeScript System | GUI, CLI, plugins, generated clients |
| Python Acceptance And Coverage | CLI, plugins |
| Rust Package | CLI, GUI, generator, plugins, packaging metadata |

Desktop journeys need the CLI as well as the GUI: remote attachment and interoperability still launch a separate headless server through dokime server. The owned desktop session itself remains embedded. Rust acceptance exercises the GUI session library and substitutes a controlled GUI launcher for installation discovery, so it does not require the shipping GUI executable. Archive stamp checks remain scoped to Tests/fixtures; artifact readers request individual files and do not demand omitted siblings. No symbol stripping, profile change, fixture removal or coverage reduction accompanies this split.

## Plugin Consent Admission

The desktop submits the installation identity and digest displayed in its review dialog. Consent verifies the stored bundle once for the operation, then runs synchronous digest and complete-disclosure admission checks on that verified entry before publishing settings. The RPC maps refusals to the existing failed-precondition and resource-exhausted statuses. Configured and installed bundle collision checks still verify their bytes; no persistent metadata cache or deadline increase is introduced. Refusal leaves both the staged installation and any previously installed authorization unchanged.
