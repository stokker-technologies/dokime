// @generated from authoritative protobuf packages.
pub mod dokime {
pub mod context {
pub mod v1 {
include!("dokime.context.v1.rs");
pub mod content {
include!("dokime.context.v1.content.rs");
}
pub mod evidence {
include!("dokime.context.v1.evidence.rs");
}
pub mod executions {
include!("dokime.context.v1.executions.rs");
}
pub mod files {
include!("dokime.context.v1.files.rs");
}
}
}
pub mod plugin {
pub mod v1 {
include!("dokime.plugin.v1.rs");
}
}
pub mod v1 {
include!("dokime.v1.rs");
pub mod backend {
include!("dokime.v1.backend.rs");
}
pub mod catalog {
include!("dokime.v1.catalog.rs");
}
pub mod content {
include!("dokime.v1.content.rs");
}
pub mod evidence {
include!("dokime.v1.evidence.rs");
}
pub mod executions {
include!("dokime.v1.executions.rs");
}
pub mod media {
include!("dokime.v1.media.rs");
}
pub mod plots {
include!("dokime.v1.plots.rs");
}
pub mod plugins {
include!("dokime.v1.plugins.rs");
}
pub mod prompts {
include!("dokime.v1.prompts.rs");
}
pub mod storage {
include!("dokime.v1.storage.rs");
}
}
}
