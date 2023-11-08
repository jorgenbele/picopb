use std::{collections::{BTreeMap, HashMap}, hash::Hash};
use std::num::ParseIntError;

use pest::{Parser, error::Error as PestError, iterators::Pair as PestPair, iterators::Pairs as PestPairs, Span};
/// This file contains the proto definition parser for proto v2
/// It is implemented using pest
use pest_derive::Parser;

use crate::common::{FieldQualifier, FieldType, MessageField, MessageType, Version, EnumType};

#[derive(Parser, Debug)]
#[grammar = "parser.pest"] // relative to src
pub struct PicoPBParser;

#[derive(Debug)]
pub enum ParserError {
    InvalidProtoDefinition,
    ExpectedStatement,
    InvalidVersionDeclaration,
    InvalidProtoVersion,
    DuplicateProtoVersion,
    ImportMustBeNonEmpty,
    ExpectedOption,
    ExpectedOptionValue,
    UnknownOption(String),
    PestRuleError(PestError<Rule>),
    ParseIntError(ParseIntError),
    ExpectedButGot(String, String),
}

impl From<PestError<Rule>> for ParserError {
    fn from(error: PestError<Rule>) -> Self {
        ParserError::PestRuleError(error)
    }
}


impl From<ParseIntError> for ParserError {
    fn from(error: ParseIntError) -> Self {
        ParserError::ParseIntError(error)
    }
}



#[derive(Debug)]
pub struct ProtoParser {
    pub version: Version,
    pub imports: Vec<String>,
    pub enum_types: HashMap<String, EnumType>,
    pub message_types: HashMap<String, MessageType>,
}

type ParseResult = Result<ProtoParser, ParserError>;
type EmptyParseResult = Result<(), ParserError>;

#[derive(Debug)]
struct MaxOption {
    max_size: Option<usize>,
    max_len: Option<usize>,
}

impl ProtoParser {

    fn usize_from_str(s: &str) -> Result<usize, ParserError> {
        str::parse::<usize>(s).map_err(|err| ParserError::ParseIntError(err))
    }

    fn parse_options(&mut self, options_statement: PestPairs<'_, Rule>) -> Result<MaxOption, ParserError> {
        let mut out = MaxOption { max_len: None, max_size: None};

        for option in options_statement.into_iter() {
            match option.as_rule() {
                Rule::option => {
                    let mut inner = option.into_inner();
                    let option = inner.next().ok_or(ParserError::ExpectedOption)?;
                    let value = inner.next().ok_or(ParserError::ExpectedOptionValue)?;

                    match option.as_str() {
                        "max_size" => {
                            out.max_size = Some(Self::usize_from_str(value.as_str())?);
                        },
                        "max_len" => {
                            out.max_len = Some(Self::usize_from_str(value.as_str())?);
                        },
                        s => return Err(ParserError::UnknownOption(s.to_string())),
                    }
                },
                _ => unreachable!() 
            }
        }
        Ok(out) 
       }

    fn parse_message_definition(&mut self, message_statement: PestPair<'_, Rule>) -> EmptyParseResult {
        // dbg!(&message_statement);

        let mut inner = message_statement.into_inner();

        let identifier = inner.next().ok_or( ParserError::ExpectedButGot("identifier".to_string(), "None".to_string()))?;
        let identifier = Self::identifier_from_span(identifier.as_span());

        let mut message_type = MessageType { identifier: identifier.clone(), fields: BTreeMap::new() };

        for value in inner {
            match value.as_rule() {
                Rule::message_field => {
                    let mut message_inner = value.into_inner();

                    let qualifier = message_inner.next().ok_or(ParserError::InvalidProtoDefinition)?.as_str();
                    let field_type = message_inner.next().ok_or(ParserError::InvalidProtoDefinition)?.as_str();
                    let identifier = message_inner.next().ok_or(ParserError::InvalidProtoDefinition)?;
                    let field_number = message_inner.next().ok_or(ParserError::InvalidProtoDefinition)?;

                    // TODO:
                    // optional options that can contain the max_size
                    let mut options : MaxOption = MaxOption { max_size: None, max_len: None };


                    if let Some(next) = message_inner.next() {
                        dbg!(&next);
                        match next.as_rule() {
                            Rule::options => {
                                if let Ok(opts)  = self.parse_options(next.into_inner()) {
                                    options = opts;
                                    println!("Updated OPTIONS");
                                }
                            },
                            _ => unreachable!()
                        }
                    }

                    let field_identifier = Self::identifier_from_span(identifier.as_span());
                    let field_ordinal = Self::ordinal_from_span(field_number.as_span())?;


                    let value = MessageField {
                        qualifier: FieldQualifier::from_str(qualifier, options.max_size),
                        field_type: FieldType::from_str(field_type, options.max_size),
                        identifier: field_identifier,
                        ordinal: field_ordinal
                    };
                    message_type.fields.insert(field_ordinal, value);
                }
                _ => unreachable!()
            }
        }
        self.message_types.insert(identifier, message_type);

        Ok(())
    }

    fn identifier_from_span<'i>(span: Span<'i>) -> String {
        let identifier_str = span.as_str();
        identifier_str.to_string()
    }

    fn string_from_span<'i>(span: Span<'i>) -> String {
        let identifier_str = span.as_str();
        identifier_str[1..identifier_str.len()-1].to_string()
    }

    fn ordinal_from_span<'i>(span: Span<'i>) -> Result<i32, ParserError> {
        let ordinal_str = span.as_str();
        let parse_result = ordinal_str.parse::<i32>();
        parse_result.map_err(ParserError::from)
    }

    fn parse_enum_definition(&mut self, enum_statement: PestPair<'_, Rule>) -> EmptyParseResult {
        let mut inner = enum_statement.into_inner();

        let identifier = inner.next().ok_or( ParserError::ExpectedButGot("identifier".to_string(), "None".to_string()))?;
        let identifier = Self::identifier_from_span(identifier.as_span());

        let mut enum_type = EnumType { identifier: identifier.clone(), pairs: BTreeMap::new() };

        for value in inner {
            match value.as_rule() {
                Rule::enum_field => {
                    let mut enum_inner = value.into_inner();
                    let identifier = enum_inner.next().ok_or(ParserError::InvalidProtoDefinition)?;
                    let number = enum_inner.next().ok_or(ParserError::InvalidProtoDefinition)?;

                    let field_identifier = Self::identifier_from_span(identifier.as_span());
                    let field_ordinal = Self::ordinal_from_span(number.as_span())?;

                    enum_type.pairs.insert(field_identifier, field_ordinal);
                }
                _ => unreachable!()
            }
        }
        self.enum_types.insert(identifier, enum_type);
        Ok(())
    }

    fn parse_block_statement(&mut self, statement: PestPair<'_, Rule>) -> EmptyParseResult {
        println!("Parsing block");
        // dbg!(&statement);
        match statement.as_rule() {
            Rule::message_definition => self.parse_message_definition(statement),
            Rule::enum_definition => self.parse_enum_definition(statement),
            _ => Err(ParserError::ExpectedButGot("message or enum definition".to_string(), statement.as_str().to_string()))
        }
    }

    fn parse_import_statement(&mut self, statement: PestPair<'_, Rule>) -> EmptyParseResult {
        println!("Parsing import statement");
        if let Some(value) = statement.into_inner().next() {
            let slice = value.as_span().as_str();
            if slice.len() == 2 {
                return Err(ParserError::ImportMustBeNonEmpty);
            }
            self.imports.push(slice[1..slice.len()-1].to_owned())

            // TODO: open files, and parse them here
        }
        Ok(())
    }

    fn parse_version_decl(&mut self, statement: PestPair<'_, Rule>) -> EmptyParseResult {
        if let Some(value) = statement.into_inner().nth(0) {
            // TODO: 
            if Self::string_from_span(value.as_span()) == "proto2" {
                self.version = Version::Proto2;
                return Ok(())
            } else {
                return Err(ParserError::InvalidProtoVersion)
            }
        }
        Err(ParserError::InvalidVersionDeclaration)
    }

    fn parse_statement(&mut self, statement: PestPair<'_, Rule>) -> EmptyParseResult {
        println!("parsing statement");
        let statement = match statement.as_rule() {
            Rule::statement => statement.into_inner().nth(0).ok_or_else(|| ParserError::ExpectedStatement)?,
            _ => return Err(ParserError::ExpectedButGot("Statement".to_string(), statement.as_node_tag().unwrap_or("<unknown>").to_string())),
        };

        match statement.as_rule() {
            Rule::block_statement => self.parse_block_statement(statement.into_inner().next().ok_or(ParserError::ExpectedStatement)?),
            Rule::import_statement =>  self.parse_import_statement(statement),  
            _ => Err(ParserError::ExpectedButGot("block statement".to_string(), statement.as_node_tag().unwrap_or("<unknown>").to_string()))
        }?;
        Ok(())
    }
}

pub fn parse(input: &str) -> ParseResult {
    let parse = PicoPBParser::parse(Rule::proto_definition, input)?;
    dbg!(&parse);

    let mut output = ProtoParser { 
        version: Version::Unknown,
        imports: Vec::new(),
        enum_types: HashMap::new(),
        message_types: HashMap::new()
    };

    // Do a single pass and extract enum and message types
    for pair in parse.into_iter() {
        match pair.as_rule() {
            Rule::proto_definition => {
                for p in pair.into_inner() {
                    match p.as_rule() {
                        Rule::statement => output.parse_statement(p)?,
                        Rule::version_decl => output.parse_version_decl(p)?,
                        Rule::EOI => break,
                        _ => return Err(ParserError::ExpectedButGot("Statement or version decl".to_string(), p.as_str().to_string())),
                    };
                }
            }
            _ => unreachable!()
        }
    }

    // Now we know which types are primitives, sub-messages, and Enums
    // Iterate through all fields in all message types and fix up those that we now know are 
    // enums
    // TODO: fixup this ugly mess
    let mut new_message_types: HashMap<String, MessageType> = HashMap::new();
    for (identifier, message_type) in output.message_types.into_iter() {
        let mut new_fields = message_type.fields.clone();
        let mut new_message_type = message_type.clone();

        for (ordinal, field) in message_type.fields {
            match &field.field_type {
                FieldType::MessageType(identifier, _) | FieldType::UnboundedMessageType(identifier) => {
                    if output.enum_types.contains_key(identifier) {
                        // this field has a type that we now know is a Enum (not a Message)
                        let mut new_field = field.clone();
                        new_field.field_type = FieldType::EnumType(identifier.clone());
                        new_fields.insert(ordinal, new_field);
                    }
                }
                _ => continue,
            }
        }
        new_message_type.fields = new_fields;
        new_message_types.insert(identifier.clone(), new_message_type);
    }
    output.message_types = new_message_types;
    Ok(output)
}