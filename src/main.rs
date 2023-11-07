use pest::Parser;
use picopb::parser::{PicoPBParser, Rule};

const PROTO_DEF: &str = "syntax = \"proto2\"; \
                         message A{int32 int_field = 1;}";

fn main() {
    //let p = PicoPBParser::parse(Rule::proto_definition, PROTO_DEF)
    //    .expect("expected to parse the proto def");

    // dbg!(PicoPBParser::parse(Rule::message_field, "required string name = 1")
    //     .expect("expected to parse the proto def"));
    // dbg!(PicoPBParser::parse(
    //     Rule::message_definition,
    //     "message A {required string name = 1; }"
    // )
    // .expect("expected to parse the proto def"));
    // dbg!(PicoPBParser::parse(Rule::COMMENT, "// this is a comment").expect("comment did not parse"));

    //
    let example_field = "required string name = 2;";
    dbg!(PicoPBParser::parse(Rule::message_field, example_field).unwrap());

    let message = "message A { required string name = 2; }";
    dbg!(PicoPBParser::parse(Rule::message_definition, message).unwrap());

    let message_several = "message A {
    required string name = 2;
    optional string name = 2;
}";
    dbg!(PicoPBParser::parse(Rule::message_definition, message_several).unwrap());

    //
    let example_proto = "syntax = \"proto2\";
     message A {
         required string name = 1;
         optional string password = 123;
         repeated int32 integers = 2;
     }";
    println!("{}", example_proto);
    dbg!(PicoPBParser::parse(Rule::proto_definition, example_proto,).unwrap());

    let example_proto2 = "
     message Responses {
         required string name = 1;
         required bool ok = 2;
         optional Error error = 2;
     }
enum Error {
    ERROR_INVALID_PASSWORD = 1;
    ERROR_INVALID_USER = 2;

}";

    println!("{}", example_proto2);
    dbg!(PicoPBParser::parse(Rule::proto_definition, example_proto2,).unwrap());
}

#[cfg(test)]
mod test {
    // let p = PicoPBParser::parse(Rule::message_field, "required string name = 1")
}
