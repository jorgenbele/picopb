use crate::common::{FieldType, Version};
/// This module contains a basic validator for the protobuf definition
/// It will check that all types are valid (either defined or builtin)
use crate::parser::{ParserError, ProtoParser};

use std::collections::HashSet;

#[derive(Debug)]
pub enum ValidatorError {
    InvalidProtoVersion,
    MissingTypeDefinition(String),
}

pub fn validate(parser: &ProtoParser) -> Result<(), ValidatorError> {
    if let Version::Unknown = parser.version {
        return Err(ValidatorError::InvalidProtoVersion);
    }

    // create lookup table of all valid types
    let mut valid_message_types = HashSet::new();
    parser.message_types.iter().for_each(|(id, _)| {
        valid_message_types.insert(id);
    });
    parser.enum_types.iter().for_each(|(id, _)| {
        valid_message_types.insert(id);
    });

    for (_, message_type) in parser.message_types.iter() {
        for (_, field) in message_type.fields.iter() {
            match &field.field_type {
                FieldType::MessageType(identifier, _)
                | FieldType::UnboundedMessageType(identifier) => {
                    if !valid_message_types.contains(identifier) {
                        return Err(ValidatorError::MissingTypeDefinition(
                            identifier.to_string(),
                        ));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
