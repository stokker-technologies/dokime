<!-- Rendered by stkr project update from .agents/GUIDANCE.md; sha256:9ddea0d66f0fe91762ba291134f1fb78dbed2925792f76af716226dfe3eed7cc -->

# Project Guidance

## Development Storage Policy

- Dokime is in development. Database schemas and evidence formats may change without version bumps for now; keep their baseline at v1.
- Clean database breaks are allowed. Do not add migrations or legacy compatibility solely to preserve development data. Use fresh storage roots when a schema change makes existing data incompatible; never silently delete or reset existing data.
