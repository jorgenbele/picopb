use std::collections::BTreeMap;

use crate::wiretypes::{WireType, Field};


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


    pub fn repr(&self) -> String {
        match self {
            Self::UnboundedString => "FieldType::UnboundedString".into(),
            Self::UnboundedBytes => "FieldType::UnboundedBytes".into(),
            Self::String(n) => format!("FieldType::String({})", n),
            Self::Bytes(n) => format!("FieldType::Bytes({})", n),
            Self::Bool => "FieldType::Bool".into(),
            Self::Int32 => "FieldType::Int32".into(),
            Self::Int64 => "FieldType::Int64".into(),
            Self::Uint64 => "FieldType::Uint64".into(),
            Self::Uint32 => "FieldType::Uint32".into(),
            Self::EnumType(identifier) => format!("FieldType::EnumType(\"{}\")", identifier),
            Self::MessageType(identifier, size) => format!("FieldType::MessageType(\"{}\", {})", identifier, size),
            Self::UnboundedMessageType(identifier) => format!("FieldType::UnboundedMessageType(\"{}\")", identifier),
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
    Repeated(usize),

    /// These are for fields that have the [packed=true] option
    /// default for proto version 2.
    PackedRepeatedUnbounded,
    PackedRepeated(usize),
}

impl FieldQualifier {
    pub fn repr(&self) -> String {
        match *self {
            Self::Optional => "FieldQualifier::Optional".into(),
            Self::Required => "FieldQualifier::Required".into(),
            Self::RepeatedUnbounded => "FieldQualifier::RepeatedUnbounded".into(),
            Self::Repeated(len) => format!("FieldQualifier::Repeated({})", len), 
            Self::PackedRepeated(len) => format!("FieldQualifier::PackedRepeated({})", len), 
            Self::PackedRepeatedUnbounded => format!("FieldQualifier::PackedRepeatedUnbounded"), 
        }   
    }
}

/// FieldOption represents a single parsed option
pub enum FieldOption {
    MaxSize(usize),
    MaxLen(usize),
    Packed(bool)
}

#[derive(Debug)]
pub struct FieldOptions {
    pub max_size: Option<usize>,
    pub max_len: Option<usize>,
    pub packed: bool,
}

impl Default for FieldOptions {
    fn default() -> Self {
        Self {
            max_size: None,
            max_len: None,
            packed: false,
        }
    }
}


impl FieldQualifier {
    pub fn from_str(s: &str, options: &FieldOptions) -> Self {
        // TODO: how should max_len and max_size be handled?
        let max_size = match (options.max_len, options.max_size) {
            (None, Some(max_size)) => Some(max_size),
            (Some(max_len), Some(max_size)) => {
                if max_len + 1 > max_size {
                    Some(max_len + 1)
                } else {
                    Some(max_size)
                }
            },
            (Some(max_len), None) => Some(max_len + 1),
            (None, None) => None,
        };

        match (s, max_size, options.packed) {
            ("optional", _, _) => Self::Optional,
            ("required", _, _) => Self::Required,

            // handle packed
            ("repeated", Some(limit), false) => Self::Repeated(limit),
            ("repeated", None, false) => Self::RepeatedUnbounded,
            ("repeated", Some(limit), true) => Self::PackedRepeated(limit),
            ("repeated", None, true) => Self::PackedRepeatedUnbounded,
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


#[derive(Debug)]
/// Packed is used to encode the fact that the type should
/// be encoded and decoded using the [packed=true] option
/// in the type system.
pub struct Packed<T>(pub T);