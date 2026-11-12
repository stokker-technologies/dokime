/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Reads Runtime

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
async fn client_reads_runtime() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Start a Real Host and Inspect Desktop Bootstrap Identity
        let root = tempfile::tempdir()?;
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut runtime = dokime::v1::runtime_client::RuntimeClient::connect(process.url.clone()).await?;
        let mut backend = dokime::v1::backend_client::BackendClient::connect(process.url.clone()).await?;
        let identity = runtime.get_identity(dokime::v1::RuntimeGetIdentityRequest {}).await?.into_inner();
        assert_eq!(identity, backend.get_identity(dokime::v1::GetBackendIdentity {}).await?.into_inner());
        assert_eq!(runtime.get_health(dokime::v1::RuntimeGetHealthRequest {}).await?.into_inner().backend_id, identity.backend_id);

        // Read Through the Request and Finish Transport
        let input = tokio_stream::iter([
            dokime::v1::RuntimeGetIdentityRunInput {
                value: Some(dokime::v1::runtime_get_identity_run_input::Value::Request(dokime::v1::RuntimeGetIdentityRequest {})),
            },
            dokime::v1::RuntimeGetIdentityRunInput {
                value: Some(dokime::v1::runtime_get_identity_run_input::Value::Finish(dokime::v1::ClientEmpty {})),
            },
        ]);
        let mut output = runtime.run_get_identity(input).await?.into_inner();
        assert_eq!(
            output.message().await?.unwrap().value.unwrap(),
            dokime::v1::runtime_get_identity_run_output::Value::Response(identity.clone())
        );
        assert!(output.message().await?.is_none());
        process.close().await?;

        // Reopen the Same Root with a New Boot Epoch and Stable Identity
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut runtime = dokime::v1::runtime_client::RuntimeClient::connect(process.url.clone()).await?;
        let reopened = runtime.get_identity(dokime::v1::RuntimeGetIdentityRequest {}).await?.into_inner();
        assert_eq!(reopened.backend_id, identity.backend_id);
        assert_ne!(reopened.epoch, identity.epoch);
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
