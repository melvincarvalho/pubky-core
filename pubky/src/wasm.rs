use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use wasm_bindgen::prelude::*;

use reqwest::{Method, RequestBuilder, Response};
use url::Url;

use crate::PubkyClient;

mod http;
mod keys;
mod pkarr;
mod session;

use keys::{Keypair, PublicKey};
use session::Session;

#[wasm_bindgen]
impl PubkyClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder().build().unwrap(),
            session_cookies: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Signup to a homeserver and update Pkarr accordingly.
    ///
    /// The homeserver is a Pkarr domain name, where the TLD is a Pkarr public key
    /// for example "pubky.o4dksfbqk85ogzdb5osziw6befigbuxmuxkuxq8434q89uj56uyy"
    #[wasm_bindgen]
    pub async fn signup(&self, keypair: &Keypair, homeserver: &PublicKey) -> Result<(), JsValue> {
        self.inner_signup(keypair.as_inner(), homeserver.as_inner())
            .await
            .map_err(|e| e.into())
    }

    /// Check the current sesison for a given Pubky in its homeserver.
    ///
    /// Returns an [Error::NotSignedIn] if so, or [reqwest::Error] if
    /// the response has any other `>=400` status code.
    #[wasm_bindgen]
    pub async fn session(&self, pubky: &PublicKey) -> Result<Option<Session>, JsValue> {
        self.inner_session(pubky.as_inner())
            .await
            .map(|s| s.map(|s| Session(s).into()))
            .map_err(|e| e.into())
    }

    /// Signout from a homeserver.
    #[wasm_bindgen]
    pub async fn signout(&self, pubky: &PublicKey) -> Result<(), JsValue> {
        self.inner_signout(pubky.as_inner())
            .await
            .map_err(|e| e.into())
    }

    /// Signin to a homeserver.
    #[wasm_bindgen]
    pub async fn signin(&self, keypair: &Keypair) -> Result<(), JsValue> {
        self.inner_signin(keypair.as_inner())
            .await
            .map_err(|e| e.into())
    }

    // === Public data ===

    #[wasm_bindgen]
    /// Upload a small payload to a given path.
    pub async fn put(&self, pubky: &PublicKey, path: &str, content: &[u8]) -> Result<(), JsValue> {
        self.inner_put(pubky.as_inner(), path, content)
            .await
            .map_err(|e| e.into())
    }

    #[wasm_bindgen]
    /// Download a small payload from a given path relative to a pubky author.
    pub async fn get(&self, pubky: &PublicKey, path: &str) -> Result<js_sys::Uint8Array, JsValue> {
        self.inner_get(pubky.as_inner(), path)
            .await
            .map(|b| (*b).into())
            .map_err(|e| e.into())
    }
}
