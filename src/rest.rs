pub mod messages;

use crate::Error;
use crate::Result;
use std::sync::Arc;
use webpki::TrustAnchor;

const API: &str = "https://api.threema.ch";
const USER_AGENT: &str = "Threema/2.8";

include!(concat!(env!("OUT_DIR"), "/src/ca.rs"));

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Self {
        Self::ParseError(e.to_string())
    }
}

impl From<ureq::Error> for Error {
    fn from(_e: ureq::Error) -> Self {
        Self::RequestError
    }
}

fn tls_config() -> Arc<rustls::ClientConfig> {
    let mut cfg = rustls::ClientConfig::new();
    cfg.root_store
        .add_server_trust_anchors(&webpki::TLSServerTrustAnchors(&THREEMA_CA));
    Arc::new(cfg)
}

pub(crate) fn send<T, R>(path: &str, body: &T) -> Result<R>
where
    T: serde::Serialize,
    R: serde::de::DeserializeOwned,
{
    let tls_config = tls_config();

    let path = API.to_owned() + path;
    let resp = ureq::post(&path)
        .set("user-agent", USER_AGENT)
        .set("accept", "application/json")
        .set_tls_config(tls_config)
        .send_json(serde_json::to_value(body)?);
    if true {
        let resp = resp.into_string()?;
        Ok(serde_json::from_str(&resp)?)
    } else {
        Ok(resp.into_json_deserialize()?)
    }
}

pub(crate) fn request<R>(path: &str) -> Result<R>
where
    R: serde::de::DeserializeOwned,
{
    let tls_config = tls_config();

    let path = API.to_owned() + path;
    let resp = ureq::get(&path)
        .set("user-agent", USER_AGENT)
        .set("accept", "application/json")
        .set_tls_config(tls_config)
        .call();
    if true {
        let resp = resp.into_string()?;
        Ok(serde_json::from_str(&resp)?)
    } else {
        Ok(resp.into_json_deserialize()?)
    }
}
