use std::{collections::{BTreeMap, HashMap}, hash::Hash};
use std::num::ParseIntError;

use pest::{Parser, error::Error as PestError, iterators::Pair as PestPair, state, Span};
/// This file contains the proto definition parser for proto v2
/// It is implemented using pest
use pest_derive::Parser;

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
pub struct EnumType {
    pub identifier: String,
    pub pairs: BTreeMap<String, i32>,
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct MessageField {
    pub qualifier: FieldQualifier,
    pub field_type: FieldType,
    pub identifier: Identifier,
    pub ordinal: Ordinal,
}

#[derive(Debug)]
pub struct MessageType {
    pub identifier: String,
    pub fields: BTreeMap<Ordinal, MessageField>,
}

#[derive(Debug)]
pub enum Version {
    Proto2,
    Unknown,
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

impl ProtoParser {

    fn parse_message_definition(&mut self, message_statement: PestPair<'_, Rule>) -> EmptyParseResult {
        dbg!(&message_statement);

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
                    let max_size = None;

                    let field_identifier = Self::identifier_from_span(identifier.as_span());
                    let field_ordinal = Self::ordinal_from_span(field_number.as_span())?;


                    let value = MessageField {
                        qualifier: FieldQualifier::from_str(qualifier, max_size),
                        field_type: FieldType::from_str(field_type, max_size),
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
        dbg!(&statement);
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

    let mut output = ProtoParser { 
        version: Version::Unknown,
        imports: Vec::new(),
        enum_types: HashMap::new(),
        message_types: HashMap::new()
    };

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
    Ok(output)
}