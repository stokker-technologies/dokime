---
name: dokime-testing
description: Validate Dokime runtime boundaries, built plugins and the native desktop walkthrough.
---
<!-- Rendered by stkr project update from .agents/skills/dokime-testing/SKILL.md; sha256:1469cf715955f1150650a12f2bc15e1b0c9e5d805cfd1419a6070cc39df9d39b -->

# Dokime Testing

Apply `stkr-testing` for layers and lanes. Run the full lanes locally on the combined tree before opening or updating a review; CI confirms.

Dokime's system coverage has two stories: Plugin And Host, using the built `obc-plugin` and server; Desktop Shell And Backend, using the native shell and server. Keep the shipped plugin as the shared system fixture.

Ordinary Rust unit tests consume the Tests archive without fixture-stamp preflight. Other Rust layers require their declared fixtures; acceptance and system lanes require their declared Product hosts.

Use [Native Desktop](references/desktop.md) for native WebView and shell ownership coverage.
