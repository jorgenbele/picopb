use std::num::ParseIntError;
use std::option;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use pest::state;
use pest::{
    error::Error as PestError, iterators::Pair as PestPair, iterators::Pairs as PestPairs, Parser,
    Span,
};
/// This file contains the proto definition parser for proto v2
/// It is implemented using pest
use pest_derive::Parser;

use crate::common::{EnumType, FieldQualifier, Field, FieldType, MessageField, MessageType, Version, FieldOption, FieldOptions};

#[derive(Parser, Debug)]
#[grammar = "parser.pest"] // relative to src
pub struct PicoPBParser;

#[derive(Debug)]
pub struct StaticSpan {
    pub start: usize,
    pub end: usize,
    pub string: String,
}

impl From<&Span<'_>> for StaticSpan {
    fn from(span: &Span<'_>) -> Self {
        Self { start: span.start(), end: span.end(), string: span.as_str().to_string() }
    }
}

impl From<Span<'_>> for StaticSpan {
    fn from(span: Span<'_>) -> Self {
        Self { start: span.start(), end: span.end(), string: span.as_str().to_string() }
    }
}

#[derive(Debug)]
pub enum ParserError {
    InvalidProtoDefinition(StaticSpan),
    ExpectedStatement(StaticSpan),
    InvalidVersionDeclaration(StaticSpan),
    InvalidProtoVersion(StaticSpan),
    DuplicateProtoVersion(StaticSpan),
    ImportMustBeNonEmpty(StaticSpan),
    ExpectedOption(StaticSpan),
    ExpectedNonempty(StaticSpan),
    ExpectedPredicateMatchButGot(StaticSpan,Rule),
    ExpectedRule(StaticSpan,Rule),
    ExpectedRuleButGot(StaticSpan,Rule, Rule),
    ExpectedOptionValue,
    UnknownOption(StaticSpan,String),
    PestRuleError(PestError<Rule>),
    ParseIntError(StaticSpan,ParseIntError),
    ExpectedButGot(StaticSpan,String, String),
}

impl From<PestError<Rule>> for ParserError {
    fn from(error: PestError<Rule>) -> Self {
        ParserError::PestRuleError(error)
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


impl ProtoParser {
    fn usize_from_str(span: Span<'_>, s: &str) -> Result<usize, ParserError> {
        str::parse::<usize>(s).map_err(|err| ParserError::ParseIntError(span.into(), err))
    }

    fn expect_rule<'a>(&mut self, pair: PestPair<'a, Rule>, rule: Rule) -> Result<PestPair<'a, Rule>, ParserError> {
        if pair.as_rule() == rule {
            return Ok(pair)
        }
        return Err(ParserError::ExpectedRuleButGot(pair.as_span().into(), rule, pair.as_rule()))
    }

    fn expect_next_rule<'a>(&mut self, span: Span<'_>, pairs: &mut PestPairs<'a, Rule>, rule: Rule) -> Result<PestPair<'a, Rule>, ParserError> {
        let value = pairs.next().ok_or(ParserError::ExpectedRule(span.into(), rule))?;
        self.expect_rule(value, rule)
    }

