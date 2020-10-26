use crate::Result;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

mod base64 {
    use serde::de::Error;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;

    pub fn serialize<S>(data: &[u8], s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let b = ::base64::encode(data);
        s.serialize_str(&b)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(d)
            .and_then(|s| ::base64::decode(&s).map_err(|err| Error::custom(err.to_string())))
    }
}

pub trait Method<R>: Serialize
where
    R: DeserializeOwned,
{
    fn path(&self) -> &'static str;
    fn call(self) -> Result<R>
    where
        Self: std::marker::Sized,
    {
        crate::rest::send(self.path(), &self)
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Bytes(#[serde(with = "base64")] Vec<u8>);

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Into<Vec<u8>> for Bytes {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Bytes(v)
    }
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest<'a> {
    #[serde(with = "base64")]
    pub public_key: &'a [u8],
    pub device_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Bytes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<Bytes>,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResponse {
    pub token: Option<Bytes>,
    pub token_resp_key_pub: Option<Bytes>,
    pub identity: Option<String>,
    pub success: Option<bool>,
    pub error: Option<String>,
}

impl<'a> Method<CreateResponse> for CreateRequest<'a> {
    fn path(&self) -> &'static str {
        "/identity/create"
    }
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPubKeyResponse {
    pub identity: String,
    pub public_key: Bytes,
}
