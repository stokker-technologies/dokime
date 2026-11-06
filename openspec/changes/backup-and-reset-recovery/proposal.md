# Back Up And Reset Recovery

## Why

The desktop already owns safe backup and reset primitives, but a failed local
startup cannot reach them from the connection screen and reset removes installed
bundles. The owner's request agrees the scope and acceptance below.

## What Changes

- Offer confirmed backup and reset after local startup failures, including corrupt
  storage and incompatible formats, excluding another backend's occupied folder.
- Name the data root and archive before confirmation; archive before deletion and
  retain the saved path after success or a later restart failure.
- Preserve installed bundle bytes, enabled state and valid settings across clear.
  Reuse ordinary plugin admission and show refused installations on Plugins.
- Preserve recoverable node identity and secret catalog without touching credentials.
- Report stopping, archiving, deleting, reinstalling and starting through reset steps.

## Capabilities

### New Capabilities

- `desktop-storage-recovery`: Confirmed startup recovery with plugin preservation.

### Modified Capabilities

None.

## Impact

Native desktop session and storage IPC, frontend connection screen/session owner,
runtime plugin recovery and admission, targeted Rust/TypeScript stories, GUI guide.
README reviewed: its existing GUI and documentation claims remain accurate.
Validation is explicitly targeted only; no full simulation. Delivery is authorized
as one immediately squash-merged PR into dev followed by worktree cleanup.
