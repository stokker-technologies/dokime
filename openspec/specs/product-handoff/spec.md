# Product Handoff Specification

## Purpose
Preserve complete product behavior, branding and native delivery contracts when reconciling divergent handoff branches.

## Requirements

### Requirement: Ship the Intended Branding
The app SHALL use the reconciled current branding assets and sidebar presentation through one authoritative asset ownership chain.

#### Scenario: Native App and Package
- **WHEN** the integrated product builds and packages its GUI and runtime resources
- **THEN** manifests, generated exports, runtime reports and the visible app use the intended surface-aware artwork
- **AND** obsolete asset ownership is removed rather than kept as a competing fallback

### Requirement: Use Native Python Dependency Locking
Python Build and Package SHALL NOT enforce a custom wheel-byte hash comparison. Ordinary uv dependency locking and package version/source checks SHALL remain; no ZIP normalization or replacement custom integrity gate is required.

#### Scenario: Platform-Specific Wheel Metadata
- **WHEN** a built wheel has platform-dependent archive metadata
- **THEN** custom hash comparison does not reject it
- **AND** the delivered tutorial remains usable through ordinary uv dependency installation
