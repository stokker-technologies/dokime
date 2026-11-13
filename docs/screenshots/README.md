# Screenshots

The published images document desktop workflows in light and dark themes.
Each scene has `<scene>-light.png` and `<scene>-dark.png`; the manifest retains
viewport, density and capture metadata.

Capture requires native desktop integration and a current SDK-authored scene
plugin. Its restoration/re-authoring work is recorded in the combined grpc-plugins
task list. The current Product build does not produce desktop capture fixtures.
`mise run screenshots -- --check` verifies the retained image inventory without
starting a desktop or rewriting images. Review generated images before committing.
