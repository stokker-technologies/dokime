# Backend Registry

## ADDED Requirements

### Requirement: Shared Backend Registry

The desktop and CLI SHALL share one backend connection registry, separate from application settings. The later locked [Owned Backend Selection](../../../embedded-server-and-ci/specs/desktop-baseline/spec.md) requirement supersedes this change's original desktop-startup and CLI-default coupling. The scenarios below express that precedence; they do not claim its implementation or proof is complete here.

#### Scenario: Remembered Remote Fails

- **WHEN** the desktop starts with an unavailable remembered remote
- **THEN** it presents reconnect and backend-selection controls
- **AND** it neither silently starts Local nor retries forever

#### Scenario: Delete The Active Backend

- **WHEN** the user confirms deletion of the active row
- **THEN** the same registry mutation as CLI remove runs and clears the default when necessary
- **AND** that connection generation disconnects

#### Scenario: Select A Backend

- **WHEN** the user selects a profile
- **THEN** selection validates that the profile still matches the selected snapshot
- **AND** remembered desktop selection remains separate from the CLI default, without persisting ephemeral Local endpoints

#### Scenario: Locked Design Decisions Apply

- **WHEN** this requirement is implemented or reviewed
- **THEN** the following shared-registry decisions hold, with desktop selection owned by the later requirement linked above:

`dokime-client/src/profiles.rs::Store::discover` already resolves `DOKIME_PROFILES_FILE` or platform `dokime/backends.json`; CLI add/list/remove/use share this store and its locked atomic update. Preserve this connection registry as the explicitly requested existing storage, separate from application settings. Move portable `Profile`, `Document`, `Secret` declarations into `dokime-domain/src/model/backends.rs` and re-export them at their current client paths. Use `Profile { identifier, label, endpoint, endpoints, expected_backend_id, ca_certificate_pem, secret }`, `Document { version, default, profiles }`, and `Secret::{None,Keyring}`; strip only `ClientSettings` arguments in their methods, using owner constants instead. Preserve `Store::{discover,load,update,remember}` semantics.

Native GUI commands: `desktop_backends_list() -> Result<Document, ConnectionError>`, `desktop_backend_add(profile: Profile) -> Result<Document, ConnectionError>`, `desktop_backend_remove(identifier: String) -> Result<Document, ConnectionError>`, `desktop_backend_use(identifier: String) -> Result<ConnectionSnapshot, ConnectionError>`, `desktop_settings_read(scope: SettingsScope) -> Result<SettingsSnapshot, SettingsCommandError>`, `desktop_settings_update(scope: SettingsScope, update: UpdateSettings) -> Result<SettingsSnapshot, SettingsCommandError>`. `SettingsCommandError` preserves validation/conflict/disconnected/persistence categories. Generate TS bindings for these commands and domain types, and Python types only where a client consumes the settings operations.

Retain generation cancellation, expected identity, CA validation and keyring access. Rows show identifier, label, endpoint, selected state and actions; The Backends toolbar imports remote files and window drop is supported; the later [Remote Access Files](../../../dokime-remote-files/specs/remote-access/spec.md) requirement owns automatic verified endpoint selection and remembered success. Manual fields remain only for unsecured lab endpoints. Delete requires confirmation and calls the same registry mutation as CLI remove, clearing the CLI default when necessary; deleting the active row disconnects that generation. CLI `backend use` retains its offline default-selection behavior. The later desktop-session owner manages initial Local choice and saved remote selection; neither the original startup attempt of `Document.default` nor saving an ephemeral local endpoint as a profile remains a requirement here.
