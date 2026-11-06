# Live Artifact Tree

## Why

Recursive Stream watches currently expose UUID upload directories, retain untouched baseline files as empty artifacts, and hide captured files until sealing.

## What Changes

- Store and display artifacts at `artifacts/<requested path>`; retain UUID identity internally.
- Reserve names in admission order, suffixing collisions before the extension with `.2`, `.3`, and so on; preserve existing rotation segment names.
- Exclude untouched Stream baseline files. Retain newly discovered empty files.
- List captured Stream files immediately, follow growing bytes with existing bounded execution-file reads, and expose completion when sealed.
- Apply the same path allocation to step uploads. Keep path validation and the v1 clean storage break without migration or reset.

## Impact

Runtime artifact storage, file watches, native execution-file reads/follows, harness parity, GUI regression proof and maintained product/GUI guides. Existing watch outcomes and reports retain only actual captured artifacts. No wire shape change is planned.
