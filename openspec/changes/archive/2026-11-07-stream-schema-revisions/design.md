# Stream Revision Design

The owner approved automatic row changes without author version bumps or a backend restart. Harmos owns native publication revision identity and activation; Dokime derives its composed telemetry storage revision from the payload graph with Harmos's canonical revision function and a fixed storage window. Native lease demand resolves the committed current publication by id. The writer stamps the storage revision from its codec, while readers retain each revision and its definition.

Finite queries select an execution and stream id/column. They use captured revisions; absent columns contribute no values. Cycle reports compose each execution's frozen documents and plots independently. Combined raw previews union column names, retain original values and use text presentation for incompatible types or units, with a warning naming the stream and column. This avoids selecting a schema by digest order or coercing stored values. Document binding errors remain isolated to their cards.

Codec registration, persisted schema lookup and saved codec collection refuse conflicting graphs under a key. No migration, deletion or reset is introduced. This build changes the existing source-derived storage-format digest, so older roots refuse with StorageFormatMismatch and require a fresh root.

Report publication failure is already durable. A dedicated finalization failure carries its already actionable cause; generic internal report errors remain private. Control wrongly treated any unsaved terminal execution as finalizing. The control projection now treats durable report failure like successful publication, and GUI Control consumes the same report outcome. Frozen step projection puts runtime initialization before the ordered template steps and cleanup.

Validation follows the explicit owner override: the selected workspace build, GUI TypeScript build and focused tests only. No lint, starters, full lanes or local simulation.

## Restart Cycle Decision

Preserve RecoverInterruptedCycles sealing every open cycle on startup, including idle cycles. Both executions' reports must build with their captured values across a restart. Prove mixed revisions within a new cycle created after restart, using live reload between its executions without another restart inside that cycle. The pre-restart execution report must remain readable after those reloads.
