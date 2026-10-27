# Backend Registry

## ADDED Requirements

### Requirement: Shared Backend Registry

The desktop and CLI SHALL share one backend connection registry, separate from application settings, under the following contract.

#### Scenario: Startup Default Fails

- **WHEN** the desktop starts and `Document.default` is absent, deleted or fails to connect
- **THEN** it attempts the default once and opens the same Backends table used in Settings
- **AND** it neither silently falls back to local nor retries forever

#### Scenario: Delete The Active Backend

- **WHEN** the user confirms deletion of the active row
- **THEN** the same registry mutation as CLI remove runs and clears the default when necessary
- **AND** that connection generation disconnects

#### Scenario: Select A Backend

- **WHEN** the user selects a profile
- **THEN** `default` is written only after a successful connect and after checking that the profile still matches the selected snapshot

#### Scenario: Locked Design Decisions Apply

- **WHEN** this requirement is implemented or reviewed
- **THEN** the following locked design decisions hold unchanged:

`dokime-client/src/profiles.rs::Store::discover` already resolves `DOKIME_PROFILES_FILE` or platform `dokime/backends.json`; CLI add/list/remove/use share this store and its locked atomic update. Preserve this connection registry as the explicitly requested existing storage, separate from application settings. Move portable `Profile`, `Document`, `Secret` declarations into `dokime-domain/src/model/backends.rs` and re-export them at their current client paths. Use `Profile { identifier, label, endpoint, expected_backend_id, ca_certificate_pem, secret }`, `Document { version, default, profiles }`, and `Secret::{None,Keyring}`; strip only `ClientSettings` arguments in their methods, using owner constants instead. Preserve `Store::{discover,load,update,remember}` semantics.

Native GUI commands: `desktop_backends_list() -> Result<Document, ConnectionError>`, `desktop_backend_add(profile: Profile) -> Result<Document, ConnectionError>`, `desktop_backend_remove(identifier: String) -> Result<Document, ConnectionError>`, `desktop_backend_use(identifier: String) -> Result<ConnectionSnapshot, ConnectionError>`, `desktop_settings_read(scope: SettingsScope) -> Result<SettingsSnapshot, SettingsCommandError>`, `desktop_settings_update(scope: SettingsScope, update: UpdateSettings) -> Result<SettingsSnapshot, SettingsCommandError>`. `SettingsCommandError` preserves validation/conflict/disconnected/persistence categories. Generate TS bindings for these commands and domain types, and Python types only where a client consumes the settings operations.

Use the existing `connection::Manager::connect(ConnectRequest)` unchanged in shape; retain generation cancellation, expected identity, CA validation and keyring access. On startup attempt `Document.default` once; absent/default-deleted/failed selection opens the same Backends table used in Settings. Rows show identifier, label, endpoint, last-selected state and actions; Add dialog imports pairing files, with manual fields only for unsecured lab endpoints. Delete requires confirmation and calls the same registry mutation as CLI remove, clearing default when necessary; deleting the active row disconnects that generation. Selection writes `default` only after successful connect and after checking that the profile still matches the selected snapshot. CLI `backend use` retains its offline default-selection behavior. Do not silently fall back to local or retry forever. A first-run local runtime can be added as a named profile using its actual endpoint.
