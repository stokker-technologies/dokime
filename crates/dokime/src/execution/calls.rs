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
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// Third Party Imports
use harmos::Message;
use harmos::operations::{Declares, OperationDeclaration};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type Answering = Arc<
    dyn Fn(Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + Send>> + Send + Sync,
>;

/* === Structs === */
#[derive(Clone, Default)]
pub struct Caller(Road);

/* === Enums === */
#[derive(Clone, Default)]
enum Road {
    #[default]
    Unmounted,
    Host(harmos::sidecar::Supervisor),
    Local(Arc<BTreeMap<String, Answering>>),
}

/* === Traits === */

/* === Implementations === */
impl Caller {
    pub fn host(host: harmos::sidecar::Supervisor) -> Self {
        Self(Road::Host(host))
    }

    pub fn local(operations: BTreeMap<String, Answering>) -> Self {
        Self(Road::Local(Arc::new(operations)))
    }

    pub async fn call<O>(
        &self,
        request: <O::Declaration as OperationDeclaration>::Request,
    ) -> crate::Result<<O::Declaration as OperationDeclaration>::Response>
    where
        O: Declares,
        <O::Declaration as OperationDeclaration>::Request: Message,
        <O::Declaration as OperationDeclaration>::Response: Message,
    {
        let operation = <O::Declaration as OperationDeclaration>::ID;
        let refused = |reason: String| crate::Error::Call { operation, reason };
        match &self.0 {
            Road::Unmounted => Err(refused(
                "this template is running without a host to route through".into(),
            )),
            Road::Host(host) => host
                .operation::<O::Declaration>(request)
                .await
                .map_err(|error| refused(error.to_string())),
            Road::Local(operations) => {
                let answer = operations.get(operation).ok_or_else(|| {
                    refused("the harness registered no answer for this operation".into())
                })?;
                let bytes = answer(request.encode_to_vec()).await.map_err(refused)?;
                <<O::Declaration as OperationDeclaration>::Response as Message>::decode(
                    bytes.as_slice(),
                )
                .map_err(|error| refused(error.to_string()))
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
