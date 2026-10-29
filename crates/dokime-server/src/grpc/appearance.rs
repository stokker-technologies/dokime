/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod assets;
mod convert;

use std::sync::Arc;

use dokime::harmos::Context;
use prost::Message;
use tonic::Status;

use crate::grpc::transcode::Transcode;
use dokime::app::LiveRuntime;
use dokime::operations::appearance as wire;
use dokime::operations::content;
use dokime::operations::hosting::{Host, Serve, Served};

const SERVER_APPEARANCE_ASSET_BYTES: u64 = 8_388_608;
const SERVER_APPEARANCE_SOURCE_BYTES: u64 = 4_194_304;

struct Resources {
    pub runtime: LiveRuntime,
    pub max_message_bytes: usize,
}

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(wire::GetAppearance),
    Operation1(content::DownloadAppearanceAsset),
}

impl Serve<wire::GetAppearance> for Resources {
    fn serve<'a>(
        &'a self, _: wire::GetAppearance, _: Context<'a, wire::GetAppearance>,
    ) -> Served<'a, dokime::domain::model::appearance::wire::AppearanceSnapshot> {
        Box::pin(async move {
            let session = dokime::app::live_runtime(&self.runtime).await?.storage().clone();
            let captured = dokime::app::live_runtime(&self.runtime)
                .await?
                .capture_appearance(source_budget(self.max_message_bytes))
                .map_err(assets::native_failure)?;

            let mut logos = dokime::harmos::storage::ReadBatch::new(())
                .try_map(|_| {
                    let mut logos = Vec::new();
                    logos
                        .try_reserve_exact(captured.frame.logos.len())
                        .map_err(|_| dokime::harmos::storage::Error::ResourceExhausted { resource: "appearance logo slots" })?;
                    Ok(logos)
                })
                .map_err(assets::native_failure)?;

            for (logo, source) in captured.frame.logos.iter().zip(captured.logos.iter()) {
                let source =
                    source.as_ref().ok_or_else(|| failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::AssetMissing))?;
                let asset = assets::capture(
                    &session,
                    source,
                    logo.plugin_id.as_str(),
                    logo.asset_id.as_str(),
                    usize::try_from(SERVER_APPEARANCE_ASSET_BYTES).expect("validated appearance asset size"),
                )
                .await?;

                logos = logos
                    .try_map_with(asset.metadata, |mut logos, metadata| {
                        logos.push(dokime::domain::model::appearance::wire::AppearanceLogo {
                            variant: logo.variant,
                            asset: Some(metadata),
                            alt: logo.alt.clone(),
                        });
                        Ok(logos)
                    })
                    .map_err(assets::native_failure)?;
            }
            let frame = captured
                .try_map_with(logos, |captured, logos| convert::appearance(captured.frame, logos))
                .map_err(|_| failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge))?;
            if frame.wire_len()? > self.max_message_bytes {
                return Err(failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge));
            }
            Ok(dokime::harmos::operations::Response::from_owner(frame.owner()))
        })
    }
}

impl Serve<content::DownloadAppearanceAsset> for Resources {
    fn serve<'a>(
        &'a self, request: content::DownloadAppearanceAsset, ctx: Context<'a, content::DownloadAppearanceAsset>,
    ) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async {
                assets::download(&dokime::app::live_runtime(&self.runtime).await?, request, self.max_message_bytes, ctx.streams()).await?;
                Ok(().into())
            })
            .await
        })
    }
}

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, max_message_bytes: usize,
) -> Result<(), super::Error> {
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        max_message_bytes,
    });

    setup.resource::<Host<wire::GetAppearance>>(Host::<wire::GetAppearance>::new(served.clone()));
    setup.resource::<Host<content::DownloadAppearanceAsset>>(Host::<content::DownloadAppearanceAsset>::new(served));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

pub(super) fn historical(
    frame: dokime::domains::installed_plugins::RuntimeAppearanceFrame,
) -> Result<dokime::domain::model::appearance::wire::AppearanceSnapshot, Status> {
    convert::appearance(frame, Vec::new()).map_err(|_| failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge))
}

fn source_budget(max_message_bytes: usize) -> usize {
    max_message_bytes.min(usize::try_from(SERVER_APPEARANCE_SOURCE_BYTES).expect("validated appearance source size"))
}

fn failure(kind: dokime::domain::model::appearance::wire::AppearanceFailureKind) -> Status {
    use dokime::domain::model::appearance::wire::AppearanceFailureKind::*;
    let (code, message) = match kind {
        AssetMissing => (tonic::Code::NotFound, "appearance asset is unavailable; refresh appearance"),
        AssetChanged => (tonic::Code::FailedPrecondition, "appearance asset changed; refresh appearance"),
        TooLarge => (tonic::Code::ResourceExhausted, "appearance exceeds its size budget"),
        InvalidReference => (tonic::Code::InvalidArgument, "invalid appearance asset reference"),
        _ => (tonic::Code::Unavailable, "appearance asset cannot be read"),
    };
    let envelope = tonic_types::Status {
        code: code as i32,
        message: message.into(),
        // A Detail The Transcoder Refuses Is Omitted; The Status Itself Still Reaches The Caller.
        details: crate::grpc::transcode::Transcode::wire_bytes(&dokime::domain::model::appearance::wire::AppearanceFailure { kind })
            .ok()
            .map(|value| prost_types::Any {
                type_url: "type.googleapis.com/dokime.v1.AppearanceFailure".into(),
                value,
            })
            .into_iter()
            .collect(),
    };
    Status::with_details(code, message, envelope.encode_to_vec().into())
}
