# Integrate Product Handoff

## Why

The CI migration started from an older dev tree and omitted the newer drop and specification-completion branches. The visible old logo exposed a broader product-baseline mismatch. Successful validation of that older tree cannot prove delivery of the requested handoff.

## What Changes

- Combine the intended product work from both existing handoff branches with the current native CI migration in the existing integration worktree.
- Reconcile overlapping product contracts and remove superseded implementations rather than retaining competing branding, tooling or API owners.
- Preserve the approved CI, packaging, Python client, executable publication and native Windows UI repairs.
- Validate the complete combined candidate, deliver through the existing protected PR into dev, and retain the authorized release prerequisites.

## Impact

The product owner covers the full repository: runtime/domain/server/CLI libraries, generated clients and bindings, GUI and branding, plugins and generated starters, examples, tests, packaging, documentation and maintained specifications. Hub changes remain in the existing Hub fixes worktree. Benchmarks remain optional; no required correctness gate is waived.