    fn expect_next_match<'a>(&mut self, span: Span<'_>, pairs: &mut PestPairs<'a, Rule>, predicate: impl Fn(&PestPair<'_, Rule>) -> bool) -> Result<PestPair<'a, Rule>, ParserError> {
        let value = pairs.next().ok_or_else(|| ParserError::ExpectedNonempty(span.into()))?;
        match predicate(&value) {
            true => Ok(value),
            false => Err(ParserError::ExpectedPredicateMatchButGot(value.as_span().into(), value.as_rule()))
        }
    }

    fn parse_nanopb_option(
        &mut self,
        option_statement: PestPair<'_, Rule>,
    ) -> Result<FieldOption, ParserError> {

        let option = self.expect_rule(option_statement, Rule::nanopb_option)?;
        let option_span = option.as_span();
        let mut inner = option.into_inner();

        let variant = self.expect_next_match(option_span.into(), &mut inner, |pair| {
            let rule = pair.as_rule();
            rule == Rule::max_size_option || rule == Rule::max_len_option
        })?;
        let variant_span = variant.as_span();
        let variant_rule = variant.as_rule();

        dbg!(variant_span);

        let number_value = self.expect_next_rule(option_span, &mut inner, Rule::number)?;
        
        match variant_rule {
            Rule::max_size_option => Ok(FieldOption::MaxSize(Self::usize_from_str(variant_span.into(), number_value.as_str())?)),
            Rule::max_len_option => Ok(FieldOption::MaxLen(Self::usize_from_str(variant_span.into(), number_value.as_str())?)),
            _ => return Err(ParserError::ExpectedButGot(variant_span.into(), "max_size_option or max_len_option".into(), "ERR".into())),
        }
    }

    fn parse_packed_option(
        &mut self,
        option_statement: PestPair<'_, Rule>,
    ) -> Result<FieldOption, ParserError> {
        let option = self.expect_rule(option_statement, Rule::packed_option)?;
        let option_span = option.as_span();
        let mut inner = option.into_inner();

        let packed_bool = self.expect_next_rule(option_span, &mut inner, Rule::bool)?;

        match packed_bool.as_str() {
            "true" => Ok(FieldOption::Packed(true)), 
            "false" => Ok(FieldOption::Packed(false)), 
            s => return Err(ParserError::ExpectedButGot(packed_bool.as_span().into(), "bool".into(), s.to_string())),
        }
    }

    /// parse_option parses a single option
    fn parse_option(
        &mut self,
        option_statement: PestPair<'_, Rule>,
    ) -> Result<FieldOption, ParserError> {
        let option = self.expect_rule(option_statement, Rule::option)?;
        let span = option.as_span();

        let mut option_inner = option.into_inner();

        let option_variant = self.expect_next_match(span,&mut option_inner, |pair| {
            let rule = pair.as_rule();
            rule == Rule::nanopb_option || rule == Rule::packed_option
        })?;

        match option_variant.as_rule() {
            Rule::nanopb_option => return self.parse_nanopb_option(option_variant),
            Rule::packed_option => return self.parse_packed_option(option_variant),
            _ => return Err(ParserError::ExpectedButGot(option_variant.as_span().into(), "nanopb_option or packed_option".into(), format!("{}", option_variant))),
        }
    }

    /// parse_options parses the list of options that can be specified 
    /// Example: [(nanopb).max_size=<value>,packed=true]
    fn parse_options(
        &mut self,
        options_statement: PestPair<'_, Rule>,
    ) -> Result<Vec<FieldOption>, ParserError> {
        let options = self.expect_rule(options_statement, Rule::options)?;
        options.into_inner()
                .into_iter()
                .map(|option| self.parse_option(option))
                .collect()
    }

    fn parse_message_definition(
        &mut self,
        message_statement: PestPair<'_, Rule>,
    ) -> EmptyParseResult {
        let span = message_statement.as_span();
        // dbg!(&message_statement);

        let mut inner = message_statement.into_inner();

        let identifier = inner.next().ok_or(ParserError::ExpectedButGot(span.into(),
            "identifier".to_string(),
            "None".to_string(),
        ))?;
        let identifier = Self::identifier_from_span(identifier.as_span());

        let mut message_type = MessageType {
            identifier: identifier.clone(),
            fields: BTreeMap::new(),
        };

        for value in inner {
            let value_span = value.as_span();
            match value.as_rule() {
                Rule::message_field => {
                    let mut message_inner = value.into_inner();

                    let qualifier = self.expect_next_rule(value_span.into(), &mut message_inner, Rule::qualifier)?;
                    let field_type = self.expect_next_rule(value_span.into(), &mut message_inner, Rule::field_type)?;
                    let identifier = self.expect_next_rule(value_span.into(), &mut message_inner, Rule::identifier)?;
                    let field_number = self.expect_next_rule(value_span.into(), &mut message_inner, Rule::number)?;

                    // TODO:
                    let mut options = FieldOptions {
                        max_size: None,
                        max_len: None,
                        packed: false,
                    };

                    // parse optional options
                    if let Some(next) = message_inner.next() {
                        dbg!(&next);
                        let opts = self.parse_options(next)?;
                        opts.iter().for_each(|option| {
                            match option {
                                FieldOption::MaxLen(max_len) => options.max_len = Some(*max_len),
                                FieldOption::MaxSize(max_size) => options.max_size = Some(*max_size),
                                FieldOption::Packed(value) => options.packed = *value,
                            }
                        });
                    }

                    let field_identifier = Self::identifier_from_span(identifier.as_span());
                    let field_ordinal = Self::ordinal_from_span(field_number.as_span())?;

                    let value = MessageField {
                        qualifier: FieldQualifier::from_str(qualifier.as_str(), &options),
                        field_type: FieldType::from_str(field_type.as_str(), options.max_size),
                        identifier: field_identifier,
                        ordinal: Field(field_ordinal),
                    };
                    message_type.fields.insert(field_ordinal, value);
                }
                _ => unreachable!(),
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
        identifier_str[1..identifier_str.len() - 1].to_string()
    }

    fn ordinal_from_span<'i>(span: Span<'i>) -> Result<u32, ParserError> {
        let ordinal_str = span.as_str();
        let parse_result = ordinal_str.parse::<u32>();
        parse_result.map_err(|err| ParserError::ParseIntError(span.into(), err))
    }

    fn parse_enum_definition(&mut self, enum_statement: PestPair<'_, Rule>) -> EmptyParseResult {
        let span = enum_statement.as_span();

        let mut inner = enum_statement.into_inner();

        let identifier = inner.next().ok_or(ParserError::ExpectedButGot(
            span.into(),
            "identifier".to_string(),
            "None".to_string(),
        ))?;
        let identifier = Self::identifier_from_span(identifier.as_span());

        let mut enum_type = EnumType {
            identifier: identifier.clone(),
            pairs: BTreeMap::new(),
        };

        for value in inner {
            let value_span = value.as_span();
            match value.as_rule() {
                Rule::enum_field => {
                    let mut enum_inner = value.into_inner();

                    let identifier = self.expect_next_rule(value_span.into(), &mut enum_inner, Rule::identifier)?;
                    let number = self.expect_next_rule(value_span.into(), &mut enum_inner, Rule::number)?;

                    let field_identifier = Self::identifier_from_span(identifier.as_span());
                    let field_ordinal = Self::ordinal_from_span(number.as_span())?;

                    enum_type.pairs.insert(field_identifier, field_ordinal);
                }
                _ => unreachable!(),
            }
        }
        self.enum_types.insert(identifier, enum_type);
        Ok(())
    }

    fn parse_block_statement(&mut self, statement: PestPair<'_, Rule>) -> EmptyParseResult {
        let block_statement = self.expect_rule(statement, Rule::block_statement)?;
        let span = block_statement.as_span();
        let mut inner = block_statement.into_inner();

        let next = self.expect_next_match(span, &mut inner, |pair| {
            let rule = pair.as_rule();
            rule == Rule::message_definition || rule == Rule::enum_definition
        })?;

        // dbg!(&statement);
        match next.as_rule() {
            Rule::message_definition => self.parse_message_definition(next),
            Rule::enum_definition => self.parse_enum_definition(next),
            _ => Err(ParserError::ExpectedButGot(
                span.into(),
                "message or enum definition".to_string(),
                next.as_str().to_string(),
            )),
        }
    }

    fn parse_import_statement(&mut self, statement: PestPair<'_, Rule>) -> EmptyParseResult {
        let span = statement.as_span();
        if let Some(value) = statement.into_inner().next() {
            let slice = value.as_span().as_str();
            if slice.len() == 2 {
                return Err(ParserError::ImportMustBeNonEmpty(span.into()));
            }
            self.imports.push(slice[1..slice.len() - 1].to_owned())

            // TODO: open files, and parse them here
        }
        Ok(())
    }

    fn parse_version_decl(&mut self, statement: PestPair<'_, Rule>) -> EmptyParseResult {
        let span = statement.as_span();
        if let Some(value) = statement.into_inner().next() {
            // TODO:
            if Self::string_from_span(value.as_span()) == "proto2" {
                self.version = Version::Proto2;
                return Ok(());
            } else {
                return Err(ParserError::InvalidProtoVersion(span.into()));
            }
        }
        Err(ParserError::InvalidVersionDeclaration(span.into()))
    }

    fn parse_statement(&mut self, statement: PestPair<'_, Rule>) -> EmptyParseResult {
        let span = statement.as_span();
        let statement = self.expect_rule(statement, Rule::statement)?;
        let mut statement_inner = statement.into_inner();

        let statement_variant = self.expect_next_match(span, &mut statement_inner, |pair| {
            let rule = pair.as_rule();
            rule == Rule::block_statement || rule == Rule::import_statement
        })?;

        match statement_variant.as_rule() {
            Rule::block_statement => self.parse_block_statement(statement_variant),
            Rule::import_statement => self.parse_import_statement(statement_variant),
            _ => Err(ParserError::ExpectedButGot(
                span.into(),
                "block statement".to_string(),
                statement_variant.as_node_tag().unwrap_or("<unknown>").to_string(),
            )),
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
        message_types: HashMap::new(),
    };

    // Do a single pass and extract enum and message types
    for pair in parse.into_iter() {
        let pair_span = pair.as_span();
        match pair.as_rule() {
            Rule::proto_definition => {
                for p in pair.into_inner() {
                    match p.as_rule() {
                        Rule::statement => output.parse_statement(p)?,
                        Rule::version_decl => output.parse_version_decl(p)?,
                        Rule::EOI => break,
                        _ => {
                            return Err(ParserError::ExpectedButGot(
                                pair_span.into(),
                                "Statement or version decl".to_string(),
                                p.as_str().to_string(),
                            ))
                        }
                    };
                }
            }
            _ => unreachable!(),
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
                FieldType::MessageType(identifier, _)
                | FieldType::UnboundedMessageType(identifier) => {
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
