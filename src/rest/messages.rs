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

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Bytes(#[serde(with = "base64")] Vec<u8>);

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(val: Bytes) -> Self {
        val.0
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Bytes(v)
    }
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPubKeyResponse {
    pub identity: String,
    pub public_key: Bytes,
}
