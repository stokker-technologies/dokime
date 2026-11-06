## ADDED Requirements

### Requirement: Explicit Settings Application
The backend SHALL save plugin settings without restarting programs and SHALL report field identifiers whose saved values differ from the running generation startup values. GUIs SHALL show Applied On Reload with a warning highlight for these fields. A backend restart starts from saved settings.

#### Scenario: Save And Reopen
- **WHEN** a running plugin setting is saved and its view is reopened or another GUI attaches
- **THEN** the program continues with its original values and every GUI receives the same differing field identifiers

#### Scenario: Reload Saved Values
- **WHEN** Reload successfully starts a new generation
- **THEN** its settings reflect saved values and matching field marks clear

### Requirement: Unified Reload Control
Plugin pages and table rows SHALL expose Reload. The table SHALL align statuses in their own column and expose Settings, Reload and Delete directly. Reload SHALL replace equivalent Retry behavior and retain Restart Pending and Restart Refused semantics.

#### Scenario: Active Work
- **WHEN** Reload would stop a program during an active cycle
- **THEN** Restart Pending remains until existing restart admission allows it

#### Scenario: Inactive Program
- **WHEN** Reload targets an installed plugin with no running program
- **THEN** it starts immediately using saved settings even during an active cycle

#### Scenario: Refused Reload
- **WHEN** replacement preparation is refused
- **THEN** the current generation remains running and retains differing field marks

### Requirement: Observer Generation Ownership
Reload SHALL stop and restart all observers owned by affected plugins, including plugin-activated observers, and new observers SHALL read newly prepared settings.

#### Scenario: Updated Collection Interval
- **WHEN** a plugin-activated observer collection interval is saved and Reload completes
- **THEN** the old observer has stopped and the replacement collects at the new interval
