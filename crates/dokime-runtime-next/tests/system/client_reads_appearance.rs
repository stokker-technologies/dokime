/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Reads Appearance

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

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
async fn client_reads_appearance() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        use dokime::v1;

        // Open the Desktop Appearance on a Fresh Host
        let root = tempfile::tempdir()?;
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut appearance = v1::appearance_client::AppearanceClient::connect(process.url.clone()).await?;
        let snapshot = appearance.get_appearance(v1::GetAppearance {}).await?.into_inner();
        assert!(!snapshot.app_name.trim().is_empty());
        assert!(v1::ThemeModePolicy::try_from(snapshot.mode_policy).is_ok());
        let mut output = appearance
            .run_get_appearance(tokio_stream::iter([
                v1::AppearanceGetAppearanceRunInput {
                    value: Some(v1::appearance_get_appearance_run_input::Value::Request(v1::GetAppearance {})),
                },
                v1::AppearanceGetAppearanceRunInput {
                    value: Some(v1::appearance_get_appearance_run_input::Value::Finish(v1::AppearanceGetAppearanceCancellation {})),
                },
            ]))
            .await?
            .into_inner();
        assert_eq!(output.message().await?.unwrap().value.unwrap(), v1::appearance_get_appearance_run_output::Value::Response(snapshot));
        assert!(output.message().await?.is_none());
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
