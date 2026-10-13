/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod analysis;
pub(crate) mod choices;
pub(crate) mod invoke;
pub(crate) mod publications;
pub(crate) mod remote;
pub(crate) mod status;

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
#[dokime::catalog]
pub(crate) enum Operations {
    PeakToPeak(dokime::analysis::Served<analysis::PeakToPeak, crate::Resources>),
    ListChoices(choices::ListChoices),
    RunCommand(invoke::RunCommand),
    PublishRecord(publications::PublishRecord),
    GetStatus(status::GetStatus),
    RemoteUnary(remote::DynamicUnaryHandler),
    RemoteDownload(remote::DynamicDownloadHandler),
    RemoteUpload(remote::DynamicUploadHandler),
    RemoteDuplex(remote::DynamicDuplexHandler),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
