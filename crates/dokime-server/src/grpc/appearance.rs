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
pub(super) use assets::download;
pub(super) use convert::historical_memory;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use prost::Message;
use tonic::{Request, Response, Status};
// Local Imports
use dokime_app::RuntimeHandle;
use dokime_protocol::v1::{self, appearance_server::Appearance};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub runtime: RuntimeHandle,
    pub max_message_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl Appearance for Service {
    async fn get_appearance(
        &self,
        _request: Request<v1::GetAppearanceRequest>,
    ) -> Result<Response<v1::AppearanceSnapshot>, Status> {
        let captured = self
            .runtime
            .capture_appearance(source_budget(self.max_message_bytes))
            .map_err(|_| failure(v1::AppearanceFailureKind::TooLarge))?;
        let logo = match captured.frame.logo.as_ref() {
            Some(logo) => {
                let source = captured
                    .logo
                    .ok_or_else(|| failure(v1::AppearanceFailureKind::AssetMissing))?;
                let asset = assets::capture(
                    source,
                    logo.plugin_id.to_string(),
                    logo.asset_id.to_string(),
                )
                .await?;
                Some(v1::AppearanceLogo {
                    asset: Some(asset.metadata),
                    alt: logo.alt.clone(),
                })
            }
            None => None,
        };
        let frame = convert::appearance(captured.frame, logo)
            .map_err(|_| failure(v1::AppearanceFailureKind::TooLarge))?;
        if frame.encoded_len() > self.max_message_bytes {
            return Err(failure(v1::AppearanceFailureKind::TooLarge));
        }
        Ok(Response::new(frame))
    }
}

/* === Public Functions === */
pub(super) fn historical(
    frame: dokime_app::capabilities::installed_plugins::RuntimeAppearanceFrame,
) -> Result<v1::AppearanceSnapshot, Status> {
    convert::historical_memory(&frame).map_err(|_| failure(v1::AppearanceFailureKind::TooLarge))?;
    convert::appearance(frame, None).map_err(|_| failure(v1::AppearanceFailureKind::TooLarge))
}

/* === Private Functions === */
fn source_budget(max_message_bytes: usize) -> usize {
    max_message_bytes.min(4 * 1024 * 1024)
}

fn failure(kind: v1::AppearanceFailureKind) -> Status {
    use v1::AppearanceFailureKind::*;
    let (code, message) = match kind {
        AssetMissing => (
            tonic::Code::NotFound,
            "appearance asset is unavailable; refresh appearance",
        ),
        AssetChanged => (
            tonic::Code::FailedPrecondition,
            "appearance asset changed; refresh appearance",
        ),
        TooLarge => (
            tonic::Code::ResourceExhausted,
            "appearance exceeds its size budget",
        ),
        InvalidReference => (
            tonic::Code::InvalidArgument,
            "invalid appearance asset reference",
        ),
        _ => (tonic::Code::Unavailable, "appearance asset cannot be read"),
    };
    let envelope = tonic_types::Status {
        code: code as i32,
        message: message.into(),
        details: vec![prost_types::Any {
            type_url: "type.googleapis.com/dokime.v1.AppearanceFailure".into(),
            value: v1::AppearanceFailure { kind: kind as i32 }.encode_to_vec(),
        }],
    };
    Status::with_details(code, message, envelope.encode_to_vec().into())
}

/* === Tests === */
