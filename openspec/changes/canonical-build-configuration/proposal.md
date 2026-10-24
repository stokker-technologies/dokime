# Canonical Build Configuration

## Why

Product and Tests repeat broad feature configurations and the serial native Build exceeds the required pipeline budget.

## What Changes

Use a canonical shipping feature configuration per native platform and profile, owned by repository mise tasks. Product and Tests run in separate parallel jobs. Ordinary tests use their canonical configuration; independent producer feature sets are permitted, with compatible Cargo cache reuse where fingerprints allow. Keep necessary fault-injection coverage in a narrow explicit exception without adding fault injection to shipped binaries. Retain all-features, no-default-features and declared cross-target compatibility proof only in an explicit local extended suite, alongside fault-injection cases; every provider CI pipeline uses the canonical configuration. Preserve all existing test behavior and packaging contracts.

Warm-cache provider pipelines must finish within 45 minutes, targeting 30. Tests Build must take no longer than Product Build; a breach requires phase-level diagnosis and redundant-work repair, without dropping required coverage.

## Impact

Build tasks, authored build/test producers, CI selection, affected tests, maintained specifications and README claims. Hub owns shared guidance and generated tooling. No release publication is authorized.

