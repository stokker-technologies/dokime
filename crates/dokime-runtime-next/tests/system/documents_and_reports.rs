/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Document Rendering and Publication Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_protocol::{builders::Plot, v1};
use dokime_runtime_next::{Host, RenderSettings};
use std::num::NonZeroUsize;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn documents_and_reports() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Seal Evidence and Take a Fixed Snapshot
        let root = super::support::tempdir()?;
        let host = Host::open(root.path(), super::support::settings())?;
        let entry = host.add_managed(&super::support::execution_fixture()?).await?;
        let document = host.catalog(&entry)?.suites[0].templates[0].documents[0].clone();
        let origin = host.materialize(&entry, "execution.capture", super::support::parameters(23.0, 0, true)).await?;
        let cycle = host.start_cycle(vec![origin]).await?;
        let execution = cycle.executions[0].clone();
        let prompt = host.next_prompt(&execution).await?;
        super::support::answer(&host, &execution, prompt)?;
        let result = cycle.wait().await?;
        let fixed = host.snapshot(&execution)?;
        let before = host.execution(&execution)?;

        // Render Real HTML with Bounded Stored-Sample Queries and No Publication
        let settings = RenderSettings {
            page_size: NonZeroUsize::new(2).unwrap(),
            display_points: NonZeroUsize::new(3).unwrap(),
        };
        let output = host.render_document(&fixed, document.clone(), settings)?;
        let html = output.html()?;
        let text = std::str::from_utf8(&html)?;
        assert!(text.contains("<svg") && text.contains("<polyline"));
        assert!(text.contains(&format!("data-count=\"{}\"", result.executions[0].samples.len())));
        assert!(output.path().is_file());
        assert!(host.reports(&execution)?.is_empty());
        assert_eq!(host.execution(&execution)?, before);
        let one_at_a_time = host.render_document(
            &fixed,
            document.clone(),
            RenderSettings {
                page_size: NonZeroUsize::new(1).unwrap(),
                ..settings
            },
        )?;
        assert_eq!(one_at_a_time.html()?, html);

        // Refuse an Unresolvable Plot Without Publishing Any Partial Output
        let invalid = v1::Document::new("Missing Field").section(
            v1::Section::new("Measurements").plot(Plot::time_series(v1::FieldReference::new("execution.reading").field("absent"))),
        );
        assert!(host.render_document(&fixed, invalid, settings).is_err());
        assert!(host.reports(&execution)?.is_empty());

        // Publish Explicitly and Reopen the Frozen Document and HTML
        let published = host.publish_report(&output)?;
        assert_eq!(published.document, Some(document));
        assert_eq!(published.html, html);
        assert_eq!(host.reports(&execution)?, vec![published.clone()]);
        assert_eq!(host.execution(&execution)?, before);
        let temporary = output.path().to_owned();
        drop(output);
        assert!(!temporary.exists());
        host.shutdown().await?;
        drop(host);
        let host = Host::open(root.path(), super::support::settings())?;
        assert_eq!(host.reports(&execution)?, vec![published]);
        assert_eq!(host.execution(&execution)?, before);
        host.shutdown().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
