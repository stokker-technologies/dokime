---
name: dokime-conventions
description: Apply Dokime domain names, feature names, suite layout and product-specific source conventions.
---

# Dokime Conventions

Use `<suite>.<segment>` template identifiers, Title Case labels and lowercase tags. Suite names carry no product prefix.

Name capability catalogs `Required`, `Subscriptions`, `Streams` and `Prompts`; name the suite catalog `Suites`.

Keep plugin-owned contracts in `domain/`, provided handlers in `operations/`, and templates in `suites/<suite>/<template>.rs`. Create only populated contribution modules; retain the repository skeleton and section markers.
