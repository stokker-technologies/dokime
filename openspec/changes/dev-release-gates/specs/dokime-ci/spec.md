## ADDED Requirements

### Requirement: Development Build Scope
Dokime SHALL own its workflows. Dev pushes SHALL run quick checks and builds without packages, publication or suites. PRs SHALL run every applicable job without impact planning. Nightly SHALL retain full native workloads. Dev merges SHALL remain ungated.

#### Scenario: Dev Push and Review
- WHEN a dev push is planned
- THEN quick checks and builds run without packages, publication or suites
- AND ordinary PRs run all applicable suites without release pin rejection

### Requirement: Release Evidence
Release reviews and publication SHALL reject git dependencies. Nightly SHALL warn
and continue workloads that can run with git dependencies. Release tagging and
publication SHALL require successful required CI evidence for the target SHA.

#### Scenario: Missing or Red Commit Proof
- WHEN release target CI is missing, pending or failed
- THEN no release tag, main fast-forward or publication is allowed until green evidence exists

### Requirement: Complete Nightly Publication
One serialized job SHALL consume both native platforms and Python packages,
draft the Nightly channel, upload all assets including SHA256SUMS, update the channel tag
once and publish only after the full inventory is verified.

#### Scenario: Interrupted Upload
- WHEN any upload fails
- THEN no partial asset inventory is published
- AND a subsequent run can resume by replacing the draft inventory

### Requirement: Owner Ruleset
Documentation SHALL describe main's required CI status check as an owner setting.

#### Scenario: Branch Policy
- WHEN this change is delivered
- THEN repository settings remain unchanged and any combined delivery follows explicit owner authorization
