# Project Guidance

## Project Tooling

- Use `mise tasks`. Run focused work with `mise run`.
- Branch names follow the conventional commit conventions. (e.g. `feat: storage engine` becomes `feat/storage-engine`)

## Development Policies

- The `dokime` project is in development, breaking APIs and storage formats is allowed. Legacy compatibility is out of scope.
- Breaking changes are encouraged if they leader to cleaner, more idiomatic or streamlined code.
- There must be no unjustified increase in project complexity - a reduction in managed lines of code is always a win.

## Documentation

- Use documentation in `docs/` as a source of truth for the project.
- Never edit documentation in `docs/`. This is user managed.
- Do not add tests for docs and the related sites and frontends.
