# Contributing

## Development Setup

Install mise, trust the checkout configuration and run `mise run prepare`.
Dependencies live in the root `.venv/`, the frontend package and Cargo target.
See [Repository Tooling](tooling.md) for native fixture prerequisites.

## Development Workflow

Read [Architecture](architecture.md) and the owning OpenSpec change before editing.
Use focused mise tasks and explicit file ownership. Keep conventional local commits;
delivery requires the owner's authorization. Generated bindings are regenerated
through their task. Desktop native membership and packaging restore together with
its host switch; see the combined grpc-plugins task list.

## Tests and Checks

```sh
mise run format
mise run check -- --language rust
mise run check -- --language python
mise run check -- --language typescript
mise run check -- --repository
mise run build:host-fixtures
mise run test -- --package dokime-runtime-next --target system --story storage_root_admission
```

Use deterministic user journeys, events and controlled clocks. Tests never assert
wall-clock speed, load, memory readings, authored prose or one repository constant.
Hosted CI/Nightly provide full Linux/Windows validation after source closure;
local work uses focused tasks. [Tooling](tooling.md) describes archives and reports.

## Pull Requests

Deliver one agreed feature/batch into dev with Conventional Commits. Describe the
concrete behavior, validation and limitations. Never hide local failures or bypass
hooks. Release publication is separately authorized and requires exact-commit proof.

## Reporting Issues

Include the source revision, platform, operation, actionable error and reproducible
steps. Redact credentials and private settings. See [Security](SECURITY.md) for
sensitive reports and [Privacy](privacy.md) for diagnostic content.
