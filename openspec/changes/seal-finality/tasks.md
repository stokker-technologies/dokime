## 1. Vocabulary

- [ ] 1.1 Replace ended and finished with sealed in host errors, phases, capabilities and projections for executions and cycles; keep stored states unchanged.
- [ ] 1.2 Replace the same wording in the desktop, the command line and every SDK client surface.

## 2. Finality

- [ ] 2.1 Refuse new notes on a sealed execution with an actionable error; prove refusal after seal and acceptance before it.
- [ ] 2.2 Refuse new attached files on a sealed execution; prove the same pair of cases.

- [x] 2.3 Offer an execution seal call that returns the outcome, repeats safely and refuses running or never-started executions; progress Watch emits one sealed outcome frame and ends, including reattachment; notes and artifacts after the run go to the cycle until it is sealed.

## 3. Cycle Seal

- [x] 3.1 Verify what the evidence archive contains today (samples, records, files, notes, documents, report) and record any gap against full evidence here before implementing.
- [ ] 3.2 Materialize the cycle report during seal under the Sealing phase and retain it; report Seal Failed with the cause and leave the cycle sealable.
- [ ] 3.3 Materialize the evidence archive containing that report during the same seal and retain it.
- [ ] 3.4 Return the retained report and archive on every download; prove equal digests across two downloads.
- [ ] 3.5 Refuse a cycle seal while an execution is running and name it.
- [ ] 3.6 Seal never-started executions as not run during a cycle seal, list them as not run in the report, exclude them from pass and fail counts and refuse a later start.

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

## 4. Validation and Delivery

- [ ] 4.1 Focused host, desktop and command-line proof of every scenario.
- [ ] 4.2 Hosted validation after delivery.
