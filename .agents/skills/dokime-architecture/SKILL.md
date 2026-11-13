---
name: dokime-architecture
description: Define Dokime domain, runtime, shell and packaging ownership and lifecycle boundaries.
---

# Dokime Architecture

Read [Architecture](../../../docs/architecture.md) as the maintained source of
architectural invariants, their reasons, status and guarding proof. Quote the
invariant a proposal touches and update that document in the same delivery.
Establish a mechanism's purpose from its handler, tests and design, or the owner,
before removing, merging or replacing it.

Follow the linked locked OpenSpec mappings for direction; verify current behavior
in the cited source and tests. The execution host is the current implementation; removed source does not
establish proof in this tree. The native desktop is outside workspace membership
until its separate host switch merges.
Keep implementation and authoring instructions in their owning SDK guides and
project skills rather than restating invariants here.
