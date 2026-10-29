---
name: dokime-testing
description: Validate Dokime runtime boundaries, built plugins and the native desktop walkthrough.
---
<!-- Rendered by stkr project update from .agents/skills/dokime-testing/SKILL.md; sha256:1ee4b8a3d7b182ed53078a8d4978760e1b1a5163eff3c7ee6d9628b055cb268f -->

# Dokime Testing

Apply `stkr-testing` for layers and lanes. Run the full lanes locally on the combined tree before opening or updating a review; CI confirms.

Dokime's system coverage has two stories: Plugin And Host, using the built `obc-plugin` and server; Desktop Shell And Backend, using the native shell and server. Keep the test-only OBC plugin as the shared system fixture; release downloads select shipped plugins separately.

Ordinary Rust unit tests consume the Tests archive without fixture-stamp preflight. Other Rust layers require their declared fixtures; acceptance and system lanes require their declared Product hosts.

Use [Native Desktop](references/desktop.md) for native WebView and shell ownership coverage.
