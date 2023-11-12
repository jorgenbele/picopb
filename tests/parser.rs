use pest::{iterators::Pairs, Parser};
use picopb::{
    parser::{PicoPBParser, Rule},
    validator::validate,
};

use picopb::parser::parse;

fn assert_eq_parse_result(result: Pairs<'_, Rule>, expected: &str) {
    let output = format!("{:?}", result);
    assert_eq!(output, expected)
}

#[test]
fn parse_syntax_proto2() {
    let proto_def: &str = "syntax = \"proto2\";";
    let result = PicoPBParser::parse(Rule::proto_definition, proto_def).unwrap();
    assert_eq_parse_result(result, "[Pair { rule: proto_definition, span: Span { str: \"syntax = \\\"proto2\\\";\", start: 0, end: 18 }, inner: [Pair { rule: version_decl, span: Span { str: \"syntax = \\\"proto2\\\";\", start: 0, end: 18 }, inner: [Pair { rule: string, span: Span { str: \"\\\"proto2\\\"\", start: 9, end: 17 }, inner: [] }] }, Pair { rule: EOI, span: Span { str: \"\", start: 18, end: 18 }, inner: [] }] }]")
}

#[test]
fn parse_bytes_field_max_size() {
    let bytes_with_max_size = "required bytes value = 1; [(nanopb).max_size=64]";
    let result = PicoPBParser::parse(Rule::message_field, bytes_with_max_size).unwrap();
    let expected = "[Pair { rule: message_field, span: Span { str: \"required bytes value = 1; [(nanopb).max_size=64]\", start: 0, end: 48 }, inner: [Pair { rule: qualifier, span: Span { str: \"required\", start: 0, end: 8 }, inner: [] }, Pair { rule: field_type, span: Span { str: \"bytes\", start: 9, end: 14 }, inner: [] }, Pair { rule: identifier, span: Span { str: \"value\", start: 15, end: 20 }, inner: [] }, Pair { rule: number, span: Span { str: \"1\", start: 23, end: 24 }, inner: [] }, Pair { rule: options, span: Span { str: \"[(nanopb).max_size=64]\", start: 26, end: 48 }, inner: [Pair { rule: option, span: Span { str: \"(nanopb).max_size=64\", start: 27, end: 47 }, inner: [Pair { rule: nanopb_option, span: Span { str: \"(nanopb).max_size=64\", start: 27, end: 47 }, inner: [Pair { rule: max_size_option, span: Span { str: \"max_size\", start: 36, end: 44 }, inner: [] }, Pair { rule: number, span: Span { str: \"64\", start: 45, end: 47 }, inner: [] }] }] }] }] }]";
    assert_eq_parse_result(result, expected)
}

#[test]
fn parse_complex_proto2() {
    let proto_def: &str = "
        syntax = \"proto2\";
        import \"common.proto\";
        import \"shared.proto\";

        message Query {
            required bytes key = 1; [(nanopb).max_size=128]
            required bytes opaque = 2; [(nanopb).max_size=64]
        }

        message Response {
            required bytes value = 1; [(nanopb).max_size=64]
            required bytes opaque = 2;
            optional Error error = 3;
        }

        message RepeatedResponse {
            repeated Response responses = 1; [(nanopb).max_size=64]
        }

        enum Error {
            ERROR_INVALID_KEY = 1;
            ERROR_NOT_FOUND = 2;
        }
";
    let result = parse(proto_def).unwrap();
    validate(&result).unwrap();
}
