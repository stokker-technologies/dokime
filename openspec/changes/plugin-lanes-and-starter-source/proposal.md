# Plugin Lanes and Starter Source

Status: both parts locked by the owner on 2026-11-19. Lanes wait for the
library's plugin-lanes change; the starter source lands with the native SDK
starters.

## Why

Plugin authors need to say that some operations never overlap, and the library
is gaining lanes a plugin can declare. The product's SDKs must expose that in
their locked authoring shape.

Separately, `dokime new` writes a starter whose SDK dependency points at
something published, so any test of it depends on an earlier release, and
trying an unpublished SDK means editing the created project by hand.

## What Changes

- Authoring: a plugin declares lanes and assigns operations to them, in every
  SDK language, in the idiom of the locked shape.
- `dokime new <name> --lang <language> --from <checkout>` takes the starter
  files from that checkout and points the created project's SDK dependency at
  that checkout's SDK as a local path. Nothing is downloaded. The environment
  variable `DOKIME_NEW_FROM` sets the same; the argument wins.
- Every story that creates a starter uses the checkout under test, so no test
  depends on a published release.

## Capabilities

### New Capabilities

- `plugin-lanes`: lane declaration in the product's SDK authoring surfaces.
- `starter-source`: creating a starter from a checkout.

## Impact

The four SDK authoring layers and their starters, `dokime new`, the starter
stories, README command examples.
