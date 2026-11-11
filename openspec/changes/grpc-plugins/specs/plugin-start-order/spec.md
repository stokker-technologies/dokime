## REMOVED Requirements

### Requirement: Contract Dependency Startup

**Reason**: Plugins connect independently over gRPC; there is no dependency startup graph.
**Migration**: Declare typed required operations; compare fingerprints when providers connect.

### Requirement: Isolated Dependency Refusals

**Reason**: Missing providers refuse calls rather than plugin startup; cycles are not an admission graph.
**Migration**: Preserve each entry's availability and report the named missing provider at call time.
