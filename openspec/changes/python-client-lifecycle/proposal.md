# Lean Python Client Lifecycle

Implement the owner's locked 2026-10-30 Python API as a clean development break.
The delivery is commit-only, with Python checks and focused client proof; no push,
release, broad validation lanes or hilti-hcl edits are authorized.

The cycle context owns cleanup on every exit. Start resolves and materializes a
template, appends fresh attempts and starts them. One case returns one awaitable
execution; expansion returns a tuple in materialization order, with starts
admitted in order to the existing server queue. Records and samples follow through
the saved seal and serialize through `to_json()`. Outcomes expose exit codes.
Save produces evidence ZIPs, optional HTML and a digest manifest without JSONL capture.
Retain primitives used by planning, interactive commands, recovery and evidence.
Remove callback run composition and migrate every repository consumer.
