use crate::MessageID;
use crate::ThreemaID;
use flat_bytes::flat_enum;
use flat_bytes::Flat;
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

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "m")]
    pub mime: String,
    #[serde(rename = "p")]
    pub preview_mime: String,
    #[serde(rename = "s")]
    pub size: u64,
    #[serde(rename = "d")]
    pub description: String,
    #[serde(flatten)]
    pub unknown: std::collections::HashMap<String, serde_json::Value>,
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
    pub unknown: std::collections::HashMap<String, serde_json::Value>,
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
    pub unknown: std::collections::HashMap<String, serde_json::Value>,
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
