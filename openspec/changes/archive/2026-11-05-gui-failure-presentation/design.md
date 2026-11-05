# GUI Failure Presentation Design

## Agreed Direction

```ts
type NotificationFailure = {
  id: string;
  title: string;
  message: string;
  stack?: string;
  // Existing root-first causes, field problems, IDs and redacted diagnostics.
};
failureOf(error: unknown, title: string): NotificationFailure;
reportFailure(failure: NotificationFailure, context: string): void;
```

Canonical native/RPC adapters retain ownership of decoding backend errors. The stem consumes their diagnostic projection without discarding it. Object identity caches normalization; a serializable identity survives copying to notification history. A new Error on a later retry is a new occurrence, while rerenders, strict effects and multiple leaves sharing an occurrence never duplicate console output. Strings use the originating presentation key when an occurrence owner provides one; equal messages from distinct action attempts remain distinct failures.

One routing module defines destinations. Views render ErrorCard and report to console after commit. Actions and background failures publish the same record through the existing notification store/toaster and console reporter. Persistence failures report without recursively trying to persist themselves. ErrorCard Copy serializes the whole record; the notification details dialog retains notification metadata and exposes the same technical record, including stack. No leaf re-extracts messages.

## Source Inventory And Ownership

One agent owns this integration branch. Implement in dependency order: `src/lib/notification-details.ts`, new routing/reporter module, notifications and shared ErrorCard/technical details; then cycle/document, settings/library/explore, startup/session/streams/plots and media/artifact consumers; then owning tests, `docs/gui.md`, `docs/product-guide.md` and affected README prose. Canonical RPC/native projections are adjusted only to provide partial diagnostics rather than constructing leaf records. Remove message helpers and failure-only PaneNotice paths. Retain loading, empty, cancellation and busy behavior. Tests live in existing domain workflow files where possible. No unresolved design decisions.

## Acceptance And Validation

Deterministic tests cover data preservation, identity and routing, console exactly once, ErrorCard record/retry/Copy and collapsed details. Never assert authored prose or colors. Run focused TypeScript stories, GUI format/lint/generated type checks and strict OpenSpec validation. Exercise a failing desktop view through the existing task if supported. After source closure, push the candidate and run the default all-platform CI simulation once beside hosted CI. Archive through the Hub lifecycle and deliver one immediately squash-merged PR into dev; clean the owned worktree and branch.


## Required Generated Inputs

Fresh dev carried stale Python/TypeScript operation deadlines. The canonical
`prepare:bindings` task refreshes those outputs from the existing operation policies;
this change does not edit those policies. The shared frontend changes also require
the task-generated embedded report refresh. These generated producers and consumers
are part of the implementation inventory and are never edited by hand.
