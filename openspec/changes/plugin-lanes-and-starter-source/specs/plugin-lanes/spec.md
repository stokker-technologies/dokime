## ADDED Requirements

### Requirement: Authors Declare Lanes

Every SDK SHALL let an author declare a lane with a name and a capacity that is
serial, a fixed concurrent width or unbounded, and assign an operation to it.
The SDK SHALL describe lanes to the host and SHALL NOT enforce them itself.

```python
instrument = Lane("instrument", serial=True)

@operation("example.measure", lane=instrument)
async def measure(ctx: Context, request: Target) -> Reading: ...
```

#### Scenario: Serial Operations

- **WHEN** two operations of a plugin are assigned to one serial lane and invoked together
- **THEN** the host runs them one after the other

#### Scenario: No Lane Named

- **WHEN** an operation names no lane
- **THEN** it runs without waiting for the plugin's other operations

### Requirement: Lane Mistakes Are Refused at Declaration

An SDK SHALL refuse, when the plugin is constructed, a lane with an empty name
or zero capacity, two lanes with one name, and an operation assigned to a lane
the plugin does not declare, with an actionable error naming the lane.

#### Scenario: Duplicate Lane Name

- **WHEN** a plugin declares two lanes with the same name
- **THEN** constructing the plugin fails and names the lane
