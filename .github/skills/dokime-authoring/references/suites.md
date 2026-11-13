# Suites

Materialization resolves a template's parameters into a case without side effects.
The host initializes execution-local resources, orders steps and criteria, handles
policy retries/cancellation and runs required cleanup. Samples, records and prompts
retain scoped evidence; evaluated and declared outcomes stay distinct.

Use the [Rust SDK Guide](../../../../docs/rust-sdk.md) or the other SDK's own guide
for current builders and signatures. The [Execution Host Guide](../../../../docs/execution-host.md)
owns current serving behavior; [Architecture](../../../../docs/architecture.md)
and the locked operation grammar own invariants and remaining proof obligations.
