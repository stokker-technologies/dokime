# Idle Stream Deadlines

## Why

Transfers must time out on stalled progress, never total duration.

## What Changes

Adopt the merged Harmos change, use operation policy for upload/download idle waits, lower TRANSFER silence to 60 seconds and remove four duplicate literal deadlines. Size caps stay unchanged.

## Impact

Client transport and input frames; server plugin install; desktop artifact upload and plugin install; operation policy; product and plugin guides. Harmos at ../harmos.wt/idle-deadlines delivers first.
