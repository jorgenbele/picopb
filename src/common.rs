use std::collections::BTreeMap;

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
            (s, Some(limit)) => Self::MessageType(s.to_string(), limit),
            (s, None) => Self::UnboundedMessageType(s.to_string()),
        }
    }

    pub fn repr(&self) -> String {
        match self {
            Self::UnboundedString => "picopb::common::FieldType::UnboundedString".into(),
            Self::UnboundedBytes => "picopb::common::FieldType::UnboundedBytes".into(),
            Self::String(n) => format!("picopb::common::FieldType::String({})", n),
            Self::Bytes(n) => format!("picopb::common::FieldType::Bytes({})", n),
            Self::Bool => "picopb::common::FieldType::Bool".into(),
            Self::Int32 => "picopb::common::FieldType::Int32".into(),
            Self::Int64 => "picopb::common::FieldType::Int64".into(),
            Self::Uint64 => "picopb::common::FieldType::Uint64".into(),
            Self::Uint32 => "picopb::common::FieldType::Uint32".into(),
            Self::EnumType(identifier) => {
                format!("picopb::common::FieldType::EnumType(\"{}\")", identifier)
            }
            Self::MessageType(identifier, size) => format!(
                "picopb::common::FieldType::MessageType(\"{}\", {})",
                identifier, size
            ),
            Self::UnboundedMessageType(identifier) => format!(
                "picopb::common::FieldType::UnboundedMessageType(\"{}\")",
                identifier
            ),
        }
    }
}

pub type Identifier = String;
pub type Ordinal = u32;

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
            Self::Optional => "picopb::common::FieldQualifier::Optional".into(),
            Self::Required => "picopb::common::FieldQualifier::Required".into(),
            Self::RepeatedUnbounded => "picopb::common::FieldQualifier::RepeatedUnbounded".into(),
            Self::Repeated(len) => format!("picopb::common::FieldQualifier::Repeated({})", len),
            Self::PackedRepeated(len) => {
                format!("picopb::common::FieldQualifier::PackedRepeated({})", len)
            }
            Self::PackedRepeatedUnbounded => {
                "picopb::common::FieldQualifier::PackedRepeatedUnbounded".to_string()
            }
        }
    }
}

/// FieldOption represents a single parsed option
pub enum FieldOption {
    MaxSize(usize),
    MaxLen(usize),
    Packed(bool),
}

#[derive(Debug)]
#[derive(Default)]
pub struct FieldOptions {
    pub max_size: Option<usize>,
    pub max_len: Option<usize>,
    pub packed: bool,
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
            }
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
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field(pub u32);

#[derive(Debug, Clone)]
pub struct MessageField {
    pub qualifier: FieldQualifier,
    pub field_type: FieldType,
    pub identifier: Identifier,
    pub ordinal: Field,
}

#[derive(Debug, Clone)]
pub struct ConstMessageField {
    pub qualifier: FieldQualifier,
    pub field_type: FieldType,
    pub identifier: &'static str,
    pub ordinal: Field,
}

#[derive(Debug, Clone)]
pub struct MessageType {
    pub identifier: String,
    pub fields: BTreeMap<Ordinal, MessageField>,
}

#[derive(Debug)]
pub struct EnumType {
    pub identifier: String,
    pub pairs: BTreeMap<String, Ordinal>,
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
