# Screenshots Task

## Why

The product screenshots are captured by hand. Nothing starts the application, drives it to a known
state and takes the pictures, so they go stale and differ in size and content from one capture to
the next. A tooling module that runs one command per scene exists but declares no scenes and no
task calls it.

## What Changes

One public task captures every screenshot from the real desktop application:

```sh
mise run screenshots                    # every scene, light and dark
mise run screenshots -- --scene cycle   # one scene, light and dark
mise run screenshots -- --check         # validate the committed files only
```

It is one linear script on the desktop story harness:

```text
start an owned backend on a fresh root, placeholder identity, virtual clock
load the bench plugin
run the fixed journey once
for each scene: for light, then dark: set theme, viewport and density, wait for the scene's
  ready condition, capture the webview without window chrome
write docs/screenshots/<scene>-<theme>.png and docs/screenshots/manifest.json
```

- State is built by running a real journey on a fresh root, not from a checked-in database.
- The journey runs on the virtual clock, so timestamps and plotted data repeat between captures.
- Every scene is captured in the light and the dark theme: `<scene>-light.png`, `<scene>-dark.png`.
- The owner runs the task on a workstation and commits the images as a chore. CI does not capture
  or publish them; they are not release assets.
- `--check` verifies presence, format, dimensions, size and the manifest. It compares no pixels.

Scenes, chosen to show the product at its best, in capture order:

| Scene | Viewport | Shows |
| --- | --- | --- |
| `cycle` | 1600x1000 | A case running: live plots with reference bands, step progress, run controls |
| `plots` | 1600x1000 | Several plots of a finished execution, one zoomed, with statistics |
| `report` | 1600x1000 | The generated report document with its plots and results table |
| `history` | 1600x1000 | A recorded cycle: executions with outcomes and one inspected checkpoint |
| `library` | 1600x1000 | A suite expanded, a template selected with its step timeline |
| `cycles` | 1600x1000 | The cycles home with finished and active cycles |
| `touch` | 1920x1200 | The running cycle at touch density |

The settings and parameter-dialog pictures are dropped. The README embeds the light images.

## What It Removes

The command-per-scene capture module with its `surface-state` naming and its scene configuration,
the hand-capture instructions, and the old file names (`cycle.png`, `touch-cycle.png` and the
rest). The site that shows the screenshots reads the new names under `docs/screenshots/`.

## Impact

`tools/python` screenshots module, the desktop story harness, `mise.toml`, `docs/screenshots/`,
README, docs/tooling.md and the product guide. Depends on the host clock arriving in Dokime for
repeatable content; until then the task works with real time and content varies.
