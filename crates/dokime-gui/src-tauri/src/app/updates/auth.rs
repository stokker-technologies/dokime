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
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

// Third Party Imports
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use keyring::{Entry, Error as KeyringError};
use rand::random;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::oneshot;
use url::Url;
use zeroize::Zeroizing;

// Local Imports
use super::sources::{self, Authorization};
use crate::app::organization::policy::{Connection, https_url};

/* === Constants === */

const DESKTOP_ACCOUNT_TIMEOUT_MS: u64 = 30_000;
const DESKTOP_UPDATE_ACCOUNT_BYTES: u64 = 65_536;

const REDIRECT: &str = "dokime-test-hub://auth/gitlab/callback";
const SERVICE: &str = "com.stokker.dokime.organization-oauth";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Accounts {
    path: PathBuf,
    load_error: Option<String>,
    inner: Mutex<Inner>,
}

struct Inner {
    users: BTreeMap<String, User>,
    pending: Option<Pending>,
    epoch: u64,
}

struct Pending {
    connection: Connection,
    state: String,
    verifier: Zeroizing<String>,
    epoch: u64,
    reply: oneshot::Sender<Result<User, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u64,
    pub username: String,
    pub name: String,
}

#[derive(Deserialize)]
struct Token {
    access_token: String,
}

pub(super) struct SignIn {
    pub url: String,
    pub epoch: u64,
    pub reply: oneshot::Receiver<Result<User, String>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Accounts {
    pub(super) fn open(path: PathBuf) -> Self {
        let read = match dokime::harmos::filesystem::input::read(&path, DESKTOP_UPDATE_ACCOUNT_BYTES as usize) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|_| "Saved release accounts are invalid; the file was retained"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(_) => Err("Saved release accounts could not be read; the file was retained"),
        };
        let (users, load_error) = match read {
            Ok(users) => (users, None),
            Err(error) => (BTreeMap::new(), Some(error.to_owned())),
        };
        Self { path, load_error, inner: Mutex::new(Inner { users, pending: None, epoch: 0 }) }
    }

    pub(super) fn user(&self, connection: &Connection) -> Result<Option<User>, String> {
        if let Some(error) = &self.load_error {
            return Err(error.clone());
        }
        let scope = scope(connection)?;
        let inner = self.inner.lock().map_err(|_| "Release account state is unavailable")?;
        Ok(inner.users.get(&scope).cloned())
    }

    pub(super) fn authorization(&self, connection: &Connection) -> Result<Option<Authorization>, String> {
        let Some(user) = self.user(connection)? else { return Ok(None) };
        let entry = entry(connection, user.id)?;
        let token = match entry.get_password() {
            Ok(token) => Zeroizing::new(token),
            Err(KeyringError::NoEntry) => return Ok(None),
            Err(_) => return Err("The release account token could not be read from secure storage".into()),
        };
        Ok(Some(Authorization { origin: https_url(&connection.host)?, token }))
    }

    pub(super) fn begin(&self, connection: Connection) -> Result<SignIn, String> {
        if let Some(error) = &self.load_error {
            return Err(error.clone());
        }
        if connection.oauth_client_id.is_empty() {
            return Err("This connection has no OAuth client identifier".into());
        }
        let mut url = https_url(&connection.host)?.join("oauth/authorize").map_err(|_| "The OAuth origin is invalid")?;
        let state = random_token();
        let verifier = Zeroizing::new(random_token());
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        url.query_pairs_mut()
            .append_pair("client_id", &connection.oauth_client_id)
            .append_pair("redirect_uri", REDIRECT)
            .append_pair("response_type", "code")
            .append_pair("scope", "read_user read_api")
            .append_pair("state", &state)
            .append_pair("code_challenge", &challenge)
            .append_pair("code_challenge_method", "S256");

        let (reply, receiver) = oneshot::channel();
        let mut inner = self.inner.lock().map_err(|_| "Release account state is unavailable")?;
        cancel(&mut inner);
        inner.pending = Some(Pending { connection, state: state.clone(), verifier, epoch: inner.epoch, reply });
        Ok(SignIn { url: url.to_string(), epoch: inner.epoch, reply: receiver })
    }

    pub(super) fn cancel(&self, epoch: Option<u64>) {
        let mut inner = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if epoch.is_none_or(|epoch| epoch == inner.epoch) {
            cancel(&mut inner);
        }
    }

    pub(super) async fn callback(self: &Arc<Self>, url: Url) {
        let callback = Url::parse(REDIRECT).expect("compiled OAuth callback");
        if url.scheme() != callback.scheme() || url.host_str() != callback.host_str() || url.path() != callback.path() {
            return;
        }
        let values: Vec<_> = url.query_pairs().collect();
        let states: Vec<_> = values.iter().filter(|(key, _)| key == "state").collect();
        if states.len() != 1 {
            return;
        }
        let pending = {
            let mut inner = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if inner.pending.as_ref().is_none_or(|pending| pending.state != states[0].1) {
                return;
            }
            inner.pending.take().expect("matching pending sign-in")
        };
        let codes: Vec<_> = values.iter().filter(|(key, _)| key == "code").collect();
        let rejected = values.iter().any(|(key, _)| key == "error");
        if rejected || codes.len() != 1 || codes[0].1.is_empty() {
            let _ = pending.reply.send(Err("The identity provider refused the sign-in or returned an invalid callback".into()));
            return;
        }
        let result = exchange(&pending.connection, &codes[0].1, &pending.verifier).await;
        let result = match result {
            Ok((user, token)) => {
                let accounts = Arc::clone(self);
                let connection = pending.connection.clone();
                let epoch = pending.epoch;
                tokio::task::spawn_blocking(move || accounts.publish(&connection, epoch, user, token))
                    .await
                    .map_err(|_| "Release token storage task failed".to_owned())
                    .and_then(|result| result)
            }
            Err(error) => Err(error),
        };
        let _ = pending.reply.send(result);
    }

