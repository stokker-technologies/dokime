## 1. Vocabulary

- [x] 1.1 Replace ended and finished with sealed in host errors, phases, capabilities and projections for executions and cycles; keep stored states unchanged.
- [x] 1.2 Replace the same wording in the desktop, the command line and every SDK client surface.

## 2. Finality

- [x] 2.1 Refuse new notes on a sealed execution with an actionable error; prove refusal after seal and acceptance before it.
- [x] 2.2 Refuse new attached files on a sealed execution; prove the same pair of cases.

- [x] 2.3 Offer an execution seal call that returns the outcome, repeats safely and refuses running or never-started executions; progress Watch emits one sealed outcome frame and ends, including reattachment; notes and artifacts after the run go to the cycle until it is sealed.

## 3. Cycle Seal

- [x] 3.1 Verify what the evidence archive contains today (samples, records, files, notes, documents, report) and record any gap against full evidence here before implementing.
- [ ] 3.2 Materialize the cycle report during seal under the Sealing phase and retain it; report Seal Failed with the cause and leave the cycle sealable.
- [x] 3.3 Materialize the evidence archive containing that report during the same seal and retain it.
- [x] 3.4 Return the retained report and archive on every download; prove equal digests across two downloads.
- [x] 3.5 Refuse a cycle seal while an execution is running and name it.
- [x] 3.6 Seal never-started executions as not run during a cycle seal, list them as not run in the report, exclude them from pass and fail counts and refuse a later start.

### Archive Inventory Before Implementation

The source inventory is `host/evidence/archive.rs`, storage `content.rs` and
`execution.rs`, and the native notes/content/report handlers. The execution ZIP
contains `evidence.pb`, finalized `telemetry/<stream>.arrow`, native `files/`
bytes and references, and `manifest.json`. `evidence.pb` carries the frozen
execution/catalog (including document declarations), ordered records/logs and
other execution evidence, the execution seal, already-published execution reports,
and committed artifact/execution-file metadata. Arrow members preserve all saved
samples/telemetry. Derived `view/` and `temporary-evidence/` files are excluded.

The cycle ZIP contains `executions/<id>.zip` and `manifest.json` only. It omits
cycle-owned notes, files and artifacts. Neither path creates a report; execution
reports appear only if previously published. Native operator notes live outside
the execution evidence table and are absent from both ZIPs. Frozen document
declarations are retained, but evaluated documents exist only inside a separately
created report. These are the gaps against full evidence including the report.

Produced ZIP inspection passed through the focused storage archive test (1 test):
`evidence.pb`, two `files/<digest>` byte/reference pairs and `manifest.json`
for the execution; `executions/execution.zip` and `manifest.json` for the cycle.
Neither ZIP contained `report.html` or native operator notes.

### Publication and Archive Decisions

The cycle seal owns publication and recovery of its staged report and archive.
They become visible with Sealed in one native storage compound mutation.
Reopening discards abandoned cycle-owned staging. Independent report publication
is not called by cycle sealing. Prepared reports have one durable lookup;
the temporary runtime registry is removed.

- [ ] 3.7 Publish the staged report, archive and sealed cycle atomically and prove late failure recovery.
- [x] 3.8 Replace nested execution ZIPs with ordered, sanitized template folders and individual reports.
- [x] 3.9 Project execution evidence into deterministic JSON, replace the unused encoded evidence file and its wire structs, and hash every archive member.
- [x] 3.10 Apply execution/cycle seal timestamps and dashed telemetry names; prove the exact three-execution entry list, sanitization, JSON, timestamps and repeat digests.
- [x] 3.11 Remove the temporary runtime report registry and migrate every caller to durable prepared reports.
- [x] 3.12 Add the starter temperature observer and creation-time archive filename; update its description and generated-project stories and README.

The source inventory adds storage archive construction and native compound/recovery,
operations report capture and archive staging, runtime report callers, archive
stories in storage/runtime/CLI, and Python SDK downloads, starter and stories.
Documentation under docs/ is owner-written and is reported rather than edited.

The pinned native compound contract admits objects, records, object deletions and
scope deletions; it has no staged-file publication item. Cycle state still lives
in execution.sqlite3 while native files and compound mutations live in
storage.sqlite3. Atomic cycle publication must close both gaps at the storage
owner; a second transaction mechanism or product-side publication mirror is not
an allowed replacement. Task 3.7 and the late archive-failure regression remain
open until that single-boundary storage facility is available.

## 4. Validation and Delivery

- [ ] 4.1 Focused host, desktop and command-line proof of every scenario.
- [ ] 4.2 Hosted validation after delivery.
