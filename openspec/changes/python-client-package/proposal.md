# Python Client Package

## Why
The existing PyPI dokime distribution belongs to another product. The user chose dokime-client as the distribution and explicitly corrected its import package to dokime_client.

## What Changes
Migrate the package directory, import namespace, generated bindings, uv workspace, build/package consumers, tests and documentation together. Use uv in installation examples. Preserve Rust crates, protocol namespaces, executable names and product identity; no import alias or compatibility package remains.

Repair the required desktop System harness failures found during provider proof. Adopt the user's canonical 120-minute job default and native pull-request cache writes while preserving individual test deadlines and required checks. Reserve local runners for actual crate compilation, including nested acceptance builds and Cargo publication verification; move artifact-only and other noncompiling jobs to hosted runners.

## Authorization
The user authorizes complete implementation and affected local proof before updating existing PR 7 into dev. Timed benchmarks remain deferred. The parent owns release dispatch and provider account configuration.
