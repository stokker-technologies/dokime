# Plot Pipeline

Status: direction agreed with the owner on 2026-11-10; implementation has not
started and waits for the owner's go. Independent of `grpc-plugins`. Nothing
here is measured yet; the first task records a baseline.

## Why

Live plots and multi-day runs are core. The renderer and storage are already
sound: the same canvas renderer the leading dashboards use, compressed
columnar files with an index, recent and saved data merged at query time,
and stored previews for whole-history plots. What remains is repeated work.
A plot over a time window re-reads and re-reduces raw samples on every
update, every request asks for a fixed 2,000 points whatever the panel
width, zooming does not fetch more detail, each live update sends a full
replacement, and the same data travels in two representations.

## What Changes

1. **Time-bucket envelopes for windowed plots.** The existing preview
   mechanism is extended so a time window is answered from stored buckets
   holding first, last, minimum, maximum and running moments. Closed buckets
   are reused; only the newest edge is computed from live data. Raw samples
   stay the evidence.
2. **The request carries the viewport and the panel width.** The backend
   buckets by time to that width and returns more detail when the user
   zooms. Minimum and maximum are taken per time bucket, not per row count.
3. **Live updates are deltas.** A watch sends new and changed buckets after
   its opening snapshot. Exact statistics such as the median are delivered on
   their own schedule and never delay a redraw. A late timestamp replaces the
   bucket it falls in.
4. **One columnar frame on the wire,** with a shared time column, in place of
   point series plus arrays.
5. **Downsampling kernels come from `tsdownsample`** in place of
   hand-maintained minimum/maximum, M4 and largest-triangle code.
6. **FFT moves into Dokime** from Harmos, which no longer offers signal
   processing.

## Memory

The requirement that working memory stays flat as retained rows grow is
removed. Using more memory while a report or plot is produced is
acceptable when the result is correct and fast. At today's rates an
execution's samples fit in memory many times over; signals in the
kilohertz range are the case that could change this, and peak memory
stays measured so that case is visible when it comes.

## Not in This Change

- Replacing the temporary SQLite staging used for plots with expressions,
  grouping or markers. It sorts, groups and joins raw rows with bounded
  memory by spilling to disk. Proposed: once stored buckets answer most
  plots, measure what these plots need at the target scale. If their rows
  fit in memory with a stated bound, compute on Arrow columns in memory and
  delete the staging; a plot beyond the bound is refused with a message to
  narrow its window. If they do not fit, a query engine over Arrow
  (DataFusion) replaces the staging and the custom projection and filter
  code, tried in its own worktree and adopted only if build times stay
  acceptable.
- A separate storage tier for old data.

## Impact

- Plot query, preview storage, the plot watch, the plot wire messages and the
  interface's plot data path.
- Supersedes the deferred `ingestion-storage-redesign`, deleted with this
  change: its baseline described storage as it was before samples moved to
  Arrow files.
