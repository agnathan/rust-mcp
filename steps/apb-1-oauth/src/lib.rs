//! An in-memory authorization server, a proxy that does not call upstream,
//! and a strict read of an OIDC discovery document.
//!
//! No socket is opened. Codes expire after [`AUTH_CODE_EXPIRY_SECONDS`].

use std::collections::HashMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};

/// PKCE method this appendix accepts.
pub const PKCE_METHOD: &str = "S256";

/// Authorization codes live this many seconds. Matches FastMCP's in-memory provider.
pub const AUTH_CODE_EXPIRY_SECONDS: u64 = 5 * 60;

/// `BASE64URL(SHA256(verifier))` with no padding.
pub fn code_challenge_s256(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

#[derive(Debug)]
struct Client {
    redirect_uri: String,
}

#[derive(Debug)]
struct Code {
    client_id: String,
    challenge: String,
    expires_at: u64,
}

/// The full server, held in maps. `now` is a second count the caller passes in.
#[derive(Debug, Default)]
pub struct MemoryOAuth {
    issuer: String,
    clients: HashMap<String, Client>,
    codes: HashMap<String, Code>,
    next_code: u64,
    next_token: u64,
}

impl MemoryOAuth {
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            ..Self::default()
        }
    }

    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    pub fn register(&mut self, client_id: impl Into<String>, redirect_uri: impl Into<String>) {
        self.clients.insert(
            client_id.into(),
            Client {
                redirect_uri: redirect_uri.into(),
            },
        );
    }

    /// Issue a code bound to the S256 challenge.
    ///
    /// # Errors
    ///
    /// The client is missing, or the redirect URI does not match the one registered.
    pub fn authorize(
        &mut self,
        client_id: &str,
        redirect_uri: &str,
        challenge: &str,
        now: u64,
    ) -> Result<String, String> {
        let client = self
            .clients
            .get(client_id)
            .ok_or_else(|| format!("unknown client: {client_id}"))?;
        if client.redirect_uri != redirect_uri {
            return Err("redirect mismatch".to_string());
        }
        self.next_code += 1;
        let code = format!("code-{}", self.next_code);
        self.codes.insert(
            code.clone(),
            Code {
                client_id: client_id.to_string(),
                challenge: challenge.to_string(),
                expires_at: now.saturating_add(AUTH_CODE_EXPIRY_SECONDS),
            },
        );
        Ok(code)
    }

    /// Turn a code and a verifier into a bearer token. The code is single-use.
    ///
    /// # Errors
    ///
    /// The code is missing, expired, or the verifier does not match the challenge.
    pub fn exchange(&mut self, code: &str, verifier: &str, now: u64) -> Result<String, String> {
        let stored = self
            .codes
            .remove(code)
            .ok_or_else(|| "unknown code".to_string())?;
        if stored.client_id.is_empty() {
            return Err("unknown client".to_string());
        }
        if now >= stored.expires_at {
            return Err("code expired".to_string());
        }
        if code_challenge_s256(verifier) != stored.challenge {
            return Err("pkce mismatch".to_string());
        }
        self.next_token += 1;
        Ok(format!("tok-{}", self.next_token))
    }

    /// Authorization-server metadata. Endpoints sit under the issuer. Nothing is served.
    pub fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "issuer": self.issuer,
            "authorization_endpoint": format!("{}/authorize", self.issuer.trim_end_matches('/')),
            "token_endpoint": format!("{}/token", self.issuer.trim_end_matches('/')),
            "code_challenge_methods_supported": [PKCE_METHOD],
        })
    }
}

#[derive(Debug)]
struct Transaction {
    client_redirect: String,
    client_state: String,
    challenge: String,
}

#[derive(Debug)]
struct ClientCode {
    challenge: String,
    upstream_token: String,
}

/// A proxy that remembers a transaction and does not call the upstream server.
#[derive(Debug)]
pub struct OAuthProxy {
    upstream_authorize: String,
    fixed_redirect: String,
    next: u64,
    transactions: HashMap<String, Transaction>,
    codes: HashMap<String, ClientCode>,
}

impl OAuthProxy {
    pub fn new(upstream_authorize: impl Into<String>, fixed_redirect: impl Into<String>) -> Self {
        Self {
            upstream_authorize: upstream_authorize.into(),
            fixed_redirect: fixed_redirect.into(),
            next: 0,
            transactions: HashMap::new(),
            codes: HashMap::new(),
        }
    }

    /// The URL the browser would open. `state` on that URL is the transaction id.
    pub fn start(&mut self, client_redirect: &str, client_state: &str, challenge: &str) -> String {
        self.next += 1;
        let id = format!("tx-{}", self.next);
        self.transactions.insert(
            id.clone(),
            Transaction {
                client_redirect: client_redirect.to_string(),
                client_state: client_state.to_string(),
                challenge: challenge.to_string(),
            },
        );
        format!(
            "{}?redirect_uri={}&state={id}",
            self.upstream_authorize, self.fixed_redirect
        )
    }

    /// The upstream code arrived. Hand the client a new code on their own redirect.
    ///
    /// # Errors
    ///
    /// The transaction id is missing.
    pub fn complete(
        &mut self,
        transaction_id: &str,
        upstream_code: &str,
    ) -> Result<String, String> {
        let tx = self
            .transactions
            .remove(transaction_id)
            .ok_or_else(|| format!("unknown transaction: {transaction_id}"))?;
        self.next += 1;
        let code = format!("client-code-{}", self.next);
        self.codes.insert(
            code.clone(),
            ClientCode {
                challenge: tx.challenge,
                upstream_token: format!("upstream:{upstream_code}"),
            },
        );
        Ok(format!(
            "{}?code={code}&state={}",
            tx.client_redirect, tx.client_state
        ))
    }

    /// Check the client's verifier and return the stored upstream token.
    ///
    /// # Errors
    ///
    /// The code is missing or the verifier does not match.
    pub fn exchange(&mut self, code: &str, verifier: &str) -> Result<String, String> {
        let stored = self
            .codes
            .remove(code)
            .ok_or_else(|| "unknown code".to_string())?;
        if code_challenge_s256(verifier) != stored.challenge {
            return Err("pkce mismatch".to_string());
        }
        Ok(stored.upstream_token)
    }
}

/// The fields a strict OIDC document must have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OidcConfig {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
}

/// Read a discovery document.
///
/// When `strict` is true, a missing required field returns
/// `Missing required configuration metadata: {name}`.
///
/// # Errors
///
/// The text is not a JSON object, or strict mode is missing a required field.
pub fn oidc_from_json(text: &str, strict: bool) -> Result<OidcConfig, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|err| format!("oidc document: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "oidc document is not an object".to_string())?;
    if strict {
        for key in [
            "issuer",
            "authorization_endpoint",
            "token_endpoint",
            "jwks_uri",
            "response_types_supported",
            "subject_types_supported",
            "id_token_signing_alg_values_supported",
        ] {
            let present = match object.get(key) {
                Some(serde_json::Value::String(text)) => !text.is_empty(),
                Some(serde_json::Value::Array(items)) => !items.is_empty(),
                Some(_) => true,
                None => false,
            };
            if !present {
                return Err(format!("Missing required configuration metadata: {key}"));
            }
        }
    }
    Ok(OidcConfig {
        issuer: string_field(object, "issuer")?,
        authorization_endpoint: string_field(object, "authorization_endpoint")?,
        token_endpoint: string_field(object, "token_endpoint")?,
        jwks_uri: string_field(object, "jwks_uri")?,
    })
}

fn string_field(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<String, String> {
    object
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| format!("Missing required configuration metadata: {key}"))
}
