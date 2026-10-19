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

Each lane preflights archive and required built artifacts before executing. Stories consume the build's stamped host, plugins, bindings and wheel; never compile a replacement inside ordinary stories. Rust benches use `<layer>-<domain>` binary names and share the bench profile.

Read [Native Desktop](references/desktop.md) when the native WebView, IPC, shell lifecycle or visible operator workflow changes. Report the platform actually exercised; Linux results do not establish Windows behaviour.
