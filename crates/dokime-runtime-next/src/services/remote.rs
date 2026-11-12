/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::remote_server::Remote for Services {
    async fn create_pairing(
        &self, request: tonic::Request<v1::RemoteCreatePairingRequest>,
    ) -> Result<tonic::Response<v1::CreatedPairing>, tonic::Status> {
        let reply = dokime_operations_next::client::remote_create_pairing::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunCreatePairingStream = dokime_operations_next::client::Output<v1::RemoteCreatePairingRunOutput>;
    async fn run_create_pairing(
        &self, request: tonic::Request<tonic::Streaming<v1::RemoteCreatePairingRunInput>>,
    ) -> Result<tonic::Response<Self::RunCreatePairingStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(
            self.context.clone(),
            request.into_inner(),
            self.context.settings.mutation_timeout,
            |frame| {
                frame.value.map(|value| match value {
                    v1::remote_create_pairing_run_input::Value::Request(request) => RunFrame::Request(request),
                    v1::remote_create_pairing_run_input::Value::Finish(_) => RunFrame::Finish,
                    v1::remote_create_pairing_run_input::Value::Cancel(_) => RunFrame::Cancel,
                })
            },
            |reply| v1::RemoteCreatePairingRunOutput {
                value: Some(v1::remote_create_pairing_run_output::Value::Response(reply)),
            },
            |context, request| async move { dokime_operations_next::client::remote_create_pairing::handle(&context, request).await },
        )
        .await
    }
    async fn read_listener(
        &self, request: tonic::Request<v1::RemoteReadListenerRequest>,
    ) -> Result<tonic::Response<v1::RemoteListenerStatus>, tonic::Status> {
        let reply = dokime_operations_next::client::remote_read_listener::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunReadListenerStream = dokime_operations_next::client::Output<v1::RemoteReadListenerRunOutput>;
    async fn run_read_listener(
        &self, request: tonic::Request<tonic::Streaming<v1::RemoteReadListenerRunInput>>,
    ) -> Result<tonic::Response<Self::RunReadListenerStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(
            self.context.clone(),
            request.into_inner(),
            self.context.settings.read_timeout,
            |frame| {
                frame.value.map(|value| match value {
                    v1::remote_read_listener_run_input::Value::Request(request) => RunFrame::Request(request),
                    v1::remote_read_listener_run_input::Value::Finish(_) => RunFrame::Finish,
                    v1::remote_read_listener_run_input::Value::Cancel(_) => RunFrame::Cancel,
                })
            },
            |reply| v1::RemoteReadListenerRunOutput {
                value: Some(v1::remote_read_listener_run_output::Value::Response(reply)),
            },
            |context, request| async move { dokime_operations_next::client::remote_read_listener::handle(&context, request).await },
        )
        .await
    }
    async fn regenerate_certificate(
        &self, request: tonic::Request<v1::RemoteRegenerateCertificateRequest>,
    ) -> Result<tonic::Response<v1::RemoteListenerStatus>, tonic::Status> {
        let reply = dokime_operations_next::client::remote_regenerate_certificate::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunRegenerateCertificateStream = dokime_operations_next::client::Output<v1::RemoteRegenerateCertificateRunOutput>;
    async fn run_regenerate_certificate(
        &self, request: tonic::Request<tonic::Streaming<v1::RemoteRegenerateCertificateRunInput>>,
    ) -> Result<tonic::Response<Self::RunRegenerateCertificateStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.mutation_timeout,
            |frame| frame.value.map(|value| match value {
                v1::remote_regenerate_certificate_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::remote_regenerate_certificate_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::remote_regenerate_certificate_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::RemoteRegenerateCertificateRunOutput { value: Some(v1::remote_regenerate_certificate_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::remote_regenerate_certificate::handle(&context, request).await },
        ).await
    }
    async fn update_listener(
        &self, request: tonic::Request<v1::RemoteUpdateListenerRequest>,
    ) -> Result<tonic::Response<v1::RemoteListenerStatus>, tonic::Status> {
        let reply = dokime_operations_next::client::remote_update_listener::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunUpdateListenerStream = dokime_operations_next::client::Output<v1::RemoteUpdateListenerRunOutput>;
    async fn run_update_listener(
        &self, request: tonic::Request<tonic::Streaming<v1::RemoteUpdateListenerRunInput>>,
    ) -> Result<tonic::Response<Self::RunUpdateListenerStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(
            self.context.clone(),
            request.into_inner(),
            self.context.settings.mutation_timeout,
            |frame| {
                frame.value.map(|value| match value {
                    v1::remote_update_listener_run_input::Value::Request(request) => RunFrame::Request(request),
                    v1::remote_update_listener_run_input::Value::Finish(_) => RunFrame::Finish,
                    v1::remote_update_listener_run_input::Value::Cancel(_) => RunFrame::Cancel,
                })
            },
            |reply| v1::RemoteUpdateListenerRunOutput {
                value: Some(v1::remote_update_listener_run_output::Value::Response(reply)),
            },
            |context, request| async move { dokime_operations_next::client::remote_update_listener::handle(&context, request).await },
        )
        .await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
