## REMOVED Requirements

### Requirement: Explicit Settings Application

**Reason**: Plugins read saved settings through a host operation with no startup injection.
**Migration**: Save settings and read them on demand; remove applied-versus-saved field marks.

### Requirement: Unified Reload Control

**Reason**: Managed and remote entries replace generation reload and Restart Pending admission.
**Migration**: Manage installed bundles or remote URLs; process restarts interrupt active runs.

### Requirement: Observer Generation Ownership

**Reason**: Stream demand belongs to gRPC connections and execution scope, not generations.
**Migration**: Open/close streams through the host; resolve declared Duration settings at demand admission.
