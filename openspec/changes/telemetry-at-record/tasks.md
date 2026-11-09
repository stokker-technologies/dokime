# Telemetry at Record Tasks

Status: proposed; deferred until the owner schedules it.

## 1. Expressions

- [ ] 1.1 Add the `at` transform (`record_type`, optional `within_s`) to `DataTransform` and the `DataExpression` builders `at` and `within_s`; reject non-telemetry sources at validation; unit tests for the wire form.

## 2. Evaluation

- [ ] 2.1 Implement the ordered merge in the document evaluator for live and sealed evidence with the at-or-before and nearest-within rules; deterministic tests with constructed streams and records, including no qualifying row, ties and tolerance edges.
- [ ] 2.2 Refuse `at` on repeat selectors with the `time_between` diagnostic shape; test.

## 3. Surfaces

- [ ] 3.1 Python SDK builders and stubs; authoring skill and product guide examples (histogram of a stream value at an event); the starter template gains one example plot.

## 4. Validation

- [ ] 4.1 Focused domain, runtime document and Python tests; `mise run lint:rust`; project check.
