# Tasks

## Implementation

- [x] 1.1 Pin all Harmos Git packages and lockfiles to its merged revision.
- [x] 1.2 Implement policy-driven upload idle progress and final response waits without total grpc-timeout; use policy for downloads.
- [x] 1.3 Set TRANSFER silence to 60 seconds and replace the four literal deadline owners in client, server and desktop.
- [x] 1.4 Add deterministic artifact upload and plugin install progress/stall end-to-end stories and policy-driven download regression proof.
- [x] 1.5 Update docs/product-guide.md and docs/plugins.md; review README impact.

## Validation And Delivery

- [x] 2.1 Complete focused client/server/desktop proof and forbidden-term diff review.
- [x] 2.2 Review the complete candidate against the existing delivery authorization.

## Workflow Follow-Up

After focused proof, publish the candidate, launch default all-platform local CI alongside PR checks, and squash into dev immediately. Report actual local results, retire the owned worktrees and branches, and close the native goal. These delivery steps remain open until performed.
