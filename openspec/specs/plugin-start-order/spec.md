# Plugin Start Order Specification

## Purpose

Define contract-ordered plugin startup and isolate dependency admission failures while preserving startup of unrelated installed packages.

## Requirements

### Requirement: Contract Dependency Startup

The runtime SHALL retain Harmos's provider-before-consumer preparation and startup, exact provided/required contract resolution, embedded providers and deterministic ordering based on plugin names. Bundle inspection and extraction SHALL remain inert and need no dependency ordering.

#### Scenario: Consumer Installed First

- **WHEN** a consumer precedes its provider in the installed input
- **THEN** lifecycle preparation and startup complete for the provider before the consumer

### Requirement: Isolated Dependency Refusals

The runtime SHALL refuse installed packages with missing or circular contract requirements using typed admission reasons that name the contract, persist those reasons in existing boot diagnostics, and start unrelated packages normally.

#### Scenario: Circular Requirements

- **WHEN** two plugins require each other's provided contracts
- **THEN** both packages are refused with circular contract reasons and unrelated plugins start

#### Scenario: Missing Provider

- **WHEN** neither another plugin nor a host provider satisfies a requirement
- **THEN** its consumer is refused with a named contract reason and unrelated plugins start
