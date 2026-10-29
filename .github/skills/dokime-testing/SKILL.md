---
name: dokime-testing
description: Validate Dokime runtime boundaries, built plugins and the native desktop walkthrough.
---
<!-- Rendered by stkr project update from .agents/skills/dokime-testing/SKILL.md; sha256:34348da46db73abd54bf50b2341330affab49cd1b434b845ef96539160daeb45 -->

# Dokime Testing

Apply `stkr-test` for layers and lanes. Use targeted proof during implementation; combined required native validation follows `stkr-ci` after source closure and authorized delivery.

Dokime's system coverage has two stories: Plugin And Host, using explicitly built OBC and host sidecars with the server; Desktop Shell And Backend, using the native shell and server. OBC is test-only under `plugins/sidecars/obc`; the shipping host plugin lives under `plugins/sidecars/host`. Test artifacts prepare OBC under `fixtures/plugins/`, while the product build exports only `plugins/host.dokime`.

Ordinary Rust unit tests consume the Tests archive without fixture-stamp preflight. Other Rust layers require their declared fixtures; acceptance and system lanes require their declared Product hosts.

Use [Native Desktop](references/desktop.md) for native WebView and shell ownership coverage.
