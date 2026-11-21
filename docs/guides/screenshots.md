---
sidebar: { order: 6 }
---

# Screenshots

The published images document desktop workflows in light and dark themes.
Each scene has `<scene>-light.png` and `<scene>-dark.png`; the manifest retains
viewport, density and capture metadata.

Capture requires native desktop integration and the [OBC plugin](../../plugins/sidecars/obc/),
which the host fixtures build and the screenshot journey admits beside its backend.
`mise run screenshots -- --check` verifies the retained image inventory without
starting a desktop or rewriting images. Review generated images before committing.
