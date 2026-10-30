# Direct Secret Lookup

## Why

The owner locked this decision on 2026-10-30: remove one-time plugin secret
bindings. A declaration names the shared Secret Storage identifier directly.
Missing required secrets fail plugin startup; adding the identifier and restarting
Dokime or using the existing Retry reads the store again.

## What Changes

- Delete secret references, environment sources, per-plugin secret writes and editable secret fields.
- Retain required/optional declarations and expose read-only availability with a prefilled Add Secret action.
- Derive Secret Storage references from installed plugin declarations, including failed plugins.
- Ignore saved references on load, keep storage v1, and preserve stored secret values.
- Regenerate clients and update the starter, authoring guidance and affected documentation.

## Impact

This is a clean SDK and wire break. External plugins must rebuild and declare the
actual store identifier; hilti-hcl is outside this repository and will not be edited.
Startup root-cause presentation belongs to the concurrent worker. This change only
supplies the typed missing-secret identifier and removes obsolete binding UI.
Validation is check-only through project mise tasks. Commit locally; do not push,
build releases/installers, run umbrella lanes or simulate CI.
