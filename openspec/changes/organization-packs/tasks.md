# Organization Packs Tasks

## Organization Pack Extension

- [ ] 62. Replace branding-pack layout/naming with organization/organization.toml across assets, generators, imports, UI, commands, examples and guidance; preserve compiled identity and restart activation.
- [ ] 63. Preserve installed legacy pack data through safe migration or explicit import guidance; retain the last valid organization/update configuration on invalid input with a visible error.
- [ ] 64. Implement one disabled/manual/automatic-check update policy with direct manifest, GitHub Releases and GitLab Releases sources; installation stays explicit and signing trust stays compiled.
- [ ] 65. Add named GitLab hosts/projects/OAuth connections and authenticated sources with host/account/client-scoped credentials, origin-safe forwarding and no secrets in packs.
- [ ] 66. Implement the version-1 plugin release feed contract in design.md, organization-declared sources/links and authoritative backend target identity; check advertised package/version against inspected bytes, enforce host compatibility/target/digest and reuse Dokime staging/consent/install/restart. Missing feed and target source blocks source completion.
- [ ] 67. Migrate all updater/organization consumers, generated bindings, tests, README and shipped examples; remove superseded GitLab-only updater coupling.
- [ ] 68. Validate switching, disabled-network behavior, public/auth/missing sources, target selection, signature rejection, invalid-pack retention, credential separation and plugin lifecycle with focused tests and native acceptance.
