---
name: dokime-conventions
description: Apply Dokime presentation conventions when authoring labels, units, plugins, suites and documents.
---

# Dokime Conventions

Use style-guide Title Case for headings, section and step comments, and human-facing labels: capitalize the first and last word and all major words. Lowercase interior a, an, the, and, or, but, nor, of, in, on, at, to, for, by, with, from, as, into, per, via, vs, over, up and off. Keep verb particles capitalized when they belong to a phrasal verb, as in “Turn Off the Heater”. Verbs and subordinating words remain capitalized: `Number of Rows`, `Plugin Startup and Stop Budgets`, `Wait Until Chamber Is Ready`. Preserve identifiers, inline code, product names, units and protocol acronyms.

Write units in square brackets directly after the label on the same line: `Mean [Mbps]`. Never use parentheses for units or place the unit on a second line below the title.

Prefer a bare label and the structured unit field in plugin, suite and document declarations. Presentation code uses `formatUnitLabel` in `crates/dokime-gui/src/lib/format.ts`, shared by the live GUI and runtime report HTML. If a declaration has no structured unit field, use the bracket convention in its label.

Keep the complete label and unit together. Table headers truncate with a tooltip containing the full heading; sort and filter controls remain visible and usable. Preserve the unit's case and symbol. Numeric values, axis ticks and editable value fields retain their value formatting; this convention governs labels and titles.
