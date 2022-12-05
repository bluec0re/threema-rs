pub mod messages;

use crate::Error;
use crate::Result;
use std::sync::Arc;
use webpki::TrustAnchor;

// from https://github.com/threema-ch/threema-android/blob/997fd7baacf314bb0238cca4912bd4d3d28b6886/app/src/main/java/ch/threema/client/ProtocolStrings.java
const API: &str = "https://apip.threema.ch";
const USER_AGENT: &str = "Threema";

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
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    root_store.add_server_trust_anchors(webpki::TlsServerTrustAnchors(&THREEMA_CA).0.iter().map(
        |ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        },
    ));
    Arc::new(
        rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth(),
    )
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().tls_config(tls_config()).build()
}

pub(crate) fn request<R>(path: &str) -> Result<R>
where
    R: serde::de::DeserializeOwned,
{
    let agent = agent();

    let path = API.to_owned() + path;
    let resp = agent
        .get(&path)
        .set("user-agent", USER_AGENT)
        .set("accept", "application/json")
        .call()?;
    Ok(resp.into_json()?)
}
