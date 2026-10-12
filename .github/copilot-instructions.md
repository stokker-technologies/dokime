<!-- Generated from .agents/GUIDANCE.md; edit .agents/GUIDANCE.md and run `stkr project update` -->

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


## Development Validation Policy

- Dev integration requires complete Linux check, release build, correctness tests and exact-revision artifact validation. Windows validation runs independently as advisory and retains failures and artifacts.
- The Linux-required policy applies only to Dokime Dev; Main, release stabilization, tags and publication require both native platforms. Incomplete Linux-only catalogs cannot satisfy release reuse. Making Windows mandatory for Dev again is an explicit policy change.
- Trusted Dev-push Linux check and artifact-consuming test jobs may use the restricted `stokker-local-linux-x64` runner. Pull requests and other refs use hosted runners. Docker-dependent release builds remain hosted. Preserve exact source identity and retained checks; runner routing is not validation evidence.
