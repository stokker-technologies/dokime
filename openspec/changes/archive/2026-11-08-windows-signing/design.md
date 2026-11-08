## Context

Packaging currently signs updater bytes internally. Signing the returned installer afterward invalidates updater metadata. The owner has no Authenticode certificate.

## Decisions

The composite action signs exported CLI binaries and wraps the existing packaging task. Azure uses Microsoft's pinned official action to install its signing module and sign exported executables; the packaging hook uses that same module for staged binaries and the final installer. PFX uses SignTool. Secrets are step environment values, never manifest contents. The hook verifies Authenticode status and timestamp before returning; minisign and inventory hashing happen afterward. No credentials means an explicit skip; partial or conflicting credentials fail closed.

The manifest schema remains unchanged. `/ALLUSERS` selects Program Files, HKLM registration, common shortcuts and machine PATH. `/CURRENTUSER` selects the existing default. Machine installation/uninstallation requires an elevated deployment process; an unelevated machine request fails with 740 rather than silently falling back. Applications remain unelevated. Scope is persisted in uninstall registration and inferred by the uninstaller from its installation location. Organizations select the switch in their deployment recipe.

## Risks and Validation

Certificate-backed signing cannot be exercised without owner credentials. Deterministic no-network tests cover credential selection, skipped signing and generated installer scope logic. Native managed-image install/uninstall and certificate-backed signing remain qualification work, not claims made by unit tests. NSIS is not a transactional MSI; rollback is an explicit uninstall/reinstall procedure with evidence backup.
