use std::{collections::{BTreeMap, HashMap}, hash::Hash, string::ParseError, option};

#[derive(Debug)]
pub struct WireType {
    pub id: usize,
    pub name: &'static str,
}

static VARINT_WIRETYPE: WireType = WireType { id: 0, name: "VARINT" };
static I64_WIRETYPE: WireType = WireType { id: 1, name: "I64" };
static LEN_WIRETYPE: WireType = WireType { id: 2, name: "LEN" };
// SGROUP and EGROUP are deprecated
static I32_WIRETYPE: WireType = WireType { id: 5, name: "I32" };

#[derive(Debug, Clone)]
pub enum FieldType {
    UnboundedString,
    UnboundedBytes,
    String(usize),
    Bytes(usize),
    Bool,
    Int32,
    Int64,
    Uint32,
    Uint64,
    EnumType(String),
    MessageType(String, usize),
    UnboundedMessageType(String),
}

impl FieldType {
    pub fn wiretype(self) -> &'static WireType {
        match self {
            Self::Bool | Self::Int32 | Self::Int64 | Self::Uint32 | Self::Uint64 | Self::EnumType(_) => &VARINT_WIRETYPE,
            Self::Bytes(_) | Self::String(_) => &LEN_WIRETYPE,
            Self::UnboundedString | Self::UnboundedBytes => &LEN_WIRETYPE,
            Self::UnboundedMessageType(_) | Self::MessageType(_, _) => &LEN_WIRETYPE,
        }
    }

    pub fn from_str(s: &str, max_size: Option<usize>) -> Self {
        match (s, max_size) {
            ("string", None) => Self::UnboundedString,
            ("bytes", None) => Self::UnboundedBytes,
            ("string", Some(limit)) => Self::String(limit),
            ("bytes", Some(limit)) => Self::Bytes(limit),
            ("bool", _) => Self::Bool,
            ("int32", _) => Self::Int32,
            ("int64", _) => Self::Int64,
            ("uint32", _) => Self::Uint32,
            ("uint64", _) => Self::Uint64,
            // if we don't recognize the type we assume it is a Message type
            // this will be verified later
            (s, Some(limit)) =>  Self::MessageType(s.to_string(), limit),
            (s, None) =>  Self::UnboundedMessageType(s.to_string()),
        }
    }
}

pub type Identifier = String;
pub type Ordinal = i32;

#[derive(Debug, Clone)]
pub enum FieldQualifier {
    Optional,
    Required,
    RepeatedUnbounded,
    Repeated(usize)
}

impl FieldQualifier {
    pub fn from_str(s: &str, max_size: Option<usize>) -> Self {
        match (s, max_size) {
            ("optional", _) => Self::Optional,
            ("required", _) => Self::Required,
            ("repeated", Some(limit)) => Self::Repeated(limit),
            ("repeated", None) => Self::RepeatedUnbounded,
            _ => unreachable!()
        }
    }
}


#[derive(Debug, Clone)]
pub struct MessageField {
    pub qualifier: FieldQualifier,
    pub field_type: FieldType,
    pub identifier: Identifier,
    pub ordinal: Ordinal,
}

#[derive(Debug, Clone)]
pub struct MessageType {
    pub identifier: String,
    pub fields: BTreeMap<Ordinal, MessageField>,
}

#[derive(Debug)]
pub struct EnumType {
    pub identifier: String,
    pub pairs: BTreeMap<String, i32>,
}

#[derive(Debug)]
pub enum Version {
    Proto2,
    Unknown,
}
