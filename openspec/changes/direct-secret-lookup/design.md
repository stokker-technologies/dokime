# Direct Secret Lookup Design

## Locked Contract

`SettingsEntry::Secret(SecretSetting { id, title, description, required })` is a
declaration, never a saved setting value. The identifier is the shared store key.
`SecretReference`, environment fallback, binding metadata and per-plugin secret
update operations are removed. Existing saved references are discarded on read;
ordinary settings retain validation. Storage and evidence remain v1.

At each initialization attempt, resolve all declared identifiers before launching
the plugin. A required missing identifier returns typed `UnavailableSecret(String)`
with the message `secret \`api_token\` is not in Secret Storage; add it and restart`.
Optional missing identifiers resolve to `None`. Backend failures remain failures;
there is no retry loop or fallback. Existing Retry starts a fresh lookup.

Plugin settings expose only declaration availability, without values or alternative
source identifiers. The GUI displays declared identifiers read-only, and Add In
Secret Storage opens the existing shared Add Secret dialog with identifier filled.
Secret catalog references come from verified installed declarations and live
registrations, deduplicated by plugin identifier, including inactive programs.

## Ownership And Delivery

Domain declarations and wire contracts precede runtime/preferences and server
adapters; macros and starter consumers follow. GUI adapters and settings surfaces
consume regenerated contracts. Focused runtime and GUI stories cover replacement
behavior. The inventory is tasks.md. One agent owns this worktree; the concurrent
root-cause worker owns its separate branch and presentation changes.

Keep this requested change tracked under openspec/changes/direct-secret-lookup
for integration review. The owner explicitly limits proof to checks and focused
tests and delivery to a local Conventional Commit.
