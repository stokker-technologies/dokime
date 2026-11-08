## Why

Data shape identities must derive from their schemas; plugin evolution must preserve existing roots and self-describing execution evidence.

## What Changes

- Pin Harmos's derived identity and stored plugin schema APIs and adopt all Rust, Python, GUI and plugin consumers.
- Derive stream, record and snapshot identities; remove redundant shape ordinals and their readers.
- Replace the source digest root guard with decode checks for Dokime-owned persistent state only. Preserve all stored data.
- Prove execution publication, sealing, reports, unchanged restart, plugin evolution on one root and named built-in refusal.

## Capabilities

### New Capabilities
- `data-shape-identity`: Schema-derived identities and persistent-state compatibility.

### Modified Capabilities

## Impact

Domain contracts, runtime persistence and evidence, clients, GUI bindings, Python authoring and stubs, plugin declarations, templates and maintained documentation. API protocol, release and algorithm versions remain unchanged. No migrations or resets.
