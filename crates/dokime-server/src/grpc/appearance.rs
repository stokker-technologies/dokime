/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod assets;
mod convert;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::operations::{Context, Operation, StreamOutput};
use prost::Message;
use tonic::Status;

// Local Imports
use crate::grpc::transcode::Transcode;
use dokime::RuntimeHandle;
use dokime::domain::operations::appearance as wire;
use dokime::domain::operations::content;

/* === Constants === */

const SERVER_APPEARANCE_ASSET_BYTES: u64 = 8_388_608;
const SERVER_APPEARANCE_SOURCE_BYTES: u64 = 4_194_304;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    pub runtime: RuntimeHandle,
    pub max_message_bytes: usize,
}

struct Download {
    resources: Arc<Resources>,
}

struct Read {
    resources: Arc<Resources>,
}

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum Operations {
    Read(Read),
    Download(Download),
}

/* === Traits === */

/* === Implementations === */

impl Operation for Read {
    type Declaration = wire::GetAppearance;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _: &mut wire::GetAppearance, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::appearance::wire::AppearanceSnapshot>, Status> {
        let session = self.resources.runtime.storage().clone();
        let captured =
            self.resources.runtime.capture_appearance(source_budget(self.resources.max_message_bytes)).map_err(assets::native_failure)?;

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
        if frame.wire_len()? > self.resources.max_message_bytes {
            return Err(failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge));
        }
        Ok(dokime::harmos::operations::Response::from_owner(frame.owner()))
    }
}

impl Operation for Download {
    type Declaration = content::DownloadAppearanceAsset;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime::domain::model::content::wire::AppearanceAssetFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, call_context: Self::Context<'a>, request: &mut content::DownloadAppearanceAsset, output: &mut Self::Endpoints,
    ) -> Result<dokime::harmos::operations::Response<()>, Status> {
        let revoked = crate::grpc::access::revocation(&call_context);
        crate::grpc::access::during(revoked, async {
            assets::download(&self.resources.runtime, std::mem::take(request), self.resources.max_message_bytes, output).await?;
            Ok(().into())
        })
        .await
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, max_message_bytes: usize) -> Result<(), super::Error> {
    runtime.operations().register_wire_operations::<Operations, Resources>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            max_message_bytes,
        }),
    )?;
    Ok(())
}

pub(super) fn historical(
    frame: dokime::domains::installed_plugins::RuntimeAppearanceFrame,
) -> Result<dokime::domain::model::appearance::wire::AppearanceSnapshot, Status> {
    convert::appearance(frame, Vec::new()).map_err(|_| failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge))
}

/* === Private Functions === */

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

/* === Tests === */
