---
name: dokime-testing
description: Validate Dokime runtime boundaries, built plugins and the native desktop walkthrough.
---

# Dokime Testing

Apply `stkr-test` for layers and lanes. Use targeted proof during implementation; combined required native validation follows `stkr-ci` after source closure and authorized delivery.

Dokime's system coverage includes runtime persistence and execution stories, Plugin And Host with explicitly built OBC and host sidecars, and Desktop Shell And Backend with the native shell and server. OBC is test-only under `plugins/sidecars/obc`; the shipping host plugin lives under `plugins/sidecars/host`. Test artifacts prepare OBC under `fixtures/plugins/`, while the product build exports only `plugins/host.dokime`.

Shared system-story finalizers save sealed execution reports and reopen their seals and linked artifacts. Stories producing no execution report are exempt only in `crates/dokime-runtime/tests/support/system_reports.json`. Keep report saving in those finalizers, without a dedicated story or command; large-cycle timing belongs to the standard `bench:rust:system` target.

Ordinary Rust unit tests consume the Tests archive without fixture-stamp preflight. Other Rust layers require their declared fixtures; acceptance and system lanes require their declared Product hosts.

Use [Native Desktop](references/desktop.md) for native WebView and shell ownership coverage.
