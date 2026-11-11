# Screenshots

Regenerate the published product images from the real desktop application:

```sh
mise run screenshots
mise run screenshots -- --scene cycle
mise run screenshots -- --check
```

Capture requires the development desktop, CLI and bench plugin artifacts from
`mise run build`, and native automation from `mise run prepare:desktop`.
The task runs one fixed journey on fresh, temporary backend storage, with the
`plugin-obc` AC testbench and the placeholder `dokime-bench` identity. It owns and
stops its backend, desktop and report browser. Existing application data is untouched.
Until the host virtual clock is adopted, timestamps and measurements use real time.

For each scene it selects light and then dark mode, sets the webview viewport and
density, waits for populated content and captures without operating-system chrome.
The inspection prompt stays pending until the running views are captured; the
journey answers it before finishing and sealing the execution. Finished plots wait
for zoom statistics, and touch plots enter the viewport before their readiness wait.
The manifest records scene, theme, viewport, density, source revision and capture time.
A selected scene replaces only its two images and metadata and builds its prerequisites.
Library capture needs no execution; running scenes stop their execution after capture.
Ready scenes and their metadata are retained if a later product operation fails;
the task still fails and retains its diagnostics under `target/screenshots/`.
`--check` starts nothing and requires all fourteen files,
their PNG structure, dimensions, 4 MiB size cap and manifest entries. It compares no pixels.
Review every regenerated image before committing it. CI does not capture or publish
screenshots, and screenshots are not release assets.

| Scene | Viewport | Content |
| --- | --- | --- |
| `cycle` | 1600×1000 | Running bench case, live measurements with reference bands and controls |
| `plots` | 1600×1000 | Finished execution plots, held zoom and trace statistics |
| `report` | 1600×1000 | Generated HTML report, rendered in the desktop webview |
| `history` | 1600×1000 | Recorded cycle and an inspected execution checkpoint |
| `library` | 1600×1000 | Expanded suite and selected template with its registered procedure steps |
| `cycles` | 1600×1000 | Finished and active cycles on the home surface |
| `touch` | 1920×1200 | Running case at touch density |

Each captured scene has `<scene>-light.png` and `<scene>-dark.png`. The README embeds
only available light images. The public product site must use these names under
`docs/screenshots/`. The manifest lists the images actually captured; `--check`
reports missing scenes until the complete set is available.

The published set contains all seven scenes in both themes. Captures require a
successful sealed execution for finished scenes and successful backend teardown;
an image captured before a later failure never makes the task pass.
