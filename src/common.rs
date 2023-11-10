use std::collections::BTreeMap;

use crate::wiretypes::WireType;


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
