/// This module contains the code generator
use crate::common::{EnumType, FieldQualifier, FieldType, MessageType, MessageField, Identifier};
use crate::parser::ProtoParser;
/// It takes the parsed result and creates rust code from the input
use convert_case::{Case, Casing};
use std::collections::HashMap;

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
            FieldType::EnumType(s) => format!("{s}"),
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
            FieldType::EnumType(s) => format!("Option<{s}>"),
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
        },

        (FieldQualifier::PackedRepeatedUnbounded, field_type) => {
            format!(
                "picopb::common::Packed<Vec<{}>>",
                field_to_rust_type(&FieldQualifier::Required, field_type)
            )
        }
        (FieldQualifier::PackedRepeated(limit), field_type) => {
            format!(
                "picopb::common::Packed<[{}; {}]>",
                field_to_rust_type(&FieldQualifier::Required, field_type),
                limit
            )
        }
    }
}

fn enum_id_to_pascal(identifier: &str) -> Result<String> {
    Ok(identifier.to_case(Case::UpperCamel))
}

fn identifier_to_const_case(identifier: &str) -> Result<String> {
    Ok(identifier.to_case(Case::UpperSnake))
}

fn generate_enum_from_trait(enum_type: &EnumType) -> Result<()> {
    // TryFrom
    println!("impl TryFrom<usize> for {} {{", enum_type.identifier);
    println!("    type Error = String;");
    println!("    fn try_from(value: usize) -> Result<Self, Self::Error> {{");
    println!("        match value {{");
    for (identifier, ordinal) in enum_type.pairs.iter() {
        println!(
            "            {} => Ok({}::{}),",
            ordinal,
            enum_type.identifier,
            enum_id_to_pascal(identifier)?
        );
    }
    println!(
        "            _ => Err(format!(\"invalid ordinal value: {{}} for enum {}\", value)),",
        enum_type.identifier
    );
    println!("        }}");
    println!("    }}");
    println!("}}");

    // TryTo
    println!("impl Into<usize> for {} {{", enum_type.identifier);
    println!("    fn into(self) -> usize {{");
    println!("        match self {{");
    for (identifier, ordinal) in enum_type.pairs.iter() {
        println!(
            "            {}::{} => {},",
            enum_type.identifier,
            enum_id_to_pascal(identifier)?,
            ordinal
        );
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
            println!("    {},", enum_id_to_pascal(identifier)?,)
        }
        println!("}}");

        generate_enum_from_trait(enum_type)?;

        // TODO: impl decoder
        // TODO: impl encoder
    }
    Ok(())
}

fn generate_message_metadata(message_type: &MessageType) -> Result<()> {
    let message_type_identifier = identifier_to_const_case(&message_type.identifier)?;

    // generate struct that holds fields metadata
    println!("pub struct {}FieldsType {{", message_type.identifier);
    for (_, field) in message_type.fields.iter() {
        println!("    pub {}: picopb::common::ConstMessageField,", field.identifier);
    }
    println!("}}");

    // generate const struct that fills that struct 
    println!("const {}_FIELDS: {}FieldsType = {}FieldsType {{", message_type_identifier, message_type.identifier, message_type.identifier);
    for (_, field) in message_type.fields.iter() {
        println!("    {}: picopb::common::ConstMessageField {{", field.identifier);
        println!("        qualifier: {},", field.qualifier.repr());
        println!("        field_type: {},", field.field_type.repr());
        println!("        identifier: \"{}\",", field.identifier);
        println!("        ordinal: picopb::common::Field({}),", field.ordinal.0);
        println!("    }},");
    }
    println!("}};");

    // impl self::fields() that returns the fields type
    println!("impl {} {{", message_type.identifier);
    println!("    fn fields(&self) -> {}FieldsType {{", message_type.identifier);
    println!("        {}_FIELDS", identifier_to_const_case(message_type.identifier.as_str())?);
    println!("    }}");
    println!("}}");


    Ok(())
}

fn as_encodable_type(field: &MessageField, prefix: &str) -> String {
    let identifier = &field.identifier;

    let wrapped = match field.qualifier {
        FieldQualifier::Repeated(_) | FieldQualifier::PackedRepeated(_) 
            => return format!("{prefix}{identifier}"),
        FieldQualifier::PackedRepeatedUnbounded | FieldQualifier::RepeatedUnbounded 
            => return format!("{prefix}{identifier}.as_slice()"),
        _ => format!("{prefix}{identifier}"),
    };

    match field.field_type {
        FieldType::UnboundedString => format!("{wrapped}.as_str()"),
        FieldType::UnboundedBytes => todo!(),
        FieldType::String(_) => format!("{wrapped}.as_slice()"),
        FieldType::Bytes(_) => format!("{wrapped}.as_bytes()"),
        FieldType::EnumType(_) => todo!(),
        FieldType::MessageType(_, _) => todo!(),
        FieldType::UnboundedMessageType(_) => todo!(),
        FieldType::Bool | FieldType::Int32 | FieldType::Int64 | FieldType::Uint32 | FieldType::Uint64  => format!("{wrapped}"),
    }
}

fn generate_message_encode(message_type: &MessageType) -> Result<()> {
    println!("impl picopb::encode::Encode for &{} {{", message_type.identifier);
    println!("    fn encode(&self, buf: &mut picopb::encode::EncodeBuffer) -> std::io::Result<usize> {{");
    println!("        let mut total_size = 0;");
    for (_, field) in message_type.fields.iter() {
        let identifier = &field.identifier;

        // We don't want to encode empty optional values
        match field.qualifier {
            FieldQualifier::Optional => {
                let value_encodable_type = as_encodable_type(field, "value_");
                println!("        if let Some(value_{identifier}) = &self.{identifier} {{");
                println!("            total_size += buf.encode({value_encodable_type}, self.fields().{identifier}.ordinal)?;");
                println!("        }}");
            },
            _ => {
                let self_encodable_type = as_encodable_type(field, "self.");
                println!("        total_size += buf.encode({self_encodable_type}, self.fields().{identifier}.ordinal)?;");
            }
        }



    }
    println!("        Ok(total_size)");
    println!("    }}");

    println!("    fn precalculate_size(&self) -> usize {{");
    println!("        let mut total_size = 0;");
    for (_, field) in message_type.fields.iter() {
        let encodable_type = as_encodable_type(field, "");
        let self_encodable_type = as_encodable_type(field, "self.");
        let identifier = &field.identifier;

        // We don't want to encode empty optional values
        // and therefore should not count them towards the size
        match field.qualifier {
            FieldQualifier::Optional => {
                println!("        if let Some(value_{identifier}) = &self.{identifier} {{");
                println!("            total_size += value_{encodable_type}.precalculate_size();");
                println!("        }}");
            },
            _ => {
                println!("        total_size += {self_encodable_type}.precalculate_size();");
            }
        }

    }
    println!("        total_size");
    println!("    }}");

    println!("}}");
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
        generate_message_metadata(message_type)?;
        // TODO: impl decoder

        // TODO: impl encoder
        generate_message_encode(message_type)?;
    }
    Ok(())
}

fn generate_imports() {
    println!("use picopb::common::*;");
    println!("use picopb::encode::ToWire;");
    println!("use picopb::encode::Encode;");
}

pub fn generate(parser: &ProtoParser) -> Result<()> {
    generate_imports();
    generate_enums(&parser.enum_types)?;
    generate_messages(&parser.message_types)?;
    Ok(())
}
