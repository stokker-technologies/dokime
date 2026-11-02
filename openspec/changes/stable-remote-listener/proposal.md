# Stable Remote Listener

## Why

Address churn currently restarts the remote listener and fails in-flight RPCs.

## What Changes

Names refresh adds missing currently advertised or configured names and retains covered names when addresses disappear or reorder. New pairings advertise all eligible endpoints, so each current advertised name must be covered. Leaf replacement preserves the CA and reloads in place. Only enabled/address/TLS/auth changes rebind. Heartbeat is cancelled and joined before management closes. Rust/Python safe reads retry UNAVAILABLE once within the existing request deadline; resumable listeners retain their existing policies. Mutations without established request-id deduplication are never retried. Harmos API is unavailable at the current pin: use an untracked local patch to C:/Users/ricca/Work/repositories/github/stokker-technologies/harmos.wt/tlsreload during implementation. The orchestrator must publish Harmos, pin its verified revision and remove the override before final validation; committed Cargo.toml/Cargo.lock stay on the current revision.

## Impact

remote controller/store/server/config/pairing, Rust/Python client retry owners and tests, real-loopback remote stream regression, docs and README impact review. Coupled worktrees: Harmos tlsreload and Dokime remotestable.
