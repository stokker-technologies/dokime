# Native Desktop Validation Specification

## Purpose
Define equal native desktop validation on Windows and Linux, deterministic driver setup, and independent generated-starter tooling.

## Requirements

### Requirement: Native Desktop Scenario Parity

Ordinary desktop system validation SHALL execute the same scenarios on Windows and Linux, including local Windows validation before a PR update. Platform-specific driver setup SHALL NOT omit ordinary scenarios.

#### Scenario: Local Windows Validation

- **WHEN** the developer runs the applicable local validation on Windows
- **THEN** the native desktop UI journeys execute rather than reporting a platform skip

#### Scenario: Both CI Platforms

- **WHEN** required desktop system jobs run on declared Windows and Linux runners
- **THEN** each platform executes the shared ordinary journey inventory using its native driver

#### Scenario: Another Execution Needs Operator Input

- **WHEN** a different execution requests input while the walkthrough inspects a measured execution
- **THEN** the walkthrough answers through the current cycle overview and restores its measured selection
- **AND** the existing prompt, stream and cycle-completion assertions remain required within their existing deadlines

### Requirement: Deterministic Windows Driver Bootstrap

Windows desktop bootstrap SHALL obtain a driver matching the installed WebView2 runtime and verify its version before executing UI tests.

[Elevated WebView2 hosts ignore environment overrides](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/security#for-an-elevated-host-app-use-appropriate-override-flags); the existing automation opt-in forwards these settings through native environment options instead.

#### Scenario: Missing Driver

- **WHEN** the runner has WebView2 but lacks its matching driver
- **THEN** shared bootstrap provisions and verifies that driver without relying on a manually configured local runner

#### Scenario: Elevated Windows Automation

- **WHEN** the native driver launches the desktop with the existing automation opt-in under an elevated Windows token
- **THEN** the desktop passes the driver's browser arguments and isolated profile directory through native WebView2 environment options
- **AND** ordinary launches retain their existing browser configuration without enabling remote debugging
