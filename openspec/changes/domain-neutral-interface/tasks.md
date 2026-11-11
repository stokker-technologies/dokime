# Domain-Neutral Interface Tasks

## 1. Contract

- [ ] 1.1 Lock the source inventory from the occupancy and domain-leak findings: interface, domain, wire, server, runtime, host plugin, tests and guidance.

## 2. Grid Plot

- [ ] 2.1 Add the grid plot kind to the document definition, wire messages and adapters, with cells, groups, frame and value bindings, reduction and highlight.
- [ ] 2.2 Add the grid renderer with the frame slider and follow-latest, and compute frames with the generic reduction in the plot path.
- [ ] 2.3 Declare the occupancy plot from the host plugin as a grid and delete the Wi-Fi component, analysis, disk-backed variant, wire messages and their tests.

## 3. Other Domain Knowledge

- [ ] 3.1 Replace the Wi-Fi channel selector with a generic grouped choice fed by the plugin's option provider.
- [ ] 3.2 Replace the fixed host-system traceability fields, categories and titles with fields and groups the plugin contributes.
- [ ] 3.3 Remove unit inference from field names and the hardcoded stream identifiers and labels in plot grouping; use declared field metadata.
- [ ] 3.4 Delete the unsupported plot kinds from the definition, wire and adapters.

## 4. Final Validation

- [ ] 4.1 Run focused interface, runtime plot and host plugin tests, and the story that shows the occupancy grid live and in a report.
- [ ] 4.2 Update the authoring guidance and the product guide.

## Workflow Follow-Up

- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.
