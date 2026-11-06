# Unified Operation Invocation

The owner decisions recorded in Harmos's paired change apply unchanged. SDK `Invocation<H: OperationHost, O, E = ()>` owns request, optional retry and endpoints; host implementations own routing and errors. Templates expose no lane. Macro-declared policy is the sole default and no-option calls retain it. Template retry values come from configuration, parameters or settings.

`StepOperationHost` retains stage-specific ExecutionScope, test harness routing, WatchFile readiness tickets, process-local built-ins and step handle ownership. Template `operation` retains Called authorization, including the existing built-in exemptions. Criterion and template observers use the same builder and Response contract. The no-argument Harmos handler call metadata accessor remains.

Verified producer findings: transient failures historically retry with unknown replay safety; ambiguous failures require explicit replay safety. Preserve declaration semantics but refuse call-site overrides that enable retries with unknown safety. Host admission remains authoritative for deadline and attempt evidence. Local harness failures retain their classification and policy; cancellation remains scoped to the running stage.

Harmos lands first. Pin the resulting full merged SHA, remove all local patches, resolve Cargo.lock against the published graph and remove gRPC config web/allowed_origins if the pin crosses Harmos #110. Refresh bindings if wire types affect generated artifacts. Never run stkr project update. Authoring references under .agents, .claude and .github remain identical, with only affected .claude/.github generated.lock digests updated.

## Attempt Evidence Ownership

Host execution dispatch attaches a process-local AttemptOwner beside the portable ExecutionScope. Carrier child routing retains local context while overlaying portable values. The middleware validates scope changes before execution and records attempts only under the retained owner. Child providers can keep the originating execution's metadata without being mistaken for its direct author; portable execution ids alone cannot append evidence. Local harness errors carry no retry classification, so their String errors remain permanent; replay-safe attempt timeouts are ambiguous and may retry under the declared or configured policy.

## Surface Errors

StepOperationHost and StepInstance use Dokime Error, converting the underlying sidecar CallError once inside the host/instance boundary. Template error enums retain their existing From<dokime::Error> conversion; await still returns the host-associated error directly, without another surface future or a transitive-conversion requirement.

## Approved Authorization

Use `pub trait Called<C, Proof = Declared> {}` with an inferred proof parameter on `operation`. Declared operations use the default proof and unconditional WatchFile/RunProcess built-ins use BuiltIn. Keep `ctx.operation(request)` unchanged. Reject unconditional built-ins in a template requires catalog with a compile-time diagnostic, including nested catalogs.

## Delivery and Retirement

The owner accepted the existing repository-wide convention baseline (about 1,780 findings across 567 files) for this scoped delivery. Leave the baseline for its later convention sweep, preserve it outside the invocation migration and disclose it in the PR.

Archive this change with `stkr specification finish unified-operation-invocation` before the final candidate commit so its contracts travel in the feature PR. After the requested focused proofs, starter task, Rust checks and repository checks, push the complete candidate, verify its remote SHA, create one ready PR into `dev` and squash immediately without waiting for CI. Use `gh pr merge --squash --subject "refactor!: unify operation invocation (#N)"` with the actual PR number. No full simulation is requested. Preserve the validation results in the review description, clean the owned Cargo outputs and retire the exact integrated `dokime.wt/invocation` and `harmos.wt/invocation` worktrees and their task branches. Do not modify the private plugin repository.
