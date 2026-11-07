## REMOVED Requirements

### Requirement: Observable and Faithful File Capture

**Reason**: Readiness, selection, rotation and source waiting are redefined by the capture operations. `Selection::New` and opt-in source waiting are removed.

**Migration**: Use the `file-capture` capability: `StartCapture` returns at the baseline, a `Newest` source replaces `Latest`, and a `Tree` source over an existing folder replaces waiting for a later file. FTP resume behavior is retained under `FTP Behind a Feature`.
