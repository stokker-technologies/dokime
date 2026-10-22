/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{Client, Error, ResponseStream, Result};
use dokime_domain::operations::Endpoint;
use dokime_domain::operations::catalog;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    pub fn suites(&self) -> Result<ResponseStream<dokime_domain::model::catalog::wire::SuiteDefinition>> {
        let client = self.clone();
        ResponseStream::spawn(self, move |sender| async move {
            let read = async {
                let mut request = catalog::ListSuites { limit: 100, after_suite_id: String::new(), cursor: None };
                loop {
                    let page = client.suite_page(&request).await?;
                    for identity in page.suites {
                        client.verify(true).await?;
                        let suite = client.definition(catalog::GetSuite { suite_id: identity.suite_id.clone() }).await?;
                        validate_definition(&identity.suite_id, &suite)?;
                        validate_listed_identity(&identity, &suite)?;
                        if sender.send(Ok(suite)).await.is_err() {
                            return Ok(());
                        }
                    }
                    match page.next_after_suite_id {
                        Some(after) => {
                            request.after_suite_id = after;
                            request.cursor = page.cursor;
                        }
                        None => return Ok(()),
                    }
                }
            }
            .await;
            if let Err(error) = read {
                let _ = sender.send(Err(error)).await;
            }
        })
    }

    async fn suite_page(&self, request: &catalog::ListSuites) -> Result<dokime_domain::model::catalog::wire::ListSuitesReply> {
        self.verify(true).await?;
        let page = self.unary::<catalog::ListSuites>(catalog::ListSuites::PATH, request.clone()).await?;
        let page = super::transport::value(page)?;
        validate_page(&self.backend().epoch, request, &page)?;
        Ok(page)
    }

    async fn definition(&self, request: catalog::GetSuite) -> Result<dokime_domain::model::catalog::wire::SuiteDefinition> {
        let reply = self.unary::<catalog::GetSuite>(catalog::GetSuite::PATH, request).await?;
        super::transport::value(reply)
    }

    pub async fn suite(&self, identifier: &str) -> Result<dokime_domain::model::catalog::wire::SuiteDefinition> {
        self.verify(true).await?;
        match self.definition(catalog::GetSuite { suite_id: identifier.into() }).await {
            Ok(suite) => {
                validate_definition(identifier, &suite)?;
                return Ok(suite);
            }
            Err(Error::Rpc(status)) if status.code() == tonic::Code::NotFound => {}
            Err(error) => return Err(error),
        }
        let mut request = catalog::ListSuites { limit: 100, after_suite_id: String::new(), cursor: None };
        let mut matched = None;
        loop {
            let page = self.suite_page(&request).await?;
            for suite in &page.suites {
                if suite.metadata.as_ref().is_some_and(|metadata| metadata.identifier == identifier)
                    && matched.replace(suite.suite_id.clone()).is_some()
                {
                    return Err(Error::Rpc(tonic::Status::invalid_argument("suite identifier is ambiguous; use its registered ID")));
                }
            }
            match page.next_after_suite_id {
                Some(after) => {
                    request.after_suite_id = after;
                    request.cursor = page.cursor;
                }
                None => break,
            }
        }
        let id = matched.ok_or_else(|| Error::Rpc(tonic::Status::not_found("suite not found")))?;
        let suite = self.definition(catalog::GetSuite { suite_id: id.clone() }).await?;
        validate_definition(&id, &suite)?;
        if suite.identity.as_ref().and_then(|identity| identity.metadata.as_ref()).is_none_or(|metadata| metadata.identifier != identifier)
        {
            return Err(invalid("resolved suite identifier changed"));
        }
        Ok(suite)
    }

    // A template is referenced as `<suite id>.<segment>`, so the reference
    // names its own suite; the registered UUID stays on the wire.
    pub async fn template(
        &self, identifier: &str,
    ) -> Result<(dokime_domain::model::catalog::wire::SuiteDefinition, dokime_domain::model::catalog::wire::TemplateDefinition)> {
        let suite = template_suite(identifier)
            .ok_or_else(|| Error::Rpc(tonic::Status::invalid_argument("a template is referenced as `<suite id>.<segment>`")))?;
        let definition = self.suite(suite).await?;
        let template = registered_template(&definition, identifier)?.clone();
        Ok((definition, template))
    }
}

