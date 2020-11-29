use crate::error::{Error, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

/// Protocol header.
///
/// Total: 16 bytes, but only 12 bytes are really useful.
#[derive(Debug)]
pub struct Header {
    pub len: usize,
    pub header_len: usize,
    pub ver: PacketVer,
    pub typ: PacketType,
}

#[derive(Debug, FromPrimitive, ToPrimitive, Copy, Clone, Eq, PartialEq)]
pub enum PacketType {
    Heartbeat = 2,
    HeartbeatResp = 3,
    Notification = 5,
    ClientAuth = 7,
    ServerAuth = 8,
}
#[derive(Debug, FromPrimitive, ToPrimitive, Copy, Clone, Eq, PartialEq)]
pub enum PacketVer {
    Plain = 0,
    Heartbeat = 1,
    Compressed = 2,
}

impl Header {
    pub fn parse(buf: &[u8]) -> Result<(Self, &[u8])> {
        if buf.len() < 16 {
            return Err(Error::InvalidHeader);
        }
        let (mut header, rest) = buf.split_at(16);
        let ret = Self {
            len: header.read_u32::<BigEndian>()? as usize,
            header_len: header.read_u16::<BigEndian>()? as usize,
            ver: PacketVer::from_u16(header.read_u16::<BigEndian>()?)
                .ok_or(Error::InvalidHeader)?,
            typ: PacketType::from_u32(header.read_u32::<BigEndian>()?)
                .ok_or(Error::InvalidHeader)?,
        };
        if ret.header_len != 16 {
            Err(Error::InvalidHeader)
        } else {
            Ok((ret, rest))
        }
    }

    pub fn new(data_len: usize, typ: PacketType, ver: PacketVer) -> Self {
        Self {
            len: data_len + 16,
            header_len: 16,
            ver,
            typ,
        }
    }

    fn write_into(&self, mut buf: &mut [u8]) -> Result<()> {
        buf.write_u32::<BigEndian>(self.len as u32)?;
        buf.write_u16::<BigEndian>(self.header_len as u16)?;
        buf.write_u16::<BigEndian>(self.ver.to_u16().unwrap())?;
        buf.write_u32::<BigEndian>(self.typ.to_u32().unwrap())?;
        buf.write_u32::<BigEndian>(1)?;

        Ok(())
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; 16];
        self.write_into(&mut buf)?;
        Ok(buf)
    }
}
