<!-- Rendered by stkr project update from .agents/skills/dokime-fast-release/SKILL.md; sha256:ebbf89d2f3d223b51a5381785ff97bc0757d3a4581e490e5009e9fb664085656 -->

---
name: dokime-fast-release
description: Run the Dokime dev channel so a requested fix reaches the owner's plugin work within 30 minutes.
---

# Dokime Fast Release

The dev channel delivers integrated work to the owner's daily plugin development without waiting for releases. Releases keep the full `stkr-publish` flow; this loop never replaces it.

## Branches

- The next-release integration branches are retired. Every task branch goes through a PR into `dev`, with required native checks and reviews; never commit directly to `dev`.
- Dokime `dev` currently pins Harmos to the owner-selected exact Public-visibility Git revision. Release / Dry refuses a Git pin; before a release, publish Harmos and pin its exact crates.io version.

## Channel

- A GitHub pre-release tagged `dev-channel` on the Dokime repository (immutable releases are disabled for Dokime so its assets can be replaced; the tag never moves and the release notes name the built commit) carries the latest dev build: the desktop installer, `dokime-cli`, `cargo-dokime`, the host plugin bundle and `SHA256SUMS`. Every build replaces its assets; the notes name the Dokime and Harmos commits and the changes since the previous build.
- Builds run from Dokime `dev` in a dedicated builder worktree with a warm target, using `mise run build:rust -- --profile release` and `mise run package:rust -- --profile release --target native`. Hosted CI runs on the pushed commit in parallel and does not gate the channel.

## Consumer

```toml
# Plugin Cargo.toml: SDK at the dev channel's branch
dokime = { git = "https://github.com/stokker-technologies/dokime", branch = "dev" }
```

```toml
# Plugin mise.toml: authoring tool from the dev channel
[tools."github:stokker-technologies/dokime"]
version = "dev-channel"
```

Loop: `cargo update -p dokime`, `mise install`, `cargo dokime build` (or `cargo dokime dev`), and reinstall the desktop installer only when it changed.

## Fast Lane

- One hot-fix worker per product takes small owner tasks (about 15 minutes of work), proves them with targeted tests only, and commits on an owned task branch for a PR into `dev`. Larger work is split or moved to the background lane.
- At most two heavy builds run at once; background streams yield to the fast lane and the channel builder.
- Budget: brief and worker start 2 minutes, fix and targeted proof 10 to 15, merge, warm build and upload 8 to 12, owner rebuild 5. Cold rebuilds (toolchain, Cargo or Harmos version moves) run outside the owner's working hours.
