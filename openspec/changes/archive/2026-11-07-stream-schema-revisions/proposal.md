# Stream Schema Revisions

## Why
Changed rows currently share a storage identity and can make saved reports unreadable. The owner approved automatic schema-derived revisions and terminal failure settlement.

## What Changes
Adopt the agreed Harmos revision, bind every saved row to its immutable definition, retain current stream presentation by id, and reject ambiguous definitions with actionable evidence errors. Cross-revision queries omit absent columns; cycle reports compose executions independently, union their preview fields and display conflicting column types as text with a named warning rather than failing the report. Settle failed finalization durably and order steps by their frozen plan.

## Impact
Runtime storage, queries, reports, native ingestion, macros, Python SDK, plugins, GUI, authoring guidance and focused tests. Development contracts remain v1. Existing ambiguous roots require fresh storage without deletion or migration. The owner explicitly limits validation to the workspace build, TypeScript build if changed and focused tests; no simulation or full lanes.
