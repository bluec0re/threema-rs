pub mod messages;

use crate::Error;
use crate::Result;
use std::result;
use std::sync::Arc;

const API: &str = "https://api.threema.ch";
const USER_AGENT: &str = "Threema/2.8";

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

struct ServerCertVerifier {}

impl rustls::ServerCertVerifier for ServerCertVerifier {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> result::Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}

pub fn send<T, R>(path: &str, body: &T) -> Result<R>
where
    T: serde::Serialize,
    R: serde::de::DeserializeOwned,
{
    let verifier = Arc::new(ServerCertVerifier {});
    let mut tls_config = rustls::ClientConfig::new();
    tls_config.dangerous().set_certificate_verifier(verifier);
    let tls_config = Arc::new(tls_config);

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

pub fn request<R>(path: &str) -> Result<R>
where
    R: serde::de::DeserializeOwned,
{
    let verifier = Arc::new(ServerCertVerifier {});
    let mut tls_config = rustls::ClientConfig::new();
    tls_config.dangerous().set_certificate_verifier(verifier);
    let tls_config = Arc::new(tls_config);

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
