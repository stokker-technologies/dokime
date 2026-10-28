/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use rusqlite::OptionalExtension;

use super::{Error, State};

#[cfg(feature = "host")]
use crate::config::ApplicationMetadata;
use dokime_domain::model::settings::{ApplicationSettings, SettingsSnapshot, UpdateSettings};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS application_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1), revision INTEGER NOT NULL CHECK (revision >= 0), body BLOB NOT NULL
);
CREATE TABLE IF NOT EXISTS application_metadata (
    id INTEGER PRIMARY KEY CHECK (id = 1), body BLOB NOT NULL
);";

impl State {
    pub(crate) async fn initialize_settings(&self) -> Result<(), Error> {
        self.call(|connection| {
            let transaction = connection.transaction()?;
            transaction.execute_batch(SCHEMA)?;
            let result = (|| -> Result<(), Error> {
                let settings = ApplicationSettings::default();
                settings.validate().map_err(Error::InvalidSettings)?;
                let body = serde_json::to_vec(&settings).map_err(|error| Error::InvalidSettings(error.to_string()))?;
                transaction.execute("INSERT OR IGNORE INTO application_settings VALUES (1, 0, ?1)", [&body])?;
                #[cfg(feature = "host")]
                {
                    let metadata = ApplicationMetadata::new();
                    let body = serde_json::to_vec(&metadata).map_err(|error| Error::InvalidSettings(error.to_string()))?;
                    transaction.execute("INSERT OR IGNORE INTO application_metadata VALUES (1, ?1)", [&body])?;
                    read_metadata(&transaction)?;
                }
                read_snapshot(&transaction)?;
                super::commit(transaction)?;
                Ok(())
            })();
            Ok(result)
        })
        .await?
    }

    pub async fn read_application_settings(&self) -> Result<SettingsSnapshot, Error> {
        self.call(|connection| Ok(read_snapshot(connection))).await?
    }

    pub async fn update_application_settings(&self, update: UpdateSettings) -> Result<SettingsSnapshot, Error> {
        update.settings.validate().map_err(Error::InvalidSettings)?;
        self.call(move |connection| {
            let transaction = connection.transaction()?;
            let result = (|| -> Result<SettingsSnapshot, Error> {
                let current = read_snapshot(&transaction)?;
                if current.revision != update.expected_revision {
                    return Err(Error::SettingsConflict {
                        expected: update.expected_revision,
                        actual: current.revision,
                    });
                }
                let revision = current
                    .revision
                    .checked_add(1)
                    .filter(|revision| *revision <= i64::MAX as u64)
                    .ok_or_else(|| Error::InvalidSettings("settings revision exhausted".into()))?;
                let body = serde_json::to_vec(&update.settings).map_err(|error| Error::InvalidSettings(error.to_string()))?;
                let changed = transaction.execute(
                    "UPDATE application_settings SET revision=?1, body=?2 WHERE id=1 AND revision=?3",
                    rusqlite::params![revision as i64, body, current.revision as i64],
                )?;
                if changed != 1 {
                    return Err(Error::InvalidSettings("settings singleton disappeared".into()));
                }
                super::commit(transaction)?;
                Ok(SettingsSnapshot { revision, settings: update.settings })
            })();
            Ok(result)
        })
        .await?
    }

    #[cfg(feature = "host")]
    pub(crate) async fn read_application_metadata(&self) -> Result<ApplicationMetadata, Error> {
        self.call(|connection| Ok(read_metadata(connection))).await?
    }

