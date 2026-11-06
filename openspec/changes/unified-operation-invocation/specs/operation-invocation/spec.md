## ADDED Requirements

### Requirement: Authorized Shared Invocation

Templates, criteria and observers SHALL expose `ctx.operation(request)` returning Harmos SDK Invocation, with Response on await and a host-owned instance on start. Called authorization and existing built-in exemptions SHALL be preserved. Request-taking call, template host and StepHost SHALL be retired without aliases; the host plugin remains.

#### Scenario: Declared And Undeclared Calls
- **WHEN** a template invokes a declared operation or an existing exempt built-in
- **THEN** the shared builder routes it with the stage's authority; an undeclared ordinary operation fails compilation.

### Requirement: Scoped Policy And Evidence

No-option calls SHALL use the operation declaration policy unchanged. Configured retry overrides SHALL affect only one call, retain operation deadlines and record every actual attempt. Unknown replay safety SHALL refuse retry-enabling overrides. Routing, harness selection, readiness and cancellation SHALL remain owned by StepOperationHost.

#### Scenario: Configured Retry
- **WHEN** a template selects retry from its settings for a replay-safe operation
- **THEN** the host receives that option, records each attempt and preserves declared deadlines; a later no-option call uses the declaration again.

#### Scenario: Stage Cleanup
- **WHEN** a stage ends while an operation started through the shared builder is unsettled
- **THEN** its existing handle ledger cancels and settles the instance with the correct stage metadata.
