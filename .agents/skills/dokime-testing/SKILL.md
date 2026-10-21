---
name: dokime-testing
description: Validate Dokime runtime boundaries, built plugins and the native desktop walkthrough.
---

# Dokime Testing

Apply the shared testing layers and project validation phase. Integration stories run one system in process; acceptance drives one built product through its public client. A spawned plugin belongs to the system walkthrough.

Dokime owns exactly two system stories, each made of named scenes:

1. Plugin And Host: the built `obc-plugin` bundle and server; install, activate, create a cycle, append cases, run a suite, prompts, live evidence, sealed report, reload/recovery and stop.
2. Desktop Shell And Backend: the built shell over the server through the native bridge; open, browse, run, live plots, report and shutdown, driven through WebKitWebDriver on Linux.

Preserve every behaviour when consolidating scenes. Before retiring a fixture, map each behaviour into a named scene or an in-process story. The shipped `obc-plugin` is the shared system fixture. Integration stories await observable changes or use a paused clock for time semantics; they do not poll with sleeps or assert wall-clock windows.

Prerequisites are selected per layer and language. Ordinary Rust unit tests consume the Tests archive without fixture-stamp preflight; other Rust test layers require their Tests fixtures. Acceptance and system lanes consume the declared hosts and Product outputs, with bindings and Python wheels only where the lane declares those products. The runner resolves host prerequisites through `hosts_for(category, language)` and product prerequisites through `products_for(category, language)`; do not require every artifact for every lane or compile replacements inside ordinary stories. Rust benches use `<layer>-<domain>` binary names and share the bench profile.

Read [Native Desktop](references/desktop.md) when the native WebView, IPC, shell lifecycle or visible operator workflow changes. Report the platform actually exercised; Linux results do not establish Windows behaviour.
