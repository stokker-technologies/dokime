# Screenshots

These images are the product screenshots for Dokime. The public product page at
[stokkertechnologies.ch](https://stokkertechnologies.ch) fetches them straight
from this folder over GitLab's raw `main` URLs, and the repository README embeds
them by relative path, so treat them as published assets: the file names are the
contract, and replacing a file republishes it everywhere at once.

```text
https://github.com/stokker-technologies/dokime/raw/refs/heads/main/screenshots/<file>
```

Every image is captured from the real desktop application driving a real backend
with the `obc-plugin` AC testbench plugin loaded, in
the application's own default theme, with no operating-system window
chrome. The backend runs under a placeholder identity (`dokime-bench` /
`Bench Operator`, no git configuration), so the captured traceability panels
carry no personal account name, address or machine name.

## The Images

| File | What it shows |
| --- | --- |
| `cycles.png` | Cycles home: the fixed groups — Needs Attention, Active, Recent — with the finished cycles on their shared columns. |
| `library.png` | The Library with a suite expanded: its templates on aligned rows, and the selected template's step timeline. |
| `dialog.png` | The template dialog on step 1, Parameters: the label/value/reset grid every case is materialized through. |
| `cycle.png` | The cycle workspace while a case runs: the sidebar's case rows, the header's run controls, and live telemetry in its plots. |
| `history.png` | The History detail for a recorded cycle: each execution with its outcome, and one inspected checkpoint's records. |
| `settings.png` | Settings: the two-column rows that autosave, with no Save buttons. |
| `touch-cycle.png` | The same cycle workspace at touch density, captured at 1920x1200. |

Every image but `touch-cycle.png` is captured at 1600x1000.

The frozen HTML report is not published here. It is a file rather than a screen,
and `crates/dokime-gui/e2e/native-desktop.mjs` writes one from the application
and loads it back into the same WebView, which certifies what a picture of it
only suggests.

## Regenerating Them

```sh
mise run screenshots:build
mise run screenshots:capture
```

`screenshots:build` builds the `dokime` backend, the OBC charger plugin bundle, and
the desktop shell with `tauri/custom-protocol` so the shell runs without a Vite
development server beside it. `screenshots:capture`
(`tools/node/gui/capture-screenshots.mjs`) starts a backend with the plugin
loaded, drives the real WebView through `WebKitWebDriver`, materializes two
cycles through the template dialog, runs them, answers their operator prompts,
switches density through the Appearance setting, and writes each image over the
file of the same name.

The capture is Linux-only and needs a running display session, `WebKitWebDriver`,
`unshare` with unprivileged user namespaces, and ImageMagick (`magick`). It fails
loudly rather than publishing an image that still carries personal data.
