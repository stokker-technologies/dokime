## ADDED Requirements

### Requirement: Typed CPU Load Hold
The host plugin SHALL provide Public `host.cpu.load.hold` accepting one `LoadHold` request with a validated `LoadTarget` and optional positive thread count.

#### Scenario: Valid Targets
- **WHEN** a caller supplies finite percent in `(0,100]` or finite FLOP/s greater than zero
- **THEN** the request is accepted and an absent worker count uses logical cores

#### Scenario: Invalid Request
- **WHEN** a target is nonfinite, zero, negative, above 100 percent or threads is zero
- **THEN** the operation refuses before starting workers

### Requirement: Counted Closed-Loop Work
The generator SHALL use exactly counted floating-point work and a discrete PI controller with clamped integration and duty in `[0,1]`.

#### Scenario: Reachable Target
- **WHEN** a simulated plant can reach the selected target
- **THEN** measured feedback converges within the focused test tolerance

#### Scenario: Saturation
- **WHEN** external host load exceeds a percent target or requested throughput exceeds capacity
- **THEN** duty saturates without integrator windup and recovers when the target becomes reachable

### Requirement: Shared CPU Sampling And Capacity
The host SHALL reuse its startup calibration and one shared CPU sampling source, without consumer-driven refreshes shortening CPU delta windows.

#### Scenario: Concurrent Consumers
- **WHEN** system telemetry and the controller read concurrently between sampler ticks
- **THEN** both observe the same cached utilization and reads do not change the sampling baseline

### Requirement: Controller Evidence
The host SHALL publish each control period on `host.cpu.load.control` with target, measured value, duty and generator FLOP/s, structured units and `Label [unit]` labels.

#### Scenario: Two Measurement Modes
- **WHEN** a period completes in percent or FLOP/s mode
- **THEN** evidence identifies the mode and represents values in the corresponding structured units

### Requirement: Cancellation Restores CPU Use
Cancelling the operation handle SHALL promptly stop all workers and return `LoadSummary` with target, achieved mean, RMS error and duration in seconds.

#### Scenario: Cancel And Join
- **WHEN** a caller starts a hold, waits and cancels its handle
- **THEN** all workers stop and joining returns the summary within the configured settlement budget

#### Scenario: Execution Cleanup
- **WHEN** execution cleanup finds an open handle
- **THEN** it cancels and settles the workload before sealing evidence, without an effect-journal inverse

### Requirement: Configuration And Example
One documented configuration SHALL own controller tuning and timing; an OBC suite SHALL demonstrate start, hold and cancel.

#### Scenario: Configured Control
- **WHEN** valid configuration changes the period, gains or slice
- **THEN** workers and controller use that configuration and operation policy supplies deadlines
