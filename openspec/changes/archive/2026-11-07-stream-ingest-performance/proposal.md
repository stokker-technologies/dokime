# Stream Ingest Performance

## Why

Fast Stream Support requires measured kHz and MiB/s throughput from a native plugin through sidecar publication to durable storage. Current per-row projection, rank indexing and checkpoint serialization multiply work and bytes.

## What Changes

Implement the owner's requested batching and shared bounded field summaries, preserve raw rows, ordering, evidence cuts and within-cap exact inputs. Audit transport, writer, ranks and previews with measured scalar/wide and 0/1/64-plot cases. Keep percentile results exact through report replay when they exceed bounded retained state. Baseline and repeat measurements on this machine, focused tests and check:rust are required; no full simulation. Deliver one PR to dev and squash immediately, then clean the owned worktree.

## Agreement

Scope and direction are authorized by the user's Stream Ingest task. This is a separate change; it does not implement the other open proposals.