    #[cfg(feature = "host")]
    pub(crate) async fn edit_application_metadata<T: Send + 'static, E: From<Error> + Send + 'static>(
        &self, change: impl FnOnce(&mut ApplicationMetadata) -> Result<T, E> + Send + 'static,
    ) -> Result<T, E> {
        self.call(move |connection| {
            let transaction = connection.transaction()?;
            let result = (|| -> Result<T, E> {
                let mut metadata = read_metadata(&transaction).map_err(E::from)?;
                let identity = metadata.identity.clone();
                let result = change(&mut metadata)?;
                if metadata.identity != identity {
                    return Err(E::from(Error::InvalidSettings("node identity is immutable".into())));
                }
                metadata.validate().map_err(Error::InvalidSettings).map_err(E::from)?;
                let body = serde_json::to_vec(&metadata).map_err(|error| E::from(Error::InvalidSettings(error.to_string())))?;
                transaction.execute("UPDATE application_metadata SET body=?1 WHERE id=1", [&body]).map_err(Error::from).map_err(E::from)?;
                super::commit(transaction).map_err(Error::from).map_err(E::from)?;
                Ok(result)
            })();
            Ok(result)
        })
        .await
        .map_err(E::from)?
    }
}

fn read_snapshot(connection: &rusqlite::Connection) -> Result<SettingsSnapshot, Error> {
    let row: Option<(i64, Vec<u8>)> = connection
        .query_row("SELECT revision, body FROM application_settings WHERE id=1", [], |row| Ok((row.get(0)?, row.get(1)?)))
        .optional()?;
    let (revision, body) = row.ok_or_else(|| Error::InvalidSettings("application settings are not initialized".into()))?;
    let settings: ApplicationSettings = serde_json::from_slice(&body).map_err(|error| Error::InvalidSettings(error.to_string()))?;
    settings.validate().map_err(Error::InvalidSettings)?;
    let revision = u64::try_from(revision).map_err(|_| Error::InvalidSettings("negative application settings revision".into()))?;
    Ok(SettingsSnapshot { revision, settings })
}

#[cfg(feature = "host")]
fn read_metadata(connection: &rusqlite::Connection) -> Result<ApplicationMetadata, Error> {
    let body: Vec<u8> = connection.query_row("SELECT body FROM application_metadata WHERE id=1", [], |row| row.get(0))?;
    let metadata: ApplicationMetadata = serde_json::from_slice(&body).map_err(|error| Error::InvalidSettings(error.to_string()))?;
    metadata.validate().map_err(Error::InvalidSettings)?;
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn updates_are_revision_checked_atomic_and_durable() {
        let root = tempfile::tempdir().unwrap();
        let database = State::open(root.path()).await.unwrap();
        database.initialize_settings().await.unwrap();
        let original = database.read_application_settings().await.unwrap();
        let mut settings = original.settings.clone();
        settings.report.plot_points = 321;
        let saved = database
            .update_application_settings(UpdateSettings {
                expected_revision: original.revision,
                settings,
            })
            .await
            .unwrap();
        assert_eq!(saved.revision, original.revision + 1);
        let conflict = database
            .update_application_settings(UpdateSettings {
                expected_revision: original.revision,
                settings: original.settings,
            })
            .await
            .unwrap_err();
        assert!(matches!(conflict, Error::SettingsConflict { expected, actual }
            if expected == original.revision && actual == saved.revision));

        let mut invalid = saved.settings.clone();
        invalid.telemetry.interval_ms = 0;
        let invalid = database
            .update_application_settings(UpdateSettings {
                expected_revision: saved.revision,
                settings: invalid,
            })
            .await;
        assert!(matches!(invalid, Err(Error::InvalidSettings(_))));
        assert_eq!(database.read_application_settings().await.unwrap(), saved);

        database
            .call(|connection| {
                connection.execute_batch(
                    "CREATE TRIGGER refuse_settings BEFORE UPDATE ON application_settings BEGIN SELECT RAISE(ABORT, 'fixture'); END;",
                )
            })
            .await
            .unwrap();
        let mut next = saved.settings.clone();
        next.report.plot_points = 456;
        assert!(
            database
                .update_application_settings(UpdateSettings {
                    expected_revision: saved.revision,
                    settings: next
                })
                .await
                .is_err()
        );
        assert_eq!(database.read_application_settings().await.unwrap(), saved);
        database.close().await.unwrap();
        let reopened = State::open(root.path()).await.unwrap();
        reopened.initialize_settings().await.unwrap();
        assert_eq!(reopened.read_application_settings().await.unwrap(), saved);
        reopened.close().await.unwrap();
    }
}
