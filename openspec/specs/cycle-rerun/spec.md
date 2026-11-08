# cycle-rerun Specification

## Purpose
Rerun an existing cycle as a new planned, unrun cycle whose cases resolve against the plugin installed now, reporting what that plugin no longer declares.

## Requirements

### Requirement: Rerun a Cycle as a New Planned Cycle
`Execution.RerunCycle` SHALL create a new cycle from any existing source cycle: Active, Sealed or archived. The new cycle SHALL carry the source label verbatim unless a label is given, the source's `continue_automatically`, and the source's planned cases in ordinal order with the same suite, case and parameters, each planned and unrun. No attempts or evidence SHALL be copied and the source SHALL be unchanged. The new cycle SHALL record `rerun_of` and `CycleSummary` SHALL expose it.

#### Scenario: Sealed Source
- **WHEN** a sealed cycle with parameterized cases is rerun
- **THEN** a new Active cycle holds the same cases and parameters in order, every execution unstarted, and `rerun_of` names the source

### Requirement: Resolve Against the Installed Plugin
Cases SHALL resolve against the plugin installed now. A case whose suite or case no longer exists SHALL be skipped and reported with its case id, suite id and reason; parameters the current case no longer declares SHALL be dropped and reported the same way. If no case survives, the rerun SHALL be refused with an actionable error naming the plugin and the missing ids, and nothing SHALL be created.

#### Scenario: Missing Case
- **WHEN** the installed plugin no longer declares one of the source's cases
- **THEN** the new cycle holds the remaining cases and the reply lists the missing case as skipped with its reason

#### Scenario: No Surviving Case
- **WHEN** none of the source's cases exists in the installed plugin
- **THEN** the rerun is refused naming the plugin and the missing case ids, and no cycle is created

### Requirement: Rerun Cycle in the Desktop
The cycle's menu SHALL offer Rerun Cycle in every phase, archived or not. On success the desktop SHALL open the new cycle, whose Control card SHALL state skipped cases as "N cases skipped: <ids>, <reason>" when the reply lists any. A rerun cycle's summary SHALL show "Rerun of <source label>" linking to the source.

#### Scenario: Sealed Cycle Menu
- **WHEN** a sealed or archived cycle's menu opens
- **THEN** Rerun Cycle is offered

#### Scenario: Skipped Detail
- **WHEN** a rerun reply lists skipped cases
- **THEN** the new cycle's Control card reads "N cases skipped: <ids>, <reason>"
