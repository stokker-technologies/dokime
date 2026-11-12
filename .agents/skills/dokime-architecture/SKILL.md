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
in the cited source and tests. The existing runtime and additive execution host
coexist: an API or proof from one does not establish the other's behavior.
Keep implementation and authoring instructions in their owning SDK guides and
project skills rather than restating invariants here.
