# test-lane-traces Specification

## Purpose
TBD - created by archiving change test-lane-traces. Update Purpose after archive.

## Requirements

### Requirement: Opt-In Native Span Capture

The runner SHALL enable process-specific, payload-free span lifetime traces for integration, acceptance and system lanes through `DOKIME_TEST_TRACE_DIR`. Native test processes, application hosts and sidecars SHALL install the subscriber and flush normally. Desktop harnesses SHALL emit matching story, scene and step spans. Outside the opt-in environment tracing SHALL remain disabled.

#### Scenario: Native And Desktop Runs

- **WHEN** a traced lane launches a test or child process
- **THEN** it inherits the lane directory and produces `<process>-<pid>.pftrace` containing span identities and timings without payloads

### Requirement: Lane Time Summaries

The runner SHALL write Markdown and JSON reports containing the top 30 spans by inclusive and self duration, per-test totals and slowest tests. Self duration SHALL subtract the union of direct child lifetimes, not double-count overlapping children. CI SHALL retain reports and traces and CONTRIBUTING SHALL explain inspection with Perfetto.

#### Scenario: Overlapping And Failed Work

- **WHEN** a lane completes or fails after producing traces
- **THEN** the summary includes available spans and JUnit totals, reports incomplete files, and never turns a failed lane into success
