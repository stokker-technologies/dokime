# Unified Operation Invocation

The owner decisions recorded in Harmos's paired change apply unchanged. SDK `Invocation<H: OperationHost, O, E = ()>` owns request, optional retry and endpoints; host implementations own routing and errors. Templates expose no lane. Macro-declared policy is the sole default and no-option calls retain it. Template retry values come from configuration, parameters or settings.

`StepOperationHost` retains stage-specific ExecutionScope, test harness routing, WatchFile readiness tickets, process-local built-ins and step handle ownership. Template `operation` retains Called authorization, including the existing built-in exemptions. Criterion and template observers use the same builder and Response contract. The no-argument Harmos handler call metadata accessor remains.

Verified producer findings: transient failures historically retry with unknown replay safety; ambiguous failures require explicit replay safety. Preserve declaration semantics but refuse call-site overrides that enable retries with unknown safety. Host admission remains authoritative for deadline and attempt evidence. Local harness failures retain their classification and policy; cancellation remains scoped to the running stage.

Harmos lands first. Pin the resulting full merged SHA, remove all local patches, resolve Cargo.lock against the published graph and remove gRPC config web/allowed_origins if the pin crosses Harmos #110. Refresh bindings if wire types affect generated artifacts. Never run stkr project update. Authoring references under .agents, .claude and .github remain identical, with only affected .claude/.github generated.lock digests updated.
