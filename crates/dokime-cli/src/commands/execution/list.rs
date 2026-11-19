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
use dokime_client_internal::ExecutionsListOptions;
use dokime::PageRequest;

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
    let client = settings.sdk(endpoint).await?;
    let parent = args.cycle.unwrap_or_default();
    let mut options = ExecutionsListOptions {
        context: settings.context()?,
        page: Some(PageRequest {
            page_size: args.page_size.map(NonZeroU32::get).unwrap_or_else(|| u32::try_from(settings.page_items.get()).unwrap_or(u32::MAX)),
            ..Default::default()
        }),
        ..Default::default()
    };
    loop {
        let at = options.at.clone();
        let mut page_request = options.page.clone().unwrap_or_default();
        let page = client.executions.list_with(&parent, Default::default(), Default::default(), Default::default(), options).await?;
        if at.is_some() && page.resolved_at != at {
            return Err("List Executions: Retained Cut Changed During Continuation".into());
        }
        crate::output::print(&page, args.json)?;
        if page.next_page_token.is_empty() {
            break;
        }
        page_request.page_token = page.next_page_token;
        options = ExecutionsListOptions {
            context: settings.context()?,
            at: page.resolved_at,
            page: Some(page_request),
        };
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
