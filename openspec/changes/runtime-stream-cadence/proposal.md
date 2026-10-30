# Runtime Stream Cadence

## Why

Host collection cadence is currently compiled into observer inventories and cannot
use the plugin settings snapshot. The owner approved a paired Harmos change and
local integration override, with commits only and no push.

## What Changes

- Retain declared observer defaults and allow runtime resource-based interval overrides.
- Add all six host collection intervals to native plugin settings, retaining defaults.
- Migrate Dokime consumers, OBC, starter templates, tests and affected guidance.
- Keep schema, identifiers and bounded replay windows static.
- Preserve operation policies and retries; CPU-load features are outside this change.

## Impact

The SDK change requires a matching Harmos revision. Development uses the paired
Harmos worktree; the final published dependency pin remains an integration follow-up.
