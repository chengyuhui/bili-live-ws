use miniz_oxide::inflate::decompress_to_vec_zlib;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, from_value, Value};

use crate::header::{Header, PacketType, PacketVer};

#[derive(Debug, Serialize)]
pub struct ClientAuth<'k> {
    roomid: u64,
    protover: u8,
    platform: &'static str,
    clientver: &'static str,
    r#type: u8,
    key: Option<&'k str>,
}

impl<'k> ClientAuth<'k> {
    pub fn new(room_id: u64, key: Option<&'k str>) -> Self {
        Self {
            roomid: room_id,
            protover: 2,
            platform: "web",
            clientver: "2.4.11",
            r#type: 2,
            key,
        }
    }
}

pub const HEARTBEAT_CONTENT: &[u8] =
    b"\x00\x00\x00\x1f\x00\x10\x00\x01\x00\x00\x00\x02\x00\x00\x00\x01[object Object]";

fn parse_single_packet(msg: &[u8]) -> (Header, Vec<u8>) {
    let (header, body) = Header::parse(&msg).unwrap();
    let body = &body[..header.len - 16];

    let body = if header.ver == PacketVer::Compressed {
        decompress_to_vec_zlib(&body).unwrap()
    } else {
        Vec::from(body)
    };

    (header, body)
}

pub fn parse_packet(msg: &[u8]) -> Packets {
    let (header, body) = parse_single_packet(msg);

    if header.typ != PacketType::Notification {
        return Packets::Single(None);
    }

    if header.ver == PacketVer::Compressed {
        Packets::Multiple { buf: body, pos: 0 }
    } else {
        Packets::Single(from_slice(&body).ok().and_then(Packet::from_value))
    }
}

#[derive(Debug)]
pub enum Packets {
    Single(Option<Packet>),
    Multiple { buf: Vec<u8>, pos: usize },
}

impl Iterator for Packets {
    type Item = Packet;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self {
                Self::Single(s) => return s.take(),
                Self::Multiple { buf, pos } => {
                    if *pos == buf.len() {
                        return None;
                    }
                    let (header, body) = parse_single_packet(&buf[*pos..]);
                    *pos += header.len;
                    match from_slice::<Value>(&body) {
                        Ok(r) => return Packet::from_value(r),
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct MedalInfo {
    pub name: String,
    pub level: u64,
    pub uname: String,
    pub room: u64,
}

#[derive(Debug)]
pub struct UserInfo {
    pub name: String,
    pub id: u64,
}

#[derive(Debug)]
pub struct DanmuPacket {
    pub text: String,
    pub user: UserInfo,
    pub medal: Option<MedalInfo>,
}

impl DanmuPacket {
    fn from_value(v: &Value) -> Option<Self> {
        let arr = v.as_array()?;
        arr[0].as_array()?; // Part ignored
        let text = arr[1].as_str()?;
        let user_info = arr[2].as_array()?;
        let medal_info = arr[3].as_array()?;

        Some(Self {
            text: text.to_string(),
            user: UserInfo {
                name: user_info[1].as_str()?.into(),
                id: user_info[0].as_u64()?,
            },
            medal: if medal_info.is_empty() {
                None
            } else {
                Some(MedalInfo {
                    level: medal_info[0].as_u64()?.into(),
                    name: medal_info[1].as_str()?.into(),
                    uname: medal_info[2].as_str()?.into(),
                    room: medal_info[3].as_u64()?.into(),
                })
            },
        })
    }
}

#[derive(Debug)]
pub struct InteractPacket {}

impl InteractPacket {
    fn from_value(v: &Value) -> Option<Self> {
        let _obj = v.as_object()?;
        Some(InteractPacket {})
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftPacket {
    pub action: String,
    pub gift_name: String,
    pub uname: String,
    pub num: u64,
    pub price: u64,
}

impl GiftPacket {
    fn from_value(v: &Value) -> Option<Self> {
        // dbg!(v);
        let res: Result<Self, _> = from_value(v.clone());

        res.ok()
    }
}

#[derive(Debug)]
pub enum Packet {
    Danmu(DanmuPacket),
    Interact(InteractPacket),
    Banner,
    Notice,
    Other(Value),
    Gift(GiftPacket),
}

impl Packet {
    fn from_value(v: Value) -> Option<Self> {
        let obj = v.as_object()?;
        let cmd = obj["cmd"].as_str()?;

        Some(match cmd {
            "DANMU_MSG" => Self::Danmu(DanmuPacket::from_value(&obj["info"])?),
            "INTERACT_WORD" => Self::Interact(InteractPacket::from_value(&obj["data"])?),
            "ROOM_BANNER" => Self::Banner,
            "NOTICE_MSG" => Self::Notice,
            "SEND_GIFT" => Self::Gift(GiftPacket::from_value(&obj["data"])?),
            _ => Self::Other(v),
        })
    }
}
