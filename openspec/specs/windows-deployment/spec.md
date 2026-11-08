# windows-deployment Specification

## Purpose
TBD - created by archiving change windows-signing. Update Purpose after archive.

## Requirements

### Requirement: Optional Verified Windows Signing

Packaging SHALL sign staged Windows executables and the final installer before computing inventory digests and updater minisign signatures when signing credentials are configured. It SHALL require RFC 3161 timestamps and valid Authenticode verification. It SHALL preserve the existing manifest schema.

#### Scenario: No Certificate Configured

- **WHEN** neither Azure nor PFX credentials are configured
- **THEN** packaging skips Authenticode with a notice and continues without contacting a signing service

#### Scenario: Signing Configured

- **WHEN** one complete signing configuration is available
- **THEN** executable signing and verification finish before payload hashing, and installer signing and verification finish before minisign

#### Scenario: Invalid Configuration or Signature

- **WHEN** credentials are partial or conflicting, signing fails, or verification is invalid
- **THEN** packaging fails rather than publishing unsigned output as signed

### Requirement: Explicit Installation Scope

The installer SHALL default to the current user and SHALL support `/ALLUSERS` for machine-wide deployment without changing the manifest contract. Machine scope SHALL use Program Files, machine registration, shared shortcuts and machine PATH. Application startup SHALL not require elevation.

#### Scenario: Machine-Wide Installation

- **WHEN** an elevated deployment process invokes setup with `/ALLUSERS /S`
- **THEN** installation uses machine scope and the matching uninstaller retains that scope

#### Scenario: Missing Privilege

- **WHEN** an unelevated process requests machine scope
- **THEN** setup fails with exit code 740 before changing installed files

#### Scenario: Existing Per-User Recipe

- **WHEN** setup runs without a scope switch or with `/CURRENTUSER`
- **THEN** the existing per-user installation behavior is retained
