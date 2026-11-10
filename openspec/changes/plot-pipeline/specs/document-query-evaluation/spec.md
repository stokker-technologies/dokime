## MODIFIED Requirements

### Requirement: Representative Scaling Benchmark

The existing bench pipeline SHALL measure structure latency, card-fill latency
and peak memory for three days, several million samples and 30 plots at increasing
row counts.

#### Scenario: Increasing History

- **WHEN** the benchmark compares equal visible documents over increasing history
- **THEN** it records structure/card-fill times and peak memory without timing assertions
