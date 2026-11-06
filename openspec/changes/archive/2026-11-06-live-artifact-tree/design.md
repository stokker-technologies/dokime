# Live Artifact Tree Design

## Agreed Scope

The owner request authorizes these behavior changes and deterministic scenarios. Artifact UUIDs remain identities; paths no longer contain them. Native uploads reserve a validated execution-relative path on admission. Same-path collisions choose the first free `.N` suffix before the final extension, starting at 2. Rotation naming remains `.segmentN` before collision allocation. Existing data is never rewritten or reset.

## Ownership And Lifecycle

Artifact uploads reuse the native output file and original production follower, preserving bounded Harmos reads, preview limits, cancellation and terminal identity. Stream watches create an upload only after capturing bytes or discovering a new empty source, append spool ranges starting at the upload's authoritative committed length (including after cancellation/retry), and finish that same upload at sealing. An FTP resume fallback seals any already visible Stream prefix before starting a separately named Mirror segment. Mirror watches keep replacement spools and upload their successful final copy. The GUI independently coalesces operation/state changes into execution-file invalidation; unchanged heartbeat frames do not refresh files. No viewer queue or viewer delivery acknowledgement enters the producer path.

## Compatibility And Reports

Untouched Stream baselines have no artifact or sealed event. New empty files remain valid captured evidence. Watch completion counts derive from sealed records. Saved descriptors, report links and execution-file inventory use the allocated path. Native host and harness consume the same semantics. FTP regression fixtures await the next observable watch event and arming before writes: recoverable empty passes are part of the existing watch contract, not proof that a baseline was captured. No migrations or legacy path adapters are added.
