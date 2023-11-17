use crate::common::{EnumType, FieldQualifier, FieldType, MessageField, MessageType};
use crate::parser::ProtoParser;
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::io::Write;

/// This module contains the code generator
/// It takes the parsed result and creates rust code from the input

#[derive(Debug)]
pub enum GeneratorError {
    InvalidProtoVersion,
    FailedToMakeUppercase,
    MissingTypeDefinition(String),
    IoRrror(std::io::Error),
}

impl From<std::io::Error> for GeneratorError {
    fn from(value: std::io::Error) -> Self {
        Self::IoRrror(value)
    }
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
            FieldType::MessageType(s) => format!("{s}"),
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
            FieldType::MessageType(s) => format!("Option<{s}>"),
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
        }

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

fn generate_enum_from_trait<T: Write>(to: &mut T, enum_type: &EnumType) -> Result<()> {
    // TryFrom
    writeln!(to, "impl TryFrom<usize> for {} {{", enum_type.identifier)?;
    writeln!(to, "    type Error = String;")?;
    writeln!(
        to,
        "    fn try_from(value: usize) -> Result<Self, Self::Error> {{"
    )?;
    writeln!(to, "        match value {{")?;
    for (identifier, ordinal) in enum_type.pairs.iter() {
        writeln!(
            to,
            "            {} => Ok({}::{}),",
            ordinal,
            enum_type.identifier,
            enum_id_to_pascal(identifier)?
        )?;
    }
    writeln!(
        to,
        "            _ => Err(format!(\"invalid ordinal value: {{}} for enum {}\", value)),",
        enum_type.identifier
    )?;
    writeln!(to, "        }}")?;
    writeln!(to, "    }}")?;
    writeln!(to, "}}")?;

    // TryTo
    writeln!(to, "impl Into<usize> for {} {{", enum_type.identifier)?;
    writeln!(to, "    fn into(self) -> usize {{")?;
    writeln!(to, "        match self {{")?;
    for (identifier, ordinal) in enum_type.pairs.iter() {
        writeln!(
            to,
            "            {}::{} => {},",
            enum_type.identifier,
            enum_id_to_pascal(identifier)?,
            ordinal
        )?;
    }
    writeln!(to, "        }}")?;
    writeln!(to, "    }}")?;
    writeln!(to, "}}")?;
    Ok(())
}

fn generate_enums<T: Write>(to: &mut T, enums: &HashMap<String, EnumType>) -> Result<()> {
    for (_, enum_type) in enums.iter() {
        writeln!(to, "#[derive(Default, Debug)]")?;
        writeln!(to, "pub enum {} {{", enum_type.identifier)?;
        let mut first = true;
        for (identifier, _) in enum_type.pairs.iter() {
            if first {
                writeln!(to, "    #[default]")?;
                first = false;
            }
            writeln!(to, "    {},", enum_id_to_pascal(identifier)?)?;
        }
        writeln!(to, "}}")?;

        generate_enum_from_trait(to, enum_type)?;

        // TODO: impl decoder
        // TODO: impl encoder
    }
    Ok(())
}

fn generate_message_metadata<T: Write>(to: &mut T, message_type: &MessageType) -> Result<()> {
    let message_type_identifier = identifier_to_const_case(&message_type.identifier)?;

    // generate struct that holds fields metadata

    writeln!(to, "#[derive(Debug)]")?;
    writeln!(to, "pub struct {}FieldsType<'a> {{", message_type.identifier)?;
    for (_, field) in message_type.fields.iter() {
        writeln!(
            to,
            "    pub {}: picopb::common::ConstMessageField<'a>,",
            field.identifier
        )?;
    }
    writeln!(to, "}}")?;

    // generate const struct that fills that struct
    writeln!(
        to,
        "const {}_FIELDS: {}FieldsType = {}FieldsType {{",
        message_type_identifier, message_type.identifier, message_type.identifier
    )?;
    for (_, field) in message_type.fields.iter() {
        writeln!(
            to,
            "    {}: picopb::common::ConstMessageField {{",
            field.identifier
        )?;
        writeln!(to, "        qualifier: {},", field.qualifier.repr())?;
        writeln!(to, "        field_type: {},", field.field_type.repr())?;
        writeln!(to, "        identifier: \"{}\",", field.identifier)?;
        writeln!(
            to,
            "        ordinal: picopb::common::Field({}),",
            field.ordinal.0
        )?;
        writeln!(to, "    }},")?;
    }
    writeln!(to, "}};")?;

    // impl self::fields() that returns the fields type
    writeln!(to, "impl {} {{", message_type.identifier)?;
    writeln!(
        to,
        "    fn fields(&self) -> {}FieldsType {{",
        message_type.identifier
    )?;
    writeln!(
        to,
        "        {}_FIELDS",
        identifier_to_const_case(message_type.identifier.as_str())?
    )?;
    writeln!(to, "    }}")?;
    writeln!(to, "}}")?;

    Ok(())
}

fn as_encodable_type(field: &MessageField, prefix: &str) -> String {
    let identifier = &field.identifier;

    let wrapped = match field.qualifier {
        FieldQualifier::Repeated(_) | FieldQualifier::PackedRepeated(_) => {
            return format!("{prefix}{identifier}")
        }
        FieldQualifier::PackedRepeatedUnbounded | FieldQualifier::RepeatedUnbounded => {
            return format!("{prefix}{identifier}.as_slice()")
        }
        _ => format!("{prefix}{identifier}"),
    };

    match field.field_type {
        FieldType::UnboundedString => format!("{wrapped}.as_str()"),
        FieldType::UnboundedBytes => format!("{wrapped}.deref()"),
        FieldType::String(_) => format!("{wrapped}.as_slice()"),
        FieldType::Bytes(_) => format!("{wrapped}.as_bytes()"),
        FieldType::EnumType(_) => todo!(),
        FieldType::MessageType(_) => format!("(&{wrapped})"),
        FieldType::Bool
        | FieldType::Int32
        | FieldType::Int64
        | FieldType::Uint32
        | FieldType::Uint64 => format!("{wrapped}"),
    }
}

