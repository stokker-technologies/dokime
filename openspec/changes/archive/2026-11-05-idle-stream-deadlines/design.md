# Agreed Design

Owner agreement: 2026-11-05. Harmos interprets timeout as maximum silence for operations with a stream and keeps total deadlines for unary operations. Adopt its merged dev revision without external path overrides. Client uploads send no grpc-timeout and rearm operation-policy idle waits whenever transport pulls a frame, including the final response wait. Downloads use their operation policy. TRANSFER becomes 60 seconds. Remove report download 300_000 overrides, server plugin upload idle, desktop artifact idle and desktop install deadline literals; each derives from its operation. Preserve frame, byte and bundle limits, cancellation and response ownership. README reviewed: landing-page claims and examples unaffected.

## Upstream Adoption

The final Harmos revision must retain a traffic idle deadline while buffered gRPC output drains after operation settlement. SDK unknown-Wasm guests retain transfer fences without consulting an unsupported clock; their host owns the deadline. Native channels retain movement timestamps before the execution driver subscribes. Consumer final proof uses the published merged revision, without local path patches.
