# Organization Installers

## Preserved Validation Obligation

This change was archived with task 4.1 still unchecked. Its source requirements
already live in `openspec/specs/organization-installers/spec.md`; that current
specification supersedes the historical deltas here, including retired inspection.
This restored record preserves the outstanding all-platform validation obligation.
Historical implementation and publication authorization below is not renewed by
restoration. Do not reapply historical deltas over the current main specification.

## Why
Distribute unchanged Dokime binaries with a company organization pack using the same native packaging command in product CI and downstream distributions.

## What Changes
- Preserve the existing organization format and CSS/font/logo model; add tray and installer asset roles.
- Remove compiled identifier and storage slug from runtime organization declarations; the current desktop does not register deep links.
- Resolve bundled defaults plus folder overrides: merge manifest fields, replace whole files, then validate; retain last valid activation on invalid input.
- Add cargo dokime installer package/inspect with local artifacts or a pinned verified release payload, ephemeral generated assets and explicit signing.
- Group current authoring commands under sidecar; build still produces .dokime and advanced package never compiles.
- Replace product installer assembly with the public command and inspect the actual output; reuse native lifecycle tests and focused command tests.

## Impact
Runtime/domain organization contracts, GUI and report appearance, Cargo authoring/packaging, build exports, native installers, release assets, CI, tests and current documentation change together. Compiled application identity and updater signing trust remain unchanged. Company distributions default to disabled update checks and use manual/deployment-managed upgrades.

## Authorization
The user approved the contracts in discussion and authorized unattended implementation with GPT-6.1 Sol medium native subagents and an auto-merge PR into dev. Native required gates and provider protections remain mandatory. The baseline is the latest committed CI-fixes branch; its unrelated work remains owned by PR 7.

