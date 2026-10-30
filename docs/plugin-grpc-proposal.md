# Proposal: One Managed Native Plugin Protocol

Status: proposed for discussion. This document changes no supported protocol.

## Decision Requested

Should Harmos replace its native plugin wire protocol with a managed gRPC
contract, with Dokime using that contract through a thin adapter?

The intended outcome is one maintained native execution protocol. During a
migration both implementations may exist, but keeping both permanently would
defeat the consolidation. A yes here authorizes designing a small prototype;
it does not commit to a production migration or a performance claim.

## Current Boundaries

Native plugins currently run in supervised child processes. Harmos owns the
framed stdin/stdout wire protocol, separate control channel, registration,
generation lifecycle and operation routing. Dokime adds suites, contributions,
execution records and application admission policy. See Harmos's
`crates/harmos-runtime/src/services/sidecar/` and Dokime's
`crates/dokime-runtime/src/runtime/plugins/`.

Dokime's existing gRPC service serves application clients. Its Rust and Python
clients do not establish a requirement for non-Rust plugin authoring, and its
service is not already a replacement for the native plugin protocol. Desktop
Tauri IPC connects the WebView to its local backend and serves a separate role.

## Proposed Ownership

Harmos would own a versioned protobuf service for managed native plugins:

- Host-owned startup establishes a private local endpoint and authenticates the
  child and its generation. Address discovery must not expose an unauthenticated
  listener to other local processes.
- A plugin endpoint exposes initialization, operation invocation, observation,
  orderly shutdown and typed failures. Streaming invocations retain input,
  output and a final result.
- A host endpoint supplies the callbacks a plugin needs, including child calls,
  evidence publication and access to admitted resources. The host keeps grant
  enforcement and rejects calls from retired generations.
- Supervision keeps ownership of child processes, deadlines, cancellation,
  draining, cleanup and exit diagnostics. A transport library does not replace
  these responsibilities.

Dokime would describe its application-specific contributions and execution
contracts using those services. Its adapter would not implement a second
process manager or another invocation protocol. Definitions-only consumers
and plugin authors should not depend on the full Dokime application host.

Start with local managed children. Remote plugin placement and non-Rust plugin
SDKs are separate requirements that need demonstrated demand. gRPC can run over
local sockets where supported; choosing TCP is not part of this proposal.

## Conditions For Proceeding

A prototype should replace one real path, preferably a small native plugin,
before changing the SDK. It must demonstrate:

1. The same typed requests, responses, signals and error context cross the
   boundary, with explicit protocol-version admission.
2. Cancellation remains responsive while ordinary traffic is backpressured.
   Cooperative cancellation retains the final result; forced termination still
   owns and settles children and evidence delivery.
3. Startup failure, unavailable dependencies, reload, stale-generation calls
   and shutdown preserve the current ownership and refusal behavior.
4. Bounded message sizes, queues and stream flow control preserve memory bounds.
5. A matched workload measures startup, small-call latency, sustained Arrow
   transfer, peak memory and cancellation latency against the existing native
   transport. There is no current evidence that localhost performance is equal.
6. The complete replacement removes enough wire and adapter code to justify
   generated services, dependencies and operational complexity.

If the prototype fails those conditions, keep the existing native protocol.
There is no benefit in adopting gRPC solely to add another supported mode.

## Trust And Migration

Native plugins execute with the child's operating-system privileges. gRPC does
not provide Wasmtime's memory isolation or restricted host imports. Retiring
guest execution is an independent decision to support trusted native plugins
for now; this protocol cannot substitute for a sandbox.

If accepted, implement reusable contracts and supervision in Harmos first,
then prepare a dependent Dokime change. Pin matching revisions during the
transition, migrate fixtures and authoring tools, and remove the old native
carrier after parity is demonstrated. Keep application-client gRPC and desktop
IPC governed by their own existing contracts.

## Review Questions

- Is a single managed gRPC plugin protocol an appropriate direction to prototype?
- Are non-Rust authors or remotely placed plugins actual near-term requirements?
- Which measured costs would make retaining the current native protocol preferable?
