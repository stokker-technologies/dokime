/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared Capture Suite

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::prelude::*;
use dokime::templates::Context;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct CaptureSuite;
pub struct ExamplePlugin;
pub struct ConfirmationTestTemplate;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ConfirmationTestTemplate {
    pub fn confirmation() -> Record<bool> {
        Record::of(Self::confirmation, "Confirmation")
    }
    pub fn template() -> Template {
        Template::of::<Self>()
            .record(Self::confirmation())
            .step(Step::new("Confirm", Self::confirm))
            .criterion(Criterion::new("Confirmed", Self::confirmed))
    }
    async fn confirm(ctx: &Context) -> Result<()> {
        let answer = ctx.prompt("Continue?", &["Yes", "No"]).await?;
        ctx.record(&Self::confirmation(), answer == "Yes").await?;
        Ok(())
    }
    async fn confirmed(ctx: &Context) -> Result<bool> {
        ctx.records(&Self::confirmation()).step(Self::confirm).last().await
    }
}

/* === Public Functions === */

pub fn plugin() -> Plugin {
    Plugin::of::<ExamplePlugin>("0.1.0")
        .suite(Suite::of::<CaptureSuite>().identifier("example.capture").template(ConfirmationTestTemplate::template()))
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{CaptureSuite, ConfirmationTestTemplate, ExamplePlugin, plugin};
    use dokime::RecordContracts;
    use dokime::encoding::Message;
    use dokime::prelude::*;
    use dokime::templates::Context;
    use dokime::testing::Harness;

    #[test]
    fn marker_types_describe_the_grouped_capture_contract() -> Result<()> {
        let description = plugin().declarations()?;
        let section = description
            .sections
            .iter()
            .find(|section| section.r#type.ends_with(".Suites"))
            .ok_or_else(|| Error::Invalid("Describe Plugin: Include Its Suites".into()))?;
        let suites = dokime::Suites::decode(section.payload.as_slice()).map_err(|error| Error::Invalid(error.to_string()))?;
        let suite = &suites.suites[0];
        assert_eq!(suite.metadata.as_ref().unwrap().identifier, "example.capture");
        let template = &suite.templates[0];
        assert_eq!(template.metadata.as_ref().unwrap().identifier, "example.capture.confirmation");
        assert_eq!(template.steps[0].operation, "example.capture.confirmation.confirm");
        assert!(description.operations.iter().any(|operation| operation.id == "example.capture.confirmation.confirmed"));
        Ok(())
    }

    #[tokio::test]
    async fn plugin_state_is_shared_and_suite_overrides_bind_before_execution() -> Result<()> {
        struct State {
            value: bool,
        }
        async fn read(ctx: &Context) -> Result<()> {
            let value = ctx.plugin::<State>()?.value;
            ctx.record(&ConfirmationTestTemplate::confirmation(), value).await?;
            Ok(())
        }
        let template = Template::of::<ConfirmationTestTemplate>()
            .record(ConfirmationTestTemplate::confirmation())
            .step(Step::new("Read", read))
            .criterion(
                Criterion::new("Confirmed", async |ctx: &Context| ctx.records(&ConfirmationTestTemplate::confirmation()).last().await)
                    .identifier("confirmed"),
            );
        let suite = Suite::of::<CaptureSuite>().template(template).identifier("example.renamed");
        let plugin = Plugin::of::<ExamplePlugin>("0.1.0").state(State { value: true }).suite(suite);
        let report = Harness::new(plugin, "example.renamed.confirmation").run().await?;
        assert!(report.passed());
        assert_eq!(report.records::<bool>("example.renamed.confirmation.confirmation")?, [true]);
        Ok(())
    }

    #[test]
    fn duplicate_cross_kind_and_empty_identifiers_are_refused() {
        for template in [
            Template::of::<()>()
                .identifier("capture")
                .label("Label")
                .record(Record::<bool>::new("same", "One"))
                .record(Record::<String>::new("same", "Two")),
            Template::of::<()>()
                .identifier("capture")
                .label("Label")
                .record(Record::<bool>::new("confirm", "One"))
                .step(Step::new("Confirm", ConfirmationTestTemplate::confirm)),
            Template::of::<()>().identifier("capture").label("Label").record(Record::<bool>::new("", "Empty")),
            Template::of::<()>().identifier("").label("Empty"),
        ] {
            assert!(
                Plugin::of::<()>("0.1.0")
                    .identifier("invalid")
                    .suite(Suite::of::<CaptureSuite>().identifier("invalid.capture").template(template))
                    .declarations()
                    .is_err()
            );
        }
        assert!(
            Plugin::of::<()>("0.1.0")
                .identifier("invalid")
                .suite(Suite::of::<CaptureSuite>().identifier("invalid.capture").template(ConfirmationTestTemplate::template()))
                .suite(Suite::of::<CaptureSuite>().identifier("invalid.capture").template(ConfirmationTestTemplate::template()))
                .declarations()
                .is_err()
        );
    }

    #[tokio::test]
    async fn materialized_instances_keep_declared_handlers_and_qualified_evidence() -> Result<()> {
        async fn read(ctx: &Context) -> Result<()> {
            ctx.record(&ConfirmationTestTemplate::confirmation(), true).await?;
            Ok(())
        }
        async fn accept(ctx: &Context) -> Result<bool> {
            ctx.records(&ConfirmationTestTemplate::confirmation()).last().await
        }
        let template = Template::of::<()>()
            .identifier("channels")
            .label("Channels")
            .record(ConfirmationTestTemplate::confirmation())
            .step(Step::new("Read", read))
            .criterion(Criterion::new("Accept", accept))
            .materialize(|mut case, _| {
                if case.steps[0].metadata.as_ref().unwrap().identifier != "read" {
                    return Err(Error::Invalid("Materialize: Supply Local Step Identifiers".into()));
                }
                let mut step = case.steps[0].clone();
                case.steps[0].metadata.as_mut().unwrap().identifier = "read_1".into();
                step.metadata.as_mut().unwrap().identifier = "read_2".into();
                case.steps.push(step);
                let mut criterion = case.criteria[0].clone();
                case.criteria[0].metadata.as_mut().unwrap().identifier = "accept_1".into();
                criterion.metadata.as_mut().unwrap().identifier = "accept_2".into();
                case.criteria.push(criterion);
                Ok(case)
            });
        let plugin = Plugin::of::<()>("0.1.0")
            .identifier("channels")
            .suite(Suite::of::<CaptureSuite>().identifier("channels.capture").template(template));
        let report = Harness::new(plugin, "channels.capture.channels").run().await?;
        assert!(report.passed(), "{report:?}");
        assert_eq!(
            report.criteria().iter().map(|criterion| criterion.criterion.as_str()).collect::<Vec<_>>(),
            ["channels.capture.channels.accept_1", "channels.capture.channels.accept_2"]
        );
        assert_eq!(report.records::<bool>("channels.capture.channels.confirmation")?, [true, true]);
        Ok(())
    }

    #[test]
    fn reused_declarations_and_explicit_identifiers_stay_local() -> Result<()> {
        let record = Record::<bool>::new("exact", "A Different Label");
        for suite in ["names.accuracy", "names.production"] {
            let template = Template::of::<()>()
                .identifier("voltage")
                .label("Another Label")
                .record(record.clone())
                .step(Step::new("Completely Different Label", ConfirmationTestTemplate::confirm).identifier("take"));
            let declarations = Plugin::of::<()>("0.1.0")
                .identifier("names")
                .suite(Suite::of::<()>().identifier(suite).label("Label").template(template))
                .declarations()?;
            let section = declarations.sections.iter().find(|section| section.r#type.ends_with(".RecordContracts")).unwrap();
            let records = RecordContracts::decode(section.payload.as_slice()).unwrap();
            assert_eq!(records.records[0].metadata.as_ref().unwrap().identifier, format!("{suite}.voltage.exact"));
            assert!(declarations.operations.iter().any(|operation| operation.id == format!("{suite}.voltage.take")));
        }
        assert_eq!(record.identifier(), "exact");
        Ok(())
    }

    #[tokio::test]
    async fn unknown_answers_and_empty_record_selections_fail() -> Result<()> {
        let report =
            Harness::new(plugin(), "example.capture.confirmation").answer("example.capture.confirmation.question", "unknown").run().await?;
        assert!(report.failure().is_some());
        assert!(report.records::<bool>("example.capture.confirmation.confirmation")?.is_empty());
        let template = Template::of::<ConfirmationTestTemplate>()
            .label("Capture")
            .record(ConfirmationTestTemplate::confirmation())
            .criterion(Criterion::new("Confirmed", ConfirmationTestTemplate::confirmed));
        let plugin =
            Plugin::of::<()>("0.1.0").identifier("empty").suite(Suite::of::<CaptureSuite>().identifier("empty.capture").template(template));
        let report = Harness::new(plugin, "empty.capture.confirmation").run().await?;
        assert!(report.failure().is_some());
        assert!(report.criteria().is_empty());
        Ok(())
    }
}
