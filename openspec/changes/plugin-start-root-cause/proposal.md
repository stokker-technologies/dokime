# Plugin Startup Root Causes

## Why

A failed plugin prepare is rendered as a Rust Debug aggregate; cascaded missing resources obscure actionable causes and lose secret actions.

## What Changes

- Harmos owns startup error classification, readable rendering and structured transport.
- Dokime maps that report onto the existing persisted failure, settings form and retry flow.
- Preserve every diagnostic and structured refusal; never parse resource names or error text.
- Use only check, format and focused test proof; commit locally without pushing.

## Impact

Paired producer: ../harmos.wt/rootcause (`fix/startup-error-root-causes`). Integration uses an uncommitted Cargo patch. Final upstream pin adoption belongs to the orchestrator after both changes land.
