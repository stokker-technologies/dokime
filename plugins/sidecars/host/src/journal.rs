/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Effect Journal: Control Inverses Kept in the Plugin's Durable State Value

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;

// Third Party Imports

// Local Imports
use crate::domain::controls::ControlState;
use dokime::encoding::Message as Encoded;
use dokime::operations::Context;
use dokime::{Error, Message, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Revision of the Plugin's Durable State Value.
pub struct Saved {
    pub value: Vec<u8>,
    pub revision: Vec<u8>,
}

/// The Outstanding Effects, Read From and Saved To One Durable State Value.
pub struct Journal<'a, S> {
    store: &'a S,
}

/// One Outstanding Effect: the Target a Control Mutated and the State That Restores It.
#[derive(Message)]
#[dokime(package = "host")]
struct Effect {
    target: String,
    inverse: Option<ControlState>,
}

/// The Durable Journal Value, Outstanding Effects in Recording Order.
#[derive(Message)]
#[dokime(package = "host")]
struct Effects {
    effects: Vec<Effect>,
}

/* === Enums === */

/* === Traits === */

/// The Plugin's Durable State Value, Persisted by the Host; `None` When the Host Serves No Durable State.
pub trait Store: Sync {
    fn load(&self) -> impl Future<Output = Result<Option<Saved>>> + Send;
    /// Replace the Value Only While Its Revision Is Still `expected` (Empty for a Value Never Saved).
    fn save(&self, value: Vec<u8>, expected: Vec<u8>) -> impl Future<Output = Result<()>> + Send;
}

/* === Implementations === */

impl<'a, S: Store> Journal<'a, S> {
    pub fn new(store: &'a S) -> Self {
        Self { store }
    }

    async fn read(&self) -> Result<Option<(Effects, Vec<u8>)>> {
        let Some(saved) = self.store.load().await? else {
            return Ok(None);
        };
        let effects = Effects::decode(saved.value.as_slice())
            .map_err(|error| Error::Invalid(format!("Read Effect Journal: Restore Its Saved Value: {error}")))?;
        Ok(Some((effects, saved.revision)))
    }
}

impl<S: Store> crate::controls::Journal for Journal<'_, S> {
    async fn record(&self, target: &str, inverse: &ControlState) -> Result<bool> {
        let Some((mut effects, revision)) = self.read().await? else {
            return Err(Error::Invalid("Set Control: Run a Host That Keeps Durable Plugin State; No Control Was Changed".into()));
        };
        if effects.effects.iter().any(|effect| effect.target == target) {
            return Ok(false);
        }
        effects.effects.push(Effect {
            target: target.into(),
            inverse: Some(inverse.clone()),
        });
        self.store.save(effects.encode_to_vec(), revision).await?;
        Ok(true)
    }

    async fn clear(&self, target: &str) -> Result<()> {
        let Some((mut effects, revision)) = self.read().await? else {
            return Ok(());
        };
        let count = effects.effects.len();
        effects.effects.retain(|effect| effect.target != target);
        if effects.effects.len() == count {
            return Ok(());
        }
        self.store.save(effects.encode_to_vec(), revision).await
    }

    async fn outstanding(&self) -> Result<Vec<(String, ControlState)>> {
        let Some((effects, _)) = self.read().await? else {
            return Ok(Vec::new());
        };
        effects
            .effects
            .into_iter()
            .map(|effect| {
                let inverse = effect.inverse.ok_or_else(|| {
                    Error::Invalid(format!("Read Effect Journal: Restore {} by Hand; Its Inverse Is Missing", effect.target))
                })?;
                Ok((effect.target, inverse))
            })
            .collect()
    }
}

/// The Library Serves No Durable Plugin State Yet, So Records Are Refused and Nothing Is Outstanding.
impl Store for Context {
    async fn load(&self) -> Result<Option<Saved>> {
        Ok(None)
    }
    async fn save(&self, _: Vec<u8>, _: Vec<u8>) -> Result<()> {
        Err(Error::Invalid("Save Effect Journal: Run a Host That Keeps Durable Plugin State".into()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
