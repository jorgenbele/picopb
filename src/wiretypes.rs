use crate::encode::ToWire;
use crate::common::Packed;

/// This file contains the predefine wiretypes for the types where this is applicable
/// TODO: ZigZag encoding of signed types
pub struct Field(pub u32);
pub struct WireTypeId(pub u32);
pub struct Tag(pub u32);

pub trait ToVarint {
    fn to_varint_encoding(&self) -> ([u8; 10], usize);
}


impl ToVarint for u32 {
    fn to_varint_encoding(&self) -> ([u8; 10], usize) {
        let mut out: [u8; 10] = [0; 10];
        let mut writable = &mut out[..];
        let count = leb128::write::unsigned(&mut writable, *self as u64).unwrap();
        (out, count)
    }
}

impl ToVarint for u64 {
    fn to_varint_encoding(&self) -> ([u8; 10], usize) {
        let mut out: [u8; 10] = [0; 10];
        let mut writable = &mut out[..];
        let count = leb128::write::unsigned(&mut writable, *self).unwrap();
        (out, count)
    }
}

impl ToVarint for i32 {
    fn to_varint_encoding(&self) -> ([u8; 10], usize) {
        let mut out: [u8; 10] = [0; 10];
        let mut writable = &mut out[..];
        let count = leb128::write::signed(&mut writable, *self as i64).unwrap();
        dbg!(count);
        dbg!(out);
        (out, count)
    }
}

impl ToVarint for i64 {
    fn to_varint_encoding(&self) -> ([u8; 10], usize) {
        let mut out: [u8; 10] = [0; 10];
        let mut writable = &mut out[..];
        let count = leb128::write::signed(&mut writable, *self).unwrap();
        (out, count)
    }
}

impl ToVarint for Tag {
    fn to_varint_encoding(&self) -> ([u8; 10], usize) {
        self.0.to_varint_encoding()
    }
}

impl Tag {
    pub fn encode(&self) -> ([u8; 10], usize) {
        // encode as little-endian u32
        self.0.to_varint_encoding()
    }
}

#[derive(Debug)]
pub enum WireType {
    VarInt,
    I64,
    SGroup,
    EGroup,
    I32,
    Len
}

#[derive(Debug)]
pub enum WireTypeError {
    InvalidTag(u32),
}

pub const VARINT_ID: u32 = 0;
pub const I64_ID: u32 = 1;
pub const LEN_ID: u32 = 2;
pub const SGROUP_ID: u32 = 3;
pub const EGROUP_ID: u32 = 4;
pub const I32_ID: u32 = 5;

impl TryFrom<u32> for WireType {
    type Error = WireTypeError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            VARINT_ID => Ok(WireType::VarInt),
            I64_ID => Ok(WireType::I64),
            LEN_ID => Ok(WireType::Len),
            SGROUP_ID => Ok(WireType::SGroup),
            EGROUP_ID => Ok(WireType::EGroup),
            I32_ID => Ok(WireType::I32),
            _ => Err(WireTypeError::InvalidTag(value))
        }
    }
}

impl WireType {
    fn to_id(&self) -> WireTypeId {
        match self {
            WireType::VarInt => WireTypeId(VARINT_ID),
            WireType::I64 => WireTypeId(I64_ID),   
            WireType::Len => WireTypeId(LEN_ID),
            WireType::SGroup => WireTypeId(SGROUP_ID),
            WireType::EGroup => WireTypeId(EGROUP_ID),
            WireType::I32 => WireTypeId(I32_ID),
        }
    }
}

pub trait WireTyped {
    fn wiretype(&self) -> WireType;
    // value is the same as WireType except for SGroup and EGroup

    fn tag(&self, field: Field) -> Tag {
        Tag((field.0 << 3) | self.wiretype().to_id().0)
    }

}

/// When using packed mode the length is prefixed to repeated messages
impl<T> WireTyped for Packed<T> {
    fn wiretype(&self) -> WireType {
        WireType::Len
    }
}

impl WireTyped for &String {
    fn wiretype(&self) -> WireType {
        WireType::Len
    }
}

impl WireTyped for &[u8] {
    fn wiretype(&self) -> WireType {
        WireType::VarInt
    }
}

impl WireTyped for i32 {
    fn wiretype(&self) -> WireType {
        WireType::VarInt
    }
}