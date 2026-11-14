# query-snapshots Specification

## Purpose
Define consistent evidence and metadata snapshot cuts, bounded cursor paging and the continuation lifecycle.

## Requirements

### Requirement: Bounded Execution and Streaming Consumption
Server readers SHALL bound decoded batches, rows, accumulators, encoding and loans by configuration, independently of history/page count. Python SHALL pull pages on demand with bounded prefetch. Oversized rows/schemas and work exhaustion SHALL fail explicitly. Statistics SHALL never be partial; failed row iteration SHALL remain visibly incomplete.

#### Scenario: Encoding and Work Budgets
- **WHEN** a page admits data for encoding or hits its scan/time ceiling
- **THEN** schema and metadata count against budgets, and oversized rows/schemas fail without splitting, omission or infinite continuation
- **AND** work exhaustion raises a typed failure, not a successful partial statistic

#### Scenario: Slow Consumer and Oversized Row
- **WHEN** a slow reader consumes a long history and later reaches a row larger than the configured page-byte ceiling
- **THEN** server/client buffers stay bounded and the oversized row produces ResourceExhausted
- **AND** no successful complete collection is returned
