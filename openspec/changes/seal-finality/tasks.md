## 1. Vocabulary

- [ ] 1.1 Replace ended and finished with sealed in host errors, phases, capabilities and projections for executions and cycles; keep stored states unchanged.
- [ ] 1.2 Replace the same wording in the desktop, the command line and every SDK client surface.

## 2. Finality

- [ ] 2.1 Refuse new notes on a sealed execution with an actionable error; prove refusal after seal and acceptance before it.
- [ ] 2.2 Refuse new attached files on a sealed execution; prove the same pair of cases.

## 3. Cycle Seal

- [ ] 3.1 Verify what the evidence archive contains today (samples, records, files, notes, documents, report) and record any gap against full evidence here before implementing.
- [ ] 3.2 Materialize the cycle report during seal under the Sealing phase and retain it; report Seal Failed with the cause and leave the cycle sealable.
- [ ] 3.3 Materialize the evidence archive containing that report during the same seal and retain it.
- [ ] 3.4 Return the retained report and archive on every download; prove equal digests across two downloads.
- [ ] 3.5 Refuse a cycle seal while an execution is running and name it.

## 4. Validation and Delivery

- [ ] 4.1 Focused host, desktop and command-line proof of every scenario.
- [ ] 4.2 Hosted validation after delivery.
