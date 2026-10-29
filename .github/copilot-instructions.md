<!-- Rendered by stkr project update from .agents/GUIDANCE.md; sha256:5b299debed8b55527dd398069d274c5d8930693d97839aaf5608ec9dd049eba5 -->

# Project Guidance

## Development Storage Policy

- Dokime is in development. Database schemas and evidence formats may change without version bumps for now; keep their baseline at v1.
- Clean database breaks are allowed. Do not add migrations or legacy compatibility solely to preserve development data. Use fresh storage roots when a schema change makes existing data incompatible; never silently delete or reset existing data.

## Documentation

- Start at `README.md` and `docs/product-guide.md`; its Documentation section links the topic guides in `docs/`. Read `docs/gui.md` before changing `crates/dokime-gui/`.
- Keep documentation in `docs/` and refer to code by path. Do not add `AGENTS.md`, `CLAUDE.md` or other Markdown notes inside code directories; package READMEs that registries publish are the exception.
