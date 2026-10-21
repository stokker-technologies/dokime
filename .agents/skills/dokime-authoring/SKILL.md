---
name: dokime-authoring
description: Author Dokime plugins, suites, steps, criteria, evidence and documents.
---

# Dokime Authoring

- [Plugins](references/plugins.md): runtime ownership, hosting and contribution layout.
- [Suites](references/suites.md): declarations, lifecycle capabilities and shared handles.
- [Documents](references/documents.md): captured evidence, records, streams and reports.

Depend on `dokime` with feature `plugin` for Dokime authoring. Import product contracts through `dokime::domain`; declare each plugin-owned contract once in its `domain/` owner.
