# Plot Marker Legend and Sparse Points

## Why

Event markers need the plot legend's labels, window counts and visibility
controls. Sparse traces can lose every visible sample despite correct statistics.

## Agreed Scope

The owner's task authorizes this change and its immediate squash delivery to dev.
Add one legend row per event record type and prompt lifecycle kind, after traces.
Use matching glyphs and theme-aware plot palette colors; keep hover labels.
Replace the prompt control with rows. Keep visibility local to the plot like
trace toggles, including frozen reports. Marker rows have window counts and empty
statistics. Plots without markers retain their existing presentation.

Reproduce alternating sparse traces and nullable faceted coordinates with real
canvas rendering before fixes. Cover point styles, every line style, gaps and
window reduction; lone samples must remain visible. Update docs/gui.md and run
only targeted TypeScript tests, GUI formatting, lint and type checks. Do not run
a full simulation. Deliver one PR to dev and remove the owned worktree and branch.

## Ownership and Impact

CartesianPlot owns visibility, viewport counts and canvas rendering.
StatsSummaryTable owns legend glyph presentation; statistics types carry row
presentation. Plot nominal tests and a focused canvas harness own regression
proof. Shared live/report rendering changes together; generated report resources
are refreshed through the existing task. README claims remain accurate.
