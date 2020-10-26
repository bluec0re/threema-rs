use crate::MessageID;
use crate::ThreemaID;
use flat_bytes::flat_enum;
use flat_bytes::Flat;
use serde::de::Deserializer;
use serde::de::Error;
use serde::de::Unexpected;
use serde::de::Visitor;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec};

flat_enum! {
    #[derive(Debug)]
    #[repr(u32)]
    pub enum Packet {
        ClientToServer(Header) = 1,
        ServerToClient(Header) = 2,
        Echo(u64) = 0x80,
        ServerAck(ThreemaID, MessageID),
        ClientAck(ThreemaID, MessageID),
        ConnectionEstablished = 0xd0,
        DublicateConnection = 0xe0,
    }
}

type PollID = [u8; 8];
flat_enum! {
    #[derive(Debug)]
    #[repr(u8)]
    pub enum Message {
        Text(Text) = 1,
        Image,
        Location = 16,
        Video = 19,
        Audio = 20,
        Poll {
            poll_id: PollID,
            details: PollDetails,
        },
        PollUpdate {
            sender: ThreemaID,
            poll_id: PollID,
            updats: PollUpdate,
        } = 22,
        File(File) = 23,
        GroupText = 65,
        GroupImage = 67,
        GroupSetMembers = 74,
        GroupSetName,
        GroupMemberLeft,
        DeliveryReceipt(MessageStatus, MessageID) = 0x80,
        TypingNotification = 0x90,
    }
}

flat_enum! {
    #[derive(Debug)]
    #[repr(u8)]
    pub enum MessageStatus {
        Delivered = 1,
        Read,
        Approved,
        Disapproved,
    }
}

#[derive(Debug, Flat)]
pub struct Header {
    pub sender: ThreemaID,
    pub receiver: ThreemaID,
    pub msg_id: MessageID,
    pub timestamp: u32,
    pub flags: u32,
    pub nickname: [u8; 32],
    pub nonce: [u8; 24],
}

#[derive(Debug)]
pub struct Text {
    pub message: String,
}

impl Flat for Text {
    fn serialize(&self) -> Vec<u8> {
        self.message.as_bytes().to_owned()
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        let message = String::from_utf8(data.to_owned()).ok()?;
        Some((Self { message }, data.len()))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderingType {
    /// Display as default file message
    File = 0,
    /// Display as media file message (e.g. image or audio message)
    Media = 1,
    /// Display as sticker (images with transparency, rendered without bubble)
    Sticker = 2,
}

impl Serialize for RenderingType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(*self as u8)
    }
}

struct EnumVisitor;

impl<'de> Visitor<'de> for EnumVisitor {
    type Value = RenderingType;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(formatter, "u8")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            0 => Ok(RenderingType::File),
            1 => Ok(RenderingType::Media),
            2 => Ok(RenderingType::Sticker),
            x => Err(Error::invalid_value(Unexpected::Unsigned(x), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for RenderingType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u8(EnumVisitor)
    }
}

impl Default for RenderingType {
    fn default() -> Self {
        RenderingType::File
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    #[serde(rename = "b")]
    blob_id: String,
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "m")]
    pub mime: String,
    #[serde(rename = "t")]
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail_blob_id: Option<String>,
    #[serde(rename = "p")]
    pub thumbnail_mime: String,
    #[serde(rename = "s")]
    pub size: u64,
    #[serde(rename = "d")]
    pub description: String,
    #[serde(rename = "j")]
    rendering_type: RenderingType,
    #[serde(rename = "k")]
    encryption_key: String,
    #[serde(flatten)]
    unknown: std::collections::HashMap<String, serde_json::Value>,
}

impl Flat for File {
    fn serialize(&self) -> Vec<u8> {
        to_vec(self).unwrap()
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        let res = from_slice(data).ok()?;
        Some((res, data.len()))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PollChoice {
    #[serde(rename = "i")]
    pub id: u32,
    #[serde(rename = "n")]
    pub text: String,
    #[serde(rename = "o")]
    pub order: u32,
    #[serde(rename = "r")]
    pub results: Vec<u32>,
    #[serde(flatten)]
    unknown: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PollDetails {
    #[serde(rename = "d")]
    pub description: String,
    #[serde(rename = "c")]
    pub choices: Vec<PollChoice>,
    #[serde(rename = "p")]
    pub participants: Vec<String>,
    #[serde(flatten)]
    unknown: std::collections::HashMap<String, serde_json::Value>,
}

impl Flat for PollDetails {
    fn serialize(&self) -> Vec<u8> {
        to_vec(self).unwrap()
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        let res = from_slice(data).ok()?;
        Some((res, data.len()))
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PollUpdate {
    updates: Vec<(u32, u32)>,
}

impl Flat for PollUpdate {
    fn serialize(&self) -> Vec<u8> {
        to_vec(self).unwrap()
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        let res = from_slice(data).ok()?;
        Some((res, data.len()))
    }
}