fn generate_message_wiretyped<T: Write>(to: &mut T, message_type: &MessageType) -> Result<()> {
    writeln!(
        to,
        "impl picopb::wiretypes::WireTyped for &{} {{",
        message_type.identifier
    )?;
    writeln!(to, "    fn wiretype(&self) -> WireType {{")?;
    writeln!(to, "        WireType::Len")?;
    writeln!(to, "    }}")?;
    writeln!(to, "}}")?;
    Ok(())
}

fn generate_message_to_wire<T: Write>(to: &mut T, message_type: &MessageType) -> Result<()> {
    writeln!(
        to,
        "impl picopb::encode::ToWire for &{} {{",
        message_type.identifier
    )?;
    writeln!(
        to,
        "    fn append(&self, buf: &mut picopb::encode::EncodeBuffer) -> std::io::Result<usize> {{"
    )?;
    writeln!(to, "        let mut total_size = 0;")?;
    for (_, field) in message_type.fields.iter() {
        let identifier = &field.identifier;

        // We don't want to encode empty optional values
        match field.qualifier {
            FieldQualifier::Optional => {
                let value_encodable_type = as_encodable_type(field, "value_");
                writeln!(
                    to,
                    "        if let Some(value_{identifier}) = &self.{identifier} {{"
                )?;
                writeln!(to, "            total_size += buf.encode({value_encodable_type}, self.fields().{identifier}.ordinal)?;")?;
                writeln!(to, "        }}")?;
            }
            _ => {
                let self_encodable_type = as_encodable_type(field, "self.");
                writeln!(to, "        total_size += buf.encode({self_encodable_type}, self.fields().{identifier}.ordinal)?;")?;
            }
        }
    }
    writeln!(to, "        Ok(total_size)")?;
    writeln!(to, "    }}")?;

    writeln!(to, "    fn precalculate_size(&self) -> usize {{")?;
    writeln!(to, "        let mut total_size = 0;")?;
    for (_, field) in message_type.fields.iter() {
        let encodable_type = as_encodable_type(field, "");
        let self_encodable_type = as_encodable_type(field, "self.");
        let identifier = &field.identifier;

        // We don't want to encode empty optional values
        // and therefore should not count them towards the size
        match field.qualifier {
            FieldQualifier::Optional => {
                writeln!(
                    to,
                    "        if let Some(value_{identifier}) = &self.{identifier} {{"
                )?;
                writeln!(
                    to,
                    "            total_size += value_{encodable_type}.precalculate_size();"
                )?;
                writeln!(to, "        }}")?;
            }
            _ => {
                writeln!(
                    to,
                    "        total_size += {self_encodable_type}.precalculate_size();"
                )?;
            }
        }
    }
    writeln!(to, "        total_size")?;
    writeln!(to, "    }}")?;

    writeln!(to, "}}")?;
    Ok(())
}

/// Generate implementation of the Randomize trait for the message
fn generate_message_impl_randomize<T: Write>(to: &mut T, message_type: &MessageType) -> Result<()> {
    writeln!(
        to,
        "impl Randomize<{0}> for {0} {{",
        message_type.identifier
    )?;
    writeln!(to, "    fn randomized() -> {} {{", message_type.identifier)?;
    writeln!(to, "        Self {{")?;
    for (_, field) in message_type.fields.iter() {
        let rust_type = field_to_rust_type(&field.qualifier, &field.field_type);
        writeln!(
            to,
            "            {}: randomized::<{rust_type}>(),",
            field.identifier
        )?;
    }
    writeln!(to, "        }}")?;
    writeln!(to, "    }}")?;
    writeln!(to, "}}")?;
    Ok(())
}

fn generate_messages<T: Write>(
    to: &mut T,
    message_types: &HashMap<String, MessageType>,
) -> Result<()> {
    for (_, message_type) in message_types.iter() {
        writeln!(to, "#[derive(Default, Debug)]")?;
        writeln!(to, "pub struct {} {{", message_type.identifier)?;
        for (_, field) in message_type.fields.iter() {
            writeln!(
                to,
                "    pub {}: {},",
                field.identifier,
                field_to_rust_type(&field.qualifier, &field.field_type)
            )?;
        }
        writeln!(to, "}}")?;
        generate_message_metadata(to, message_type)?;
        // TODO: impl decoder

        generate_message_wiretyped(to, message_type)?;
        generate_message_to_wire(to, message_type)?;
        generate_message_impl_randomize(to, message_type)?;
    }
    Ok(())
}

fn generate_imports<T: Write>(to: &mut T) -> std::io::Result<()> {
    writeln!(to, "use picopb::common::*;")?;
    writeln!(to, "use picopb::encode::ToWire;")?;
    writeln!(to, "use picopb::wiretypes::{{WireType, WireTyped}};")?;
    writeln!(to, "use picopb::randomizer::{{randomized, Randomize}};")?;
    writeln!(to, "use std::ops::Deref;")
}

pub fn generate<T: Write>(to: &mut T, parser: &ProtoParser) -> Result<()> {
    generate_imports(to)?;
    generate_enums(to, &parser.enum_types)?;
    generate_messages(to, &parser.message_types)?;
    Ok(())
}
