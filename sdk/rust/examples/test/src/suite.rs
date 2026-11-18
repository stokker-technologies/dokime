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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Capture;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Capture {
    pub fn confirmation() -> Record<bool> {
        Record::of(Self::confirmation, "Confirmation")
    }
    pub fn template() -> Template {
        Template::of::<Self>("Capture")
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
    Plugin::new("example", "0.1.0").suite(Suite::single(Capture::template()))
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Capture, plugin};
    use dokime::encoding::Message;
    use dokime::prelude::*;
    use dokime::testing::Harness;
    use dokime::{RecordContracts, Suites};

    #[tokio::test]
    async fn single_and_grouped_capture_match_the_python_record_contract() -> Result<()> {
        for (plugin, template) in [
            (plugin(), "capture"),
            (Plugin::new("example", "0.1.0").suite(Suite::new("example", "Example").template(Capture::template())), "example.capture"),
        ] {
            let declarations = plugin.declarations()?;
            let section = declarations.sections.iter().find(|section| section.r#type.ends_with(".RecordContracts")).unwrap();
            let contracts = RecordContracts::decode(section.payload.as_slice()).unwrap();
            assert_eq!(contracts.records[0].message, "google.protobuf.BoolValue");
            assert_eq!(contracts.records[0].metadata.as_ref().unwrap().identifier, format!("{template}.confirmation"));
            let section = declarations.sections.iter().find(|section| section.r#type.ends_with(".Suites")).unwrap();
            let suites = Suites::decode(section.payload.as_slice()).unwrap();
            let bound = &suites.suites[0].templates[0];
            assert_eq!(bound.metadata.as_ref().unwrap().identifier, template);
            assert_eq!(bound.steps[0].metadata.as_ref().unwrap().identifier, format!("{template}.confirm"));
            assert_eq!(bound.steps[0].operation, format!("{template}.confirm"));
        }
        let report = Harness::new(plugin(), "capture").run().await?;
        assert!(report.passed(), "{report:?}");
        assert_eq!(report.records::<bool>("capture.confirmation")?, [true]);
        let report = Harness::new(plugin(), "capture").answer("capture.question", "1").run().await?;
        assert!(!report.passed());
        assert_eq!(report.records::<bool>("capture.confirmation")?, [false]);
        assert!(report.failure().is_none());
        Ok(())
    }

    #[test]
    fn duplicate_cross_kind_and_empty_identifiers_are_refused() {
        for template in [
            Template::new("capture", "Label").record(Record::<bool>::new("same", "One")).record(Record::<String>::new("same", "Two")),
            Template::new("capture", "Label").record(Record::<bool>::new("confirm", "One")).step(Step::new("Confirm", Capture::confirm)),
            Template::new("capture", "Label").record(Record::<bool>::new("", "Empty")),
            Template::new("", "Empty"),
        ] {
            assert!(Plugin::new("invalid", "0.1.0").suite(Suite::single(template)).declarations().is_err());
        }
        assert!(
            Plugin::new("invalid", "0.1.0")
                .suite(Suite::single(Capture::template()))
                .suite(Suite::single(Capture::template()))
                .declarations()
                .is_err()
        );
    }

    #[tokio::test]
    async fn materialized_instances_keep_declared_handlers_and_qualified_evidence() -> Result<()> {
        async fn read(ctx: &Context) -> Result<()> {
            ctx.record(&Capture::confirmation(), true).await?;
            Ok(())
        }
        async fn accept(ctx: &Context) -> Result<bool> {
            ctx.records(&Capture::confirmation()).last().await
        }
        let template = Template::new("channels", "Channels")
            .record(Capture::confirmation())
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
        let plugin = Plugin::new("channels", "0.1.0").suite(Suite::single(template));
        let report = Harness::new(plugin, "channels").run().await?;
        assert!(report.passed(), "{report:?}");
        assert_eq!(
            report.criteria().iter().map(|criterion| criterion.criterion.as_str()).collect::<Vec<_>>(),
            ["channels.accept_1", "channels.accept_2"]
        );
        assert_eq!(report.records::<bool>("channels.confirmation")?, [true, true]);
        Ok(())
    }

    #[test]
    fn reused_declarations_and_explicit_identifiers_stay_local() -> Result<()> {
        let record = Record::<bool>::new("exact", "A Different Label");
        for suite in ["accuracy", "production"] {
            let template = Template::new("voltage", "Another Label")
                .record(record.clone())
                .step(Step::new("Completely Different Label", Capture::confirm).identifier("take"));
            let declarations = Plugin::new("names", "0.1.0").suite(Suite::new(suite, "Label").template(template)).declarations()?;
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
        let report = Harness::new(plugin(), "capture").answer("capture.question", "unknown").run().await?;
        assert!(report.failure().is_some());
        assert!(report.records::<bool>("capture.confirmation")?.is_empty());
        let template =
            Template::of::<Capture>("Capture").record(Capture::confirmation()).criterion(Criterion::new("Confirmed", Capture::confirmed));
        let plugin = Plugin::new("empty", "0.1.0").suite(Suite::single(template));
        let report = Harness::new(plugin, "capture").run().await?;
        assert!(report.failure().is_some());
        assert!(report.criteria().is_empty());
        Ok(())
    }
}
