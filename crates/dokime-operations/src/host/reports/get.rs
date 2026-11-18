/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reports Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::FileReference;
use harmos_runtime::Identifier;
use prost::Message;

// Local Imports
use super::PreparedReport;
use crate::host::storage::administration;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::ReportMetadata;
use dokime_protocol::host::ReportOrigin;
use dokime_protocol::host::ReportsGetRequest;
use dokime_protocol::host::ReportsGetResponse;
use dokime_protocol::host::report_subject::Kind;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Reports/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ReportsGetRequest) -> Result<ReportsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let mut temporary = context.host.temporary_report(&request.name)?;
        if temporary.is_none() && !request.name.is_empty() {
            temporary = administration::message::<PreparedReport>(context, &request.name).await?.0.file;
        }
        if request.name.is_empty()
            && let Some(Kind::Cycle(cycle)) = request.subject.as_ref().and_then(|subject| subject.kind.as_ref())
        {
            let scope = Scope::Cycle(cycle).identifier()?;
            let (entries, _) = administration::scan(context, "prepared-report/", None).await?;
            for entry in entries.into_iter().rev() {
                if entry.scope == scope {
                    temporary = PreparedReport::decode(entry.payload.as_slice())?.file;
                    break;
                }
            }
        }
        let scope = match temporary.as_ref().and_then(|file| file.metadata.as_ref()).and_then(|metadata| metadata.scope.as_ref()) {
            Some(scope) => Identifier::new(scope.clone())
                .map_err(|source| Error::Invalid(format!("Get Report: Restore Its Storage Scope: {source}")))?,
            None => match request.subject.as_ref().and_then(|subject| subject.kind.as_ref()) {
                Some(Kind::Execution(execution)) => Scope::Execution(execution).identifier()?,
                Some(Kind::Cycle(cycle)) => Scope::Cycle(cycle).identifier()?,
                None => return Err(Error::Invalid("Get Report: Select Its Published Subject or Prepared Report".into())),
            },
        };
        let resource = format!(
            "report/{}/{}",
            request.name,
            crate::host::views::hex(&crate::host::paging::bytes(&ReportsGetRequest { at: None, ..request.clone() }))
        );
        let (mut reply, cut) = crate::host::views::read(context, scope, &resource, request.at.as_ref(), async {
            let execution = match request.subject.as_ref().and_then(|subject| subject.kind.as_ref()) {
                Some(Kind::Execution(execution)) => Some(execution),
                _ => None,
            };
            let report = execution
                .map(|execution| context.host.store().reports(execution))
                .transpose()?
                .unwrap_or_default()
                .into_iter()
                .find(|report| request.name.is_empty() || report.identifier == request.name);
            let Some(report) = report else {
                let reference = temporary.ok_or_else(|| {
                    Error::NotFound(
                        "Get Report: Use Download Report to Create and Save a Report First (Reports/Create for API Clients)".into(),
                    )
                })?;
                return Ok(ReportsGetResponse {
                    file: Some(reference),
                    report: Some(ReportMetadata {
                        identifier: request.name.clone(),
                        test_cycle_id: match request.subject.as_ref().and_then(|subject| subject.kind.as_ref()) {
                            Some(Kind::Cycle(cycle)) => cycle.clone(),
                            Some(Kind::Execution(execution)) => context.host.store().metadata(execution)?.execution.cycle,
                            None => String::new(),
                        },
                        test_execution_id: execution.cloned(),
                        origin: ReportOrigin::Current as i32,
                        ..Default::default()
                    }),
                    resolved_at: None,
                });
            };
            let key = format!("published-report/{}", report.identifier);
            let (reference, revision) = administration::message::<FileReference>(context, &key).await?;
            if revision.is_empty() {
                return Err(Error::NotFound(
                    "Get Report: Use Download Report to Create and Save a Report First (Reports/Create for API Clients)".into(),
                ));
            }
            let execution = execution.unwrap();
            let cut = context.host.store().metadata(execution)?;
            Ok(ReportsGetResponse {
                report: Some(ReportMetadata {
                    test_cycle_id: cut.execution.cycle,
                    test_execution_id: Some(execution.clone()),
                    identifier: report.identifier,
                    metadata: report.document.and_then(|document| document.metadata),
                    origin: ReportOrigin::Sealed as i32,
                    ..Default::default()
                }),
                file: Some(reference),
                resolved_at: None,
            })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ReportsGetRequest, ReportsGetResponse, context, "dokime.host.v1", "Reports", "Get");
