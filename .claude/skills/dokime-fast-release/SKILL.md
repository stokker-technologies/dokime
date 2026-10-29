---
name: dokime-fast-release
description: Run the Dokime dev channel so a requested fix reaches the owner's plugin work within 30 minutes.
---
<!-- Rendered by stkr project update from .agents/skills/dokime-fast-release/SKILL.md; sha256:7c3563fbf1e0ed9cb2ba54a394d7800765560c36fae733071f098ded00e355f7 -->

# Dokime Fast Release

The dev channel delivers integrated work to the owner's daily plugin development without waiting for releases. Releases keep the full `stkr-publish` flow; this loop never replaces it.

## Branches

- `feat/next-release` in Dokime and Harmos are long-lived integration branches with draft PRs into `dev`. All next-release work lands there.
- While on these branches, Dokime may pin Harmos to an exact git revision of Harmos `feat/next-release`. Release / Dry fails while a git pin exists; that is expected on the draft. Before the PR leaves draft, Harmos is released and Dokime pins the exact crates.io version.

## Channel

- A GitHub pre-release tagged `dev-channel` on the Dokime repository (immutable releases are disabled for Dokime so its assets can be replaced; the tag never moves and the release notes name the built commit) carries the latest dev build: the desktop installer, `dokime-cli`, `cargo-dokime`, the host plugin bundle and `SHA256SUMS`. Every build replaces its assets; the notes name the Dokime and Harmos commits and the changes since the previous build.
- Builds run from a dedicated builder worktree with a warm target, using `mise run build:rust -- --profile release` and `mise run package:rust -- --profile release --target native`. Hosted CI runs on the pushed commit in parallel and does not gate the channel.

## Consumer

```toml
# Plugin Cargo.toml: SDK at the dev channel's branch
dokime = { git = "https://github.com/stokker-technologies/dokime", branch = "feat/next-release" }
```

```toml
# Plugin mise.toml: authoring tool from the dev channel
[tools."github:stokker-technologies/dokime"]
version = "dev-channel"
```

Loop: `cargo update -p dokime`, `mise install`, `cargo dokime build` (or `cargo dokime dev`), and reinstall the desktop installer only when it changed.

## Fast Lane

- One hot-fix worker per product takes small owner tasks (about 15 minutes of work), proves them with targeted tests only, and commits to `feat/next-release`. Larger work is split or moved to the background lane.
- At most two heavy builds run at once; background streams yield to the fast lane and the channel builder.
- Budget: brief and worker start 2 minutes, fix and targeted proof 10 to 15, merge, warm build and upload 8 to 12, owner rebuild 5. Cold rebuilds (toolchain, Cargo or Harmos version moves) run outside the owner's working hours.