/* === Public Functions === */

// The suite a readable template id names, when the id is well formed.
pub fn template_suite(identifier: &str) -> Option<&str> {
    let (suite, _) = identifier.rsplit_once('.')?;
    dokime_domain::model::definition::template_identifier(suite, identifier).ok().map(|()| suite)
}

pub(crate) fn registered_template<'a>(
    suite: &'a dokime_domain::model::catalog::wire::SuiteDefinition, identifier: &str,
) -> Result<&'a dokime_domain::model::catalog::wire::TemplateDefinition> {
    let mut matches =
        suite.templates.iter().filter(|candidate| candidate.metadata.as_ref().is_some_and(|metadata| metadata.identifier == identifier));
    let selected = matches
        .next()
        .ok_or_else(|| Error::Rpc(tonic::Status::not_found(format!("template `{identifier}` is not registered in its suite"))))?;
    if matches.next().is_some() {
        return Err(Error::Rpc(tonic::Status::failed_precondition(format!(
            "template `{identifier}` is declared more than once in its suite"
        ))));
    }
    dokime_domain::model::identity::TemplateId::from_uuid(&selected.template_id)
        .map_err(|_| invalid("template has an invalid identity"))?;
    Ok(selected)
}

/* === Private Functions === */

fn validate_listed_identity(
    identity: &dokime_domain::model::catalog::wire::SuiteIdentity, suite: &dokime_domain::model::catalog::wire::SuiteDefinition,
) -> Result<()> {
    if suite.identity.as_ref() != Some(identity) {
        return Err(invalid("suite definition differs from the listed identity"));
    }
    Ok(())
}

fn validate_definition(id: &str, suite: &dokime_domain::model::catalog::wire::SuiteDefinition) -> Result<()> {
    let identity = suite.identity.as_ref().ok_or_else(|| invalid("suite definition has no identity"))?;
    if identity.suite_id.trim().is_empty() {
        return Err(invalid("suite definition has an invalid identity"));
    }
    if identity.suite_id != id {
        return Err(invalid("suite definition identity differs"));
    }
    Ok(())
}

fn validate_page(epoch: &str, request: &catalog::ListSuites, page: &dokime_domain::model::catalog::wire::ListSuitesReply) -> Result<()> {
    let cursor =
        page.cursor.as_ref().filter(|cursor| !cursor.epoch.is_empty()).ok_or_else(|| invalid("suite page has no observed epoch"))?;
    if cursor.epoch != epoch {
        return Err(Error::IdentityChanged);
    }
    if (request.cursor.is_some() && request.cursor != page.cursor) || page.suites.len() > request.limit as usize {
        return Err(invalid("suite page changed revision or exceeded its limit"));
    }
    let mut previous = request.after_suite_id.as_str();
    for suite in &page.suites {
        if suite.suite_id.trim().is_empty() {
            return Err(invalid("suite page has an invalid identity"));
        }
        if suite.suite_id.as_str() <= previous {
            return Err(invalid("suite page identities do not advance"));
        }
        previous = &suite.suite_id;
    }
    if let Some(after) = &page.next_after_suite_id
        && (after.is_empty() || page.suites.last().map(|suite| &suite.suite_id) != Some(after))
    {
        return Err(invalid("suite page has an invalid continuation"));
    }
    Ok(())
}

fn invalid(message: &'static str) -> Error {
    Error::Rpc(tonic::Status::data_loss(message))
}

/* === Tests === */
