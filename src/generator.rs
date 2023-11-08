/// This module contains the code generator
/// It takes the parsed result and creates rust code from the input
use convert_case::{Case, Casing};
use crate::parser::{
    EnumType, FieldQualifier, FieldType, MessageType, ParserError, ProtoParser, Version,
};
use std::{
    collections::{HashMap, HashSet},
    error::Error, ops::ControlFlow,
};

use std::iter::zip;

#[derive(Debug)]
pub enum GeneratorError {
    InvalidProtoVersion,
    FailedToMakeUppercase,
    MissingTypeDefinition(String),
}

pub type Result<T> = std::result::Result<T, GeneratorError>;

fn field_to_rust_type(qualifier: &FieldQualifier, field_type: &FieldType) -> String {
    match (qualifier, field_type) {
        (FieldQualifier::Required, field_type) => match field_type {
            FieldType::Bool => "bool".to_owned(),
            FieldType::UnboundedString => "String".to_owned(),
            FieldType::String(limit) => format!("ArrayString<{limit}>"),
            FieldType::Bytes(limit) => format!("[u8; {}]", limit),
            FieldType::UnboundedBytes => format!("bytes::Bytes"),
            FieldType::Int32 => "i32".to_owned(),
            FieldType::Int64 => "i64".to_owned(),
            FieldType::Uint64 => "u64".to_owned(),
            FieldType::Uint32 => "u32".to_owned(),
            FieldType::MessageType(s, _) => format!("{s}"),
            FieldType::UnboundedMessageType(s) => format!("{s}"),
        },
        (FieldQualifier::Optional, field_type) => match field_type {
            FieldType::Bool => "Option<bool>".to_owned(),
            FieldType::UnboundedString => "Option<String>".to_owned(),
            FieldType::String(limit) => format!("Option<ArrayString<{limit}>>"),
            FieldType::Bytes(limit) => format!("Option<[u8; {}]>", limit),
            FieldType::UnboundedBytes => format!("Option<bytes::Bytes>"),
            FieldType::Int32 => "Option<i32>".to_owned(),
            FieldType::Int64 => "Option<i64>".to_owned(),
            FieldType::Uint64 => "Option<u64>".to_owned(),
            FieldType::Uint32 => "Option<u32>".to_owned(),
            FieldType::MessageType(s, _) => format!("Option<{s}>"),
            FieldType::UnboundedMessageType(s) => format!("Option<{s}>"),
        },
        (FieldQualifier::RepeatedUnbounded, field_type) => {
            format!(
                "Vec<{}>",
                field_to_rust_type(&FieldQualifier::Required, field_type)
            )
        }
        (FieldQualifier::Repeated(limit), field_type) => {
            format!(
                "[{}; {}]",
                field_to_rust_type(&FieldQualifier::Required, field_type),
                limit
            )
        }
    }
}

fn enum_id_to_pascal(identifier: &str) -> Result<String> {
    Ok(identifier.to_case(Case::UpperCamel))
}

fn generate_enum_from_trait(enum_type: &EnumType) -> Result<()> {
    // TryFrom
    println!("impl TryFrom<usize> for {} {{", enum_type.identifier);
    println!("    type Error = String;");
    println!("    fn try_from(value: usize) -> Result<Self, Self::Error> {{");
    println!("        match value {{");
    for (identifier, ordinal) in enum_type.pairs.iter() {
        println!("            {} => Ok({}::{}),", ordinal, enum_type.identifier, enum_id_to_pascal(identifier)?);
    }
    println!("            _ => Err(format!(\"invalid ordinal value: {{}} for enum {}\", value)),", enum_type.identifier);
    println!("        }}");
    println!("    }}");
    println!("}}");


    // TryTo
    println!("impl Into<usize> for {} {{", enum_type.identifier);
    println!("    fn into(self) -> usize {{");
    println!("        match self {{");
    for (identifier, ordinal) in enum_type.pairs.iter() {
        println!("            {}::{} => {},", enum_type.identifier, enum_id_to_pascal(identifier)?, ordinal);
    }
    println!("        }}");
    println!("    }}");
    println!("}}");
    Ok(())
}

fn generate_enums(enums: &HashMap<String, EnumType>) -> Result<()> {
    for (_, enum_type) in enums.iter() {
        println!("pub enum {} {{", enum_type.identifier);
        for (identifier, _) in enum_type.pairs.iter() {
            println!(
                "    {},",
                enum_id_to_pascal(identifier)?,
            )
        }
        println!("}}");

        generate_enum_from_trait(enum_type)?;

        // TODO: impl decoder
        // TODO: impl encoder
    }
    Ok(())
}

fn generate_messages(message_types: &HashMap<String, MessageType>) -> Result<()> {
    for (_, message_type) in message_types.iter() {
        println!("pub struct {} {{", message_type.identifier);
        for (_, field) in message_type.fields.iter() {
            println!(
                "    pub {}: {},",
                field.identifier,
                field_to_rust_type(&field.qualifier, &field.field_type)
            )
        }
        println!("}}");
        // TODO: impl decoder
        // TODO: impl encoder
    }
    Ok(())
}

pub fn generate(parser: &ProtoParser) -> Result<()> {
    generate_enums(&parser.enum_types)?;
    generate_messages(&parser.message_types)?;
    Ok(())
}
