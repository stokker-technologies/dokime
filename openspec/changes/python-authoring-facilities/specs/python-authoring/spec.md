## ADDED Requirements

### Requirement: Single Operation Invocation
The SDK SHALL expose one awaited operation call on Context and Client, default-construct an omitted request and remove BoundOperation.

#### Scenario: Omitted Request
- **WHEN** an author awaits ctx.operation(OP) with a default-constructible request type
- **THEN** the SDK constructs that type and returns the typed response through the existing contract-checked invocation path

### Requirement: Ordered Standard Logging
The SDK SHALL capture synchronous standard logging in the active scope and flush before awaited host calls and handler return, mapping levels to protocol severities.

#### Scenario: Evidence Ordering
- **WHEN** a handler logs then records evidence or returns
- **THEN** buffered logs reach evidence.Logs/Append first; records outside execution go to plugin diagnostics

### Requirement: Plugin Observer Classes
The SDK SHALL register plugin-level Observer, WindowObserver and Publisher classes with generic row types, plugin-qualified snake_case identities, explicit template selection and optional close.

#### Scenario: Sampling Period
- **WHEN** the host starts and stops sampling a stream
- **THEN** a fresh instance serves that period, closes on stop and broadcasts its records to every capturing execution

### Requirement: Plugin Facilities
The SDK SHALL bind Setting, Secret, Resource and Record attributes by attribute name; the host SHALL validate saved setting constraints.

#### Scenario: Snapshot and Resource
- **WHEN** an activation or Retry begins and concurrent handlers request a resource
- **THEN** synchronous setting reads use the activation snapshot and the resource opens once under a lock and closes at deactivation

#### Scenario: Missing Secret
- **WHEN** a declared secret is missing or empty
- **THEN** the SDK raises Refused with an instruction to store the labelled secret for this plugin and Retry

#### Scenario: Invalid Setting
- **WHEN** a saved setting violates its type, required, choices or bounds constraints
- **THEN** the host refuses the save with an actionable error

### Requirement: Typed Parameters
The SDK SHALL expose typed constrained Parameter declarations and ctx.parameter; the host SHALL validate constraints at case creation.

#### Scenario: Invalid Case Parameter
- **WHEN** case creation supplies a value outside choices or bounds
- **THEN** the host refuses creation before the template executes

### Requirement: Handler Failure Policy
The SDK SHALL accept OperationPolicy.transient and classify unexpected handler exceptions as Permanent unless the predicate selects Transient, preserving identity and exception text.

#### Scenario: Predicate Classification
- **WHEN** a handler raises an exception
- **THEN** the policy predicate determines transient classification while an unselected exception yields Permanent with the handler identity

### Requirement: Declaration Expressions and Plots
The SDK SHALL offer Record.field and Record.time_since, observer default plots and explicit document placement using one parameter-reference plot API.

#### Scenario: Explicit Placement
- **WHEN** a template selects an observer and places its plots in a section
- **THEN** only explicitly placed plots appear and record expressions resolve declared identities

### Requirement: Self-Contained Examples
The SDK SHALL show one file per operation group or template with no plugin-wide shared helper modules.

#### Scenario: Starter Authoring
- **WHEN** an author reads the starter template
- **THEN** its declarations and behavior are visible in its own file using the replacement public surfaces
