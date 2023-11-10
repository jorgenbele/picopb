use pest::{iterators::Pairs, Parser};
use picopb::{
    generator::generate,
    parser::{parse, PicoPBParser, Rule},
    validator::validate,
};

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
