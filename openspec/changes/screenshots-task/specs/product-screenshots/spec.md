## ADDED Requirements

### Requirement: Captured Screenshots

The `screenshots` task SHALL capture every declared scene from the running desktop application in
the light and the dark theme and write `docs/screenshots/<scene>-<theme>.png` with a manifest.

#### Scenario: Both Themes Per Scene

- **WHEN** the task completes for a scene
- **THEN** a light and a dark image of that scene exist at the scene's viewport and the manifest lists both with the revision

#### Scenario: Check Without Capture

- **WHEN** the task runs with `--check`
- **THEN** it starts no application and fails if an image is missing, has the wrong dimensions or is absent from the manifest
