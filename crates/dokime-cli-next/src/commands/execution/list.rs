/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read or Update the Host Through the SDK

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroU32;

// Third Party Imports

// Local Imports
use dokime_protocol::client;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long)]
    page_size: Option<NonZeroU32>,

    #[arg(long)]
    cycle: Option<String>,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let client = settings.sdk(endpoint, false).await?;
    let mut request = client::ExecutionsListRequest {
        parent: args.cycle.unwrap_or_default(),
        page: Some(dokime::client::PageRequest {
            page_size: args.page_size.map(NonZeroU32::get).unwrap_or_else(|| u32::try_from(settings.page_items.get()).unwrap_or(u32::MAX)),
            ..Default::default()
        }),
        ..Default::default()
    };
    loop {
        let page = client.executions.list(settings.request(request.clone())).await?.into_inner();
        if request.at.is_some() && page.resolved_at != request.at {
            return Err("List Executions: Retained Cut Changed During Continuation".into());
        }
        crate::output::print(&page, args.json)?;
        if page.next_page_token.is_empty() {
            break;
        }
        request.at = page.resolved_at;
        request.page.as_mut().unwrap().page_token = page.next_page_token;
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
