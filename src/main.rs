use pest::Parser;
use picopb::{
    generator::generate,
    parser::{parse, PicoPBParser, Rule},
    validator::validate,
};

const PROTO_DEF: &str = "syntax = \"proto2\"; \
                         message A{int32 int_field = 1;}";

fn main() {
    // let example_field = "required string name = 2;";
    // PicoPBParser::parse(Rule::message_field, example_field).unwrap();

    // let message = "message A { required string name = 2; }";
    // PicoPBParser::parse(Rule::message_definition, message).unwrap();

    // let message_several = "message A {
    // required string name = 2;
    // optional string name = 2;
    // }";
    // PicoPBParser::parse(Rule::message_definition, message_several).unwrap();

    // //
    // let example_proto = "syntax = \"proto2\";
    //  message A {
    //      required string name = 1;
    //      optional string password = 123;
    //      repeated int32 integers = 2;
    //  }";
    // println!("{}", example_proto);
    // PicoPBParser::parse(Rule::proto_definition, example_proto).unwrap();

    // let example_proto2 = "
    // syntax = \"proto2\";
    // import \"common.proto\";
    //  message Responses {
    //      required string name = 1;
    //      required bool ok = 2;
    //      optional Error error = 2;
    //  }
    //  enum Error {
    //      ERROR_INVALID_PASSWORD = 1;
    //      ERROR_INVALID_USER = 2;
    // }";

    // println!("{}", example_proto2);
    // PicoPBParser::parse(Rule::proto_definition, example_proto2).unwrap();
        // message Response {
        //     required bytes value = 1; [(nanopb).max_size=64]
        //     required bytes opaque = 2;
        //     optional Error error = 3;
        // }

        // message RepeatedResponse {
        //     repeated Response responses = 1; [(nanopb).max_size=64]
        // }

        // message PackedIntegers {
        //     repeated int32 integers = 1; [packed=true, (nanopb).max_size=64]
        // }

        // import \"common.proto\";
        // import \"shared.proto\";

    let result = parse(
        "
        syntax = \"proto2\";

        message Query {
            required bytes key = 1; [(nanopb).max_size=8]
            required bytes opaque = 2; [(nanopb).max_size=8]
        }

        enum Error {
            ERROR_INVALID_KEY = 1;
            ERROR_NOT_FOUND = 2;
        }
    ",
    );
    let result = result.unwrap();
    dbg!(&result);
    validate(&result).unwrap();
    generate(&result).unwrap();

}

#[cfg(test)]
mod test {
    // let p = PicoPBParser::parse(Rule::message_field, "required string name = 1")
}
