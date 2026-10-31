# Installation Specification

## Purpose
Keep Windows installation and uninstallation available to standard users, with firewall permission managed explicitly by the exporting desktop.

## Requirements

### Requirement: Standard-User Installation
Default Windows installation and uninstallation SHALL perform no firewall operations or elevation and SHALL request user-level execution.

#### Scenario: Locked-Down PC
- **WHEN** a standard user installs or uninstalls the default distribution
- **THEN** no firewall administrator permission is required

### Requirement: Optional Host Firewall Action
The exporting desktop SHALL inspect inbound Windows firewall access for a running non-loopback listener, offer an explicitly elevated allow action when needed, and display non-fatal cancellation or failure. Labels SHALL derive from the organization. Outbound clients SHALL have no firewall action.

#### Scenario: Private Listener Is Blocked
- **WHEN** a local non-loopback listener lacks effective inbound allowance
- **THEN** settings explain that an administrator is needed and offer Allow In Firewall

#### Scenario: Disable Remote Access
- **WHEN** the app disables remote access
- **THEN** owned-rule removal is best effort without prompting for elevation and skipped removal is reported
