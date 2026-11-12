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
impl v1::content_server::Content for Services {
    type DownloadAppearanceAssetStream = dokime_operations_next::client::Output<v1::ContentDownloadAppearanceAssetOutput>;
    async fn download_appearance_asset(
        &self, request: tonic::Request<v1::ContentDownloadAppearanceAssetRequest>,
    ) -> Result<tonic::Response<Self::DownloadAppearanceAssetStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented(
            "Content.DownloadAppearanceAsset: Not Yet Served by the New Host; Refresh Appearance After Its Owner Supplies Asset Storage",
        ))
    }
    type RunDownloadAppearanceAssetStream = dokime_operations_next::client::Output<v1::ContentDownloadAppearanceAssetRunOutput>;
    async fn run_download_appearance_asset(
        &self, request: tonic::Request<tonic::Streaming<v1::ContentDownloadAppearanceAssetRunInput>>,
    ) -> Result<tonic::Response<Self::RunDownloadAppearanceAssetStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented(
            "Content.RunDownloadAppearanceAsset: Not Yet Served by the New Host; Refresh Appearance After Its Owner Supplies Asset Storage",
        ))
    }
    type DownloadArtifactStream = dokime_operations_next::client::Output<v1::ContentDownloadArtifactOutput>;
    async fn download_artifact(
        &self, request: tonic::Request<v1::ContentDownloadArtifactRequest>,
    ) -> Result<tonic::Response<Self::DownloadArtifactStream>, tonic::Status> {
        let reply = dokime_operations_next::client::content_download_artifact::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunDownloadArtifactStream = dokime_operations_next::client::Output<v1::ContentDownloadArtifactRunOutput>;
    async fn run_download_artifact(
        &self, request: tonic::Request<tonic::Streaming<v1::ContentDownloadArtifactRunInput>>,
    ) -> Result<tonic::Response<Self::RunDownloadArtifactStream>, tonic::Status> {
        let reply = dokime_operations_next::client::run_call::finite(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::content_download_artifact_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::content_download_artifact_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::content_download_artifact_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::content_download_artifact::handle(&context, request).await },
            |frame| v1::ContentDownloadArtifactRunOutput {
                value: frame.value.map(|value| match value {
                    v1::content_download_artifact_output::Value::Item(item) => v1::content_download_artifact_run_output::Value::Item(item),
                    v1::content_download_artifact_output::Value::Response(reply) => {
                        v1::content_download_artifact_run_output::Value::Response(reply)
                    }
                }),
            },
            || v1::ContentDownloadArtifactRunOutput {
                value: Some(v1::content_download_artifact_run_output::Value::Response(v1::ClientEmpty {})),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type DownloadEvidenceStream = dokime_operations_next::client::Output<v1::ContentDownloadEvidenceOutput>;
    async fn download_evidence(
        &self, request: tonic::Request<v1::ContentDownloadEvidenceRequest>,
    ) -> Result<tonic::Response<Self::DownloadEvidenceStream>, tonic::Status> {
        let reply = dokime_operations_next::client::content_download_evidence::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunDownloadEvidenceStream = dokime_operations_next::client::Output<v1::ContentDownloadEvidenceRunOutput>;
    async fn run_download_evidence(
        &self, request: tonic::Request<tonic::Streaming<v1::ContentDownloadEvidenceRunInput>>,
    ) -> Result<tonic::Response<Self::RunDownloadEvidenceStream>, tonic::Status> {
        let reply = dokime_operations_next::client::run_call::finite(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::content_download_evidence_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::content_download_evidence_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::content_download_evidence_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::content_download_evidence::handle(&context, request).await },
            |frame| v1::ContentDownloadEvidenceRunOutput {
                value: frame.value.map(|value| match value {
                    v1::content_download_evidence_output::Value::Item(item) => v1::content_download_evidence_run_output::Value::Item(item),
                    v1::content_download_evidence_output::Value::Response(reply) => {
                        v1::content_download_evidence_run_output::Value::Response(reply)
                    }
                }),
            },
            || v1::ContentDownloadEvidenceRunOutput {
                value: Some(v1::content_download_evidence_run_output::Value::Response(v1::ClientEmpty {})),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type DownloadExecutionFileStream = dokime_operations_next::client::Output<v1::ContentDownloadExecutionFileOutput>;
    async fn download_execution_file(
        &self, request: tonic::Request<v1::ContentDownloadExecutionFileRequest>,
    ) -> Result<tonic::Response<Self::DownloadExecutionFileStream>, tonic::Status> {
        let reply = dokime_operations_next::client::content_download_execution_file::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunDownloadExecutionFileStream = dokime_operations_next::client::Output<v1::ContentDownloadExecutionFileRunOutput>;
    async fn run_download_execution_file(
        &self, request: tonic::Request<tonic::Streaming<v1::ContentDownloadExecutionFileRunInput>>,
    ) -> Result<tonic::Response<Self::RunDownloadExecutionFileStream>, tonic::Status> {
        let reply =
            dokime_operations_next::client::run_call::finite(
                &self.context,
                request.into_inner(),
                |input| {
                    input.value.map(|value| match value {
                        v1::content_download_execution_file_run_input::Value::Request(request) => {
                            dokime_operations_next::client::run_call::Input::Request(request)
                        }
                        v1::content_download_execution_file_run_input::Value::Cancel(_) => {
                            dokime_operations_next::client::run_call::Input::Cancel
                        }
                        v1::content_download_execution_file_run_input::Value::Finish(_) => {
                            dokime_operations_next::client::run_call::Input::Finish
                        }
                    })
                },
                |context, request| async move {
                    dokime_operations_next::client::content_download_execution_file::handle(&context, request).await
                },
                |frame| v1::ContentDownloadExecutionFileRunOutput {
                    value: frame.value.map(|value| match value {
                        v1::content_download_execution_file_output::Value::Item(item) => {
                            v1::content_download_execution_file_run_output::Value::Item(item)
                        }
                        v1::content_download_execution_file_output::Value::Response(reply) => {
                            v1::content_download_execution_file_run_output::Value::Response(reply)
                        }
                    }),
                },
                || v1::ContentDownloadExecutionFileRunOutput {
                    value: Some(v1::content_download_execution_file_run_output::Value::Response(v1::ClientEmpty {})),
                },
            )
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type FollowExecutionFileStream = dokime_operations_next::client::Output<v1::ContentFollowExecutionFileOutput>;
    async fn follow_execution_file(
        &self, request: tonic::Request<v1::ContentFollowExecutionFileRequest>,
    ) -> Result<tonic::Response<Self::FollowExecutionFileStream>, tonic::Status> {
        let reply = dokime_operations_next::client::content_follow_execution_file::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunFollowExecutionFileStream = dokime_operations_next::client::Output<v1::ContentFollowExecutionFileRunOutput>;
    async fn run_follow_execution_file(
        &self, request: tonic::Request<tonic::Streaming<v1::ContentFollowExecutionFileRunInput>>,
    ) -> Result<tonic::Response<Self::RunFollowExecutionFileStream>, tonic::Status> {
        let reply=dokime_operations_next::client::run_call::finite(&self.context,request.into_inner(),|input|input.value.map(|value|match value {v1::content_follow_execution_file_run_input::Value::Request(request)=>dokime_operations_next::client::run_call::Input::Request(request),v1::content_follow_execution_file_run_input::Value::Cancel(_)=>dokime_operations_next::client::run_call::Input::Cancel,v1::content_follow_execution_file_run_input::Value::Finish(_)=>dokime_operations_next::client::run_call::Input::Finish}),|context,request|async move{dokime_operations_next::client::content_follow_execution_file::handle(&context,request).await},
                |frame|v1::ContentFollowExecutionFileRunOutput {value:frame.value.map(|value|match value{v1::content_follow_execution_file_output::Value::Item(item)=>v1::content_follow_execution_file_run_output::Value::Item(item),v1::content_follow_execution_file_output::Value::Response(reply)=>v1::content_follow_execution_file_run_output::Value::Response(reply)})},
                ||v1::ContentFollowExecutionFileRunOutput {value:Some(v1::content_follow_execution_file_run_output::Value::Response(v1::ClientEmpty {}))}).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_artifacts(
        &self, request: tonic::Request<v1::ContentListArtifactsRequest>,
    ) -> Result<tonic::Response<v1::ListArtifactsReply>, tonic::Status> {
        let reply = dokime_operations_next::client::content_list_artifacts::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListArtifactsStream = dokime_operations_next::client::Output<v1::ContentListArtifactsRunOutput>;
    async fn run_list_artifacts(
        &self, request: tonic::Request<tonic::Streaming<v1::ContentListArtifactsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListArtifactsStream>, tonic::Status> {
        let reply = dokime_operations_next::client::run_call::unary(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::content_list_artifacts_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::content_list_artifacts_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::content_list_artifacts_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::content_list_artifacts::handle(&context, request).await },
            |reply| v1::ContentListArtifactsRunOutput {
                value: Some(v1::content_list_artifacts_run_output::Value::Response(reply)),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_execution_files(
        &self, request: tonic::Request<v1::ContentListExecutionFilesRequest>,
    ) -> Result<tonic::Response<v1::ListExecutionFilesReply>, tonic::Status> {
        let reply = dokime_operations_next::client::content_list_execution_files::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListExecutionFilesStream = dokime_operations_next::client::Output<v1::ContentListExecutionFilesRunOutput>;
    async fn run_list_execution_files(
        &self, request: tonic::Request<tonic::Streaming<v1::ContentListExecutionFilesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListExecutionFilesStream>, tonic::Status> {
        let reply = dokime_operations_next::client::run_call::unary(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::content_list_execution_files_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::content_list_execution_files_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::content_list_execution_files_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::content_list_execution_files::handle(&context, request).await },
            |reply| v1::ContentListExecutionFilesRunOutput {
                value: Some(v1::content_list_execution_files_run_output::Value::Response(reply)),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn read_execution_file(
        &self, request: tonic::Request<v1::ContentReadExecutionFileRequest>,
    ) -> Result<tonic::Response<v1::ExecutionFileWindow>, tonic::Status> {
        let reply = dokime_operations_next::client::content_read_execution_file::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunReadExecutionFileStream = dokime_operations_next::client::Output<v1::ContentReadExecutionFileRunOutput>;
    async fn run_read_execution_file(
        &self, request: tonic::Request<tonic::Streaming<v1::ContentReadExecutionFileRunInput>>,
    ) -> Result<tonic::Response<Self::RunReadExecutionFileStream>, tonic::Status> {
        let reply = dokime_operations_next::client::run_call::unary(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::content_read_execution_file_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::content_read_execution_file_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::content_read_execution_file_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::content_read_execution_file::handle(&context, request).await },
            |reply| v1::ContentReadExecutionFileRunOutput {
                value: Some(v1::content_read_execution_file_run_output::Value::Response(reply)),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn upload_artifact(
        &self, request: tonic::Request<tonic::Streaming<v1::ContentUploadArtifactInput>>,
    ) -> Result<tonic::Response<v1::ArtifactMetadata>, tonic::Status> {
        let reply = dokime_operations_next::client::content_upload_artifact::handle(&self.context, request.into_inner(), |input| {
            input.value.map(|value| match value {
                v1::content_upload_artifact_input::Value::Request(_) => {
                    dokime_operations_next::client::content_upload_artifact::Input::Request
                }
                v1::content_upload_artifact_input::Value::Item(item) => {
                    dokime_operations_next::client::content_upload_artifact::Input::Item(item)
                }
            })
        })
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunUploadArtifactStream = dokime_operations_next::client::Output<v1::ContentUploadArtifactRunOutput>;
    async fn run_upload_artifact(
        &self, request: tonic::Request<tonic::Streaming<v1::ContentUploadArtifactRunInput>>,
    ) -> Result<tonic::Response<Self::RunUploadArtifactStream>, tonic::Status> {
        let reply = dokime_operations_next::client::content_upload_artifact::handle(&self.context, request.into_inner(), |input| {
            input.value.map(|value| match value {
                v1::content_upload_artifact_run_input::Value::Request(_) => {
                    dokime_operations_next::client::content_upload_artifact::Input::Request
                }
                v1::content_upload_artifact_run_input::Value::Item(item) => {
                    dokime_operations_next::client::content_upload_artifact::Input::Item(item)
                }
                v1::content_upload_artifact_run_input::Value::Cancel(_) => {
                    dokime_operations_next::client::content_upload_artifact::Input::Cancel
                }
                v1::content_upload_artifact_run_input::Value::Finish(_) => {
                    dokime_operations_next::client::content_upload_artifact::Input::Finish
                }
            })
        })
        .await
        .map_err(dokime_operations_next::client::status)?;
        let (sender, output) = tokio::sync::mpsc::channel(self.context.settings.stream_buffer);
        sender
            .send(Ok(v1::ContentUploadArtifactRunOutput {
                value: Some(v1::content_upload_artifact_run_output::Value::Response(reply)),
            }))
            .await
            .map_err(|_| tonic::Status::cancelled("Upload Receiver Closed"))?;
        Ok(tonic::Response::new(tokio_stream::wrappers::ReceiverStream::new(output)))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
