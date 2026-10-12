# Project Guidance

## Documentation Access

- Never read or write `docs/` unless the user explicitly instructs you to do so. Do not use `docs/` as implicit task context.

## Development Storage Policy

- Dokime is in development. Database schemas and evidence formats may change without version bumps for now; keep their baseline at v1.
- Clean database breaks are allowed. Do not add migrations or legacy compatibility solely to preserve development data. Use fresh storage roots when a schema change makes existing data incompatible; never silently delete or reset existing data.

## Existing Project Workflow

- Explicit user instructions and more specific repository instructions take precedence.
- Preserve unrelated local changes. Never use git worktrees in this repository.
- Use Conventional Commits for repository changes.
- Do not add assistant/vendor attribution, AI-generated notes, or `Co-Authored-By` trailers unless explicitly requested.

- Project-local guidance and skills are canonical under `.agents/`; regenerate provider copies with `stkr project update`. Do not author competing provider copies.
- Complete cohesive implementation before broad validation. Every project Python environment is the repository-root `.venv/`. Private task inputs follow the Hub task lifecycle.
- Use repository mise tasks discovered from `mise.toml`; apply `dokime-desktop-validation` for changed native desktop workflows.
