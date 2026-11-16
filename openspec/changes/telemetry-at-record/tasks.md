# Telemetry at Record Tasks

Status: post-release; implementation follows the first stable release and waits for owner scheduling.

## 1. Expressions

- [ ] 1.1 Add the `at` transform (`record_type`, optional `within_s`) to `DataTransform` and the `DataExpression` builders `at` and `within_s`; reject non-telemetry sources at validation; unit tests for the wire form.

## 2. Evaluation

- [x] 2.1 Superseded by [operation-boundary tasks 8.1–8.2](../operation-boundary/tasks.md): ordered merge belongs to Harmos query transforms, with no private document-evaluator merge kernel.
- [ ] 2.2 Refuse `at` on repeat selectors with the `time_between` diagnostic shape; test.

- [ ] 2.3 After the first stable release, bind the at-or-before and nearest-within expression to Harmos time-merge operations; prove live/sealed parity, missing rows, ties and tolerance edges with deterministic records and samples. operation-boundary owns the generic merge; this change owns these expression semantics.

## 3. Surfaces

- [ ] 3.1 Python SDK builders and stubs; authoring skill and product guide examples (histogram of a stream value at an event); the shipped SDK examples gain one example plot.

## 4. Validation

- [ ] 4.1 Focused domain, runtime document and Python tests; `mise run lint -- --language rust` and focused Python checks through project tasks.
