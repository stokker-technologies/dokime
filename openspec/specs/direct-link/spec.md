# Direct Link Specification

## Purpose

Make remote access work between two PCs connected only by an Ethernet cable,
with visible addresses and optional firewall access for local-subnet peers.

## Requirements

### Requirement: Direct Cable Endpoints

The server SHALL include physical IPv4 link-local addresses after routable
addresses, excluding virtual adapters and IPv6 link-local. Listener status SHALL
provide the same ordered export endpoints with adapter names. Exported TLS
endpoints SHALL be covered by a leaf signed by the persistent CA.

#### Scenario: Cable Without A Gateway

- **WHEN** a physical cable adapter acquires an IPv4 link-local address
- **THEN** its endpoint and adapter name appear before export, the exported file includes it, and TLS validates it

#### Scenario: Addresses Change

- **WHEN** the detected address changes
- **THEN** the leaf is reissued under the same CA and the preview reflects current detection

### Requirement: Optional Served-Network Firewall Access

The desktop SHALL detect active profiles for adapters served by the listener,
explain their names and profiles, and offer an optional elevated action creating
an inbound program/TCP-port rule for those profiles with RemoteAddress LocalSubnet.
Installers SHALL NOT change the firewall.

#### Scenario: Public Unidentified Cable

- **WHEN** the served cable is Public and the operator chooses Allow In Firewall
- **THEN** the rule permits the listener on that profile for local-subnet peers

#### Scenario: Specific Private Bind

- **WHEN** only a Private adapter is served and another adapter is Public
- **THEN** the action targets the served Private profile and the notice identifies that adapter

#### Scenario: Cancelled Elevation

- **WHEN** the operator cancels administrator approval
- **THEN** no rule changes and the application remains usable
