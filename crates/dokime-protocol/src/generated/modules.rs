// @generated from authoritative protobuf packages.
pub mod dokime {
pub mod client {
pub mod v1 {
include!("dokime.client.v1.rs");
pub mod backend {
include!("dokime.client.v1.backend.rs");
}
pub mod content {
include!("dokime.client.v1.content.rs");
}
pub mod evidence {
include!("dokime.client.v1.evidence.rs");
}
pub mod executions {
include!("dokime.client.v1.executions.rs");
}
pub mod media {
include!("dokime.client.v1.media.rs");
}
pub mod plots {
include!("dokime.client.v1.plots.rs");
}
pub mod plugins {
include!("dokime.client.v1.plugins.rs");
}
pub mod prompts {
include!("dokime.client.v1.prompts.rs");
}
pub mod storage {
include!("dokime.client.v1.storage.rs");
}
}
}
pub mod plugin {
pub mod host {
pub mod v1 {
include!("dokime.plugin.host.v1.rs");
pub mod content {
include!("dokime.plugin.host.v1.content.rs");
}
pub mod evidence {
include!("dokime.plugin.host.v1.evidence.rs");
}
pub mod executions {
include!("dokime.plugin.host.v1.executions.rs");
}
pub mod files {
include!("dokime.plugin.host.v1.files.rs");
}
}
}
pub mod v1 {
include!("dokime.plugin.v1.rs");
}
}
pub mod v1 {
include!("dokime.v1.rs");
}
}
