# Project Guidance

## Branch Work

- Resume from `openspec/changes/grpc-plugins/design.md` and the owning change's task list. All open changes integrate together on `feat/grpc-plugins`; this feature worktree hands back local commits. Do not push without authorization.
- Discover tasks with `mise tasks`; run focused work through `mise run`. Hosted CI and Nightly provide full validation.
- Build Python host fixtures with `mise run build:host-fixtures -- --language python`.

## Development Storage Policy

- Dokime is in development. Database schemas and evidence formats may change without version bumps for now; keep their baseline at v1.
- Clean database breaks are allowed. Do not add migrations or legacy compatibility solely to preserve development data. Use fresh storage roots when a schema change makes existing data incompatible; never silently delete or reset existing data.

## Documentation

- Read `docs/architecture.md` before proposing architecture, protocol or public API changes, and update it in the same delivery.
- Start at `README.md` and `docs/product-guide.md`. Read `docs/gui.md` before changing `crates/dokime-gui/`.
- Keep documentation in `docs/` and refer to code by path. Do not add `AGENTS.md`, `CLAUDE.md` or other Markdown notes inside code directories; package READMEs that registries publish are the exception.
