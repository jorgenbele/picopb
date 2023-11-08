/// This module contains the code generator 
/// It takes the parsed result and creates rust code from the input

use crate::parser::{ProtoParser, Version, EnumType, MessageType, FieldType, ParserError, FieldQualifier};
use std::{error::Error, collections::HashSet};

#[derive(Debug)]
pub enum GeneratorError {
    InvalidProtoVersion,
    MissingTypeDefinition(String),
}

fn field_to_rust_type(qualifier: &FieldQualifier, field_type: &FieldType) -> String {
    match (qualifier, field_type) {
        (FieldQualifier::Required, field_type) => {
            match field_type {
                    FieldType::Bool => "bool".to_owned(),
                    FieldType::UnboundedString => "String".to_owned(),
                    FieldType::String(limit) => format!("ArrayString<{limit}>"),
                    FieldType::Bytes(limit) => format!("Bytes::bytes"),
                    FieldType::UnboundedBytes => format!("Bytes::bytes"),
                    FieldType::Int32 => "i32".to_owned(),
                    FieldType::Int64 => "i64".to_owned(),
                    FieldType::Uint64 => "u64".to_owned(),
                    FieldType::Uint32 => "u32".to_owned(),
                    FieldType::MessageType(s, limit) => format!("{s}"),
                    FieldType::UnboundedMessageType(s) => format!("{s}"),
            }
        },
        (FieldQualifier::Optional, field_type) => {
            match field_type {
                    FieldType::Bool => "Option<bool>".to_owned(),
                    FieldType::UnboundedString => "Option<String>".to_owned(),
                    FieldType::String(limit) => format!("Option<ArrayString<{limit}>>"),
                    FieldType::Bytes(limit) => format!("Option<Bytes::bytes>"),
                    FieldType::UnboundedBytes => format!("Option<Bytes::bytes>"),
                    FieldType::Int32 => "Option<i32>".to_owned(),
                    FieldType::Int64 => "Option<i64>".to_owned(),
                    FieldType::Uint64 => "Option<u64>".to_owned(),
                    FieldType::Uint32 => "Option<u32>".to_owned(),
                    FieldType::MessageType(s, limit) => format!("Option<{s}>"),
                    FieldType::UnboundedMessageType(s) => format!("Option<{s}>"),
            }
        },
        (FieldQualifier::RepeatedUnbounded, field_type) => {
            format!("Vec<{}>", field_to_rust_type(&FieldQualifier::Required, field_type))
        },
        (FieldQualifier::Repeated(limit), field_type) => {
            format!("[{}; {}]", field_to_rust_type(&FieldQualifier::Required, field_type), limit)
        },

    }
}

pub fn generate(parser: &ProtoParser) -> Result<(), GeneratorError> {
    for (_, message_type) in parser.message_types.iter() {
        println!("pub struct {} {{", message_type.identifier);
        for (_, field) in message_type.fields.iter() {
            println!("    pub {}: {},", field.identifier, field_to_rust_type(&field.qualifier, &field.field_type))
        }
        println!("}}");

        // TODO: impl decoder
        // TODO: impl encoder
    }
    Ok(())
}