/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Revision-Checked Settings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_runtime::storage::sqlite::testing::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn updates_are_revision_checked_atomic_and_durable() {
    let root = tempfile::tempdir().unwrap();
    let database = State::open(root.path()).await.unwrap();
    database.initialize_settings().await.unwrap();
    let original = database.read_application_settings().await.unwrap();
    let mut settings = original.settings.clone();
    settings.report.plot_points = 321;
    settings.plugins.startup_timeout_s = 90;
    settings.plugins.stop_timeout_s = 20;
    let saved = database
        .update_application_settings(UpdateSettings {
            expected_revision: original.revision,
            settings,
        })
        .await
        .unwrap();
    assert_eq!(saved.revision, original.revision + 1);
    assert_eq!(database.read_application_settings().await.unwrap(), saved);
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

    let mut unbounded = saved.settings.clone();
    unbounded.plugins.startup_timeout_s = 0;
    let unbounded = database
        .update_application_settings(UpdateSettings {
            expected_revision: saved.revision,
            settings: unbounded,
        })
        .await;
    assert!(matches!(unbounded, Err(Error::InvalidSettings(_))));

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
