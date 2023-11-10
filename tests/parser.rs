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
fn encode_syntax_proto2() {
    let proto_def: &str = "syntax = \"proto2\";";
    let result = PicoPBParser::parse(Rule::proto_definition, proto_def).unwrap();
    assert_eq_parse_result(result, "[Pair { rule: proto_definition, span: Span { str: \"syntax = \\\"proto2\\\";\", start: 0, end: 18 }, inner: [Pair { rule: version_decl, span: Span { str: \"syntax = \\\"proto2\\\";\", start: 0, end: 18 }, inner: [Pair { rule: string, span: Span { str: \"\\\"proto2\\\"\", start: 9, end: 17 }, inner: [] }] }, Pair { rule: EOI, span: Span { str: \"\", start: 18, end: 18 }, inner: [] }] }]")
}

#[test]
fn encode_complex_proto2() {
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
