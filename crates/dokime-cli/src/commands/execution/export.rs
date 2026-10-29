/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Export Execution Evidence With Explicit Interrupt Ownership

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    input: super::super::evidence::Args,

    #[command(flatten)]
    recovery: super::recovery::Args,
}

pub(crate) async fn run(args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    super::super::evidence::run(args.input, dokime::domain::model::exports::EvidenceSubject::Execution, Some(args.recovery), settings).await
}