    pub(super) fn sign_out(&self, connection: &Connection) -> Result<(), String> {
        let scope = scope(connection)?;
        let mut inner = self.inner.lock().map_err(|_| "Release account state is unavailable")?;
        cancel(&mut inner);
        let Some(user) = inner.users.get(&scope).cloned() else { return Ok(()) };
        let mut next = inner.users.clone();
        next.remove(&scope);
        self.save(&next)?;
        inner.users = next;
        match entry(connection, user.id)?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(_) => Err("Signed out; the old secure token could not be removed".into()),
        }
    }

    fn publish(&self, connection: &Connection, epoch: u64, user: User, token: Zeroizing<String>) -> Result<User, String> {
        let scope = scope(connection)?;
        let mut inner = self.inner.lock().map_err(|_| "Release account state is unavailable")?;
        if inner.epoch != epoch {
            return Err("The release connection changed before sign-in completed".into());
        }
        let entry = entry(connection, user.id)?;
        entry.set_password(&token).map_err(|_| "The release token could not be saved in secure storage")?;
        let mut next = inner.users.clone();
        next.insert(scope, user.clone());
        self.save(&next)?;
        inner.users = next;
        Ok(user)
    }

    fn save(&self, users: &BTreeMap<String, User>) -> Result<(), String> {
        if let Some(error) = &self.load_error {
            return Err(error.clone());
        }
        let bytes = serde_json::to_vec(users).map_err(|_| "Release account metadata could not be encoded")?;
        if bytes.len() as u64 > DESKTOP_UPDATE_ACCOUNT_BYTES {
            return Err("Release account metadata exceeds its size limit".into());
        }
        dokime::harmos::filesystem::publication::write(&self.path, &bytes).map_err(|_| "Release account metadata could not be saved".into())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn cancel(inner: &mut Inner) {
    inner.epoch = inner.epoch.wrapping_add(1);
    if let Some(pending) = inner.pending.take() {
        let _ = pending.reply.send(Err("Release sign-in was cancelled".into()));
    }
}

fn scope(connection: &Connection) -> Result<String, String> {
    let origin = https_url(&connection.host)?.origin().ascii_serialization();
    let digest = Sha256::digest(format!("{origin}\0{}", connection.oauth_client_id).as_bytes());
    Ok(URL_SAFE_NO_PAD.encode(digest))
}

fn entry(connection: &Connection, account: u64) -> Result<Entry, String> {
    let user = format!("{}:{account}", scope(connection)?);
    Entry::new(SERVICE, &user).map_err(|_| "Release secure storage is unavailable".into())
}

fn random_token() -> String {
    URL_SAFE_NO_PAD.encode(random::<[u8; 32]>())
}

async fn exchange(connection: &Connection, code: &str, verifier: &str) -> Result<(User, Zeroizing<String>), String> {
    let origin = https_url(&connection.host)?;
    let token_url = origin.join("oauth/token").map_err(|_| "The OAuth origin is invalid")?;
    let client = crate::app::http::client()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_millis(DESKTOP_ACCOUNT_TIMEOUT_MS))
        .build()
        .map_err(|_| "OAuth HTTP initialization failed")?;
    let form = [
        ("client_id", connection.oauth_client_id.as_str()),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", REDIRECT),
        ("code_verifier", verifier),
    ];
    let body = url::form_urlencoded::Serializer::new(String::new()).extend_pairs(form).finish();
    let response = client
        .post(token_url)
        .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|_| "OAuth token exchange failed")?;
    let mut response = response.error_for_status().map_err(|_| "The identity provider refused the token exchange")?;
    let mut bytes = Zeroizing::new(Vec::new());
    while let Some(chunk) = response.chunk().await.map_err(|_| "OAuth token response failed")? {
        if chunk.len() > (DESKTOP_UPDATE_ACCOUNT_BYTES as usize).saturating_sub(bytes.len()) {
            return Err("OAuth token response exceeds its size limit".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let decoded: Token = serde_json::from_slice(&bytes).map_err(|_| "OAuth token response is invalid")?;
    let token = Zeroizing::new(decoded.access_token);
    let authorization = Authorization { origin: origin.clone(), token: token.clone() };
    let user_url = origin.join("api/v4/user").map_err(|_| "The OAuth origin is invalid")?;
    let bytes = sources::fetch(&user_url, Some(&authorization), DESKTOP_UPDATE_ACCOUNT_BYTES as usize).await?;
    let user: User = serde_json::from_slice(&bytes).map_err(|_| "The identity provider returned an invalid account")?;
    Ok((user, token))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unmatched_callbacks_preserve_pending_auth_and_namespaces_are_disjoint() {
        let root = tempfile::tempdir().unwrap();
        let accounts = Arc::new(Accounts::open(root.path().join("accounts.json")));
        let connection = Connection { host: "https://gitlab.example.test".into(), oauth_client_id: "client-one".into() };
        let pending = accounts.begin(connection.clone()).unwrap();
        accounts.callback(Url::parse(&format!("{REDIRECT}?state=wrong&code=unused")).unwrap()).await;
        assert!(accounts.inner.lock().unwrap().pending.is_some());
        let other = Connection { oauth_client_id: "client-two".into(), ..connection.clone() };
        assert_ne!(scope(&connection).unwrap(), scope(&other).unwrap());
        let host = Connection { host: "https://other.example.test".into(), ..connection };
        assert_ne!(scope(&host).unwrap(), scope(&other).unwrap());
        accounts.cancel(None);
        assert!(pending.reply.await.unwrap().is_err());
        assert!(!root.path().join("accounts.json").exists());
    }
}
