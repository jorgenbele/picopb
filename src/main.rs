use clap::Parser;
use std::fs::read_to_string;
// use pest::Parser;
use picopb::{
    generator::generate,
    parser::parse,
    validator::validate,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    generate: bool,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    #[arg(long, default_value_t = true)]
    validate: bool,

    proto_file: String,
}

fn main() {
    let args = Args::parse();

    let input = read_to_string(args.proto_file).expect("failed to read file");
    let result = parse(&input).expect("failed to parse input");
    if args.verbose {
        dbg!(&result);
    }
    if args.validate {
        validate(&result).expect("failed to validate input");
    }
    if args.generate {
        generate(&result).expect("failed to generate");
    }

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

    // let result = parse(
    //     "
    //     syntax = \"proto2\";

    //     message Query {
    //         required bytes key = 1; [(nanopb).max_size=8]
    //         required bytes opaque = 2; [(nanopb).max_size=8]
    //     }

    //     enum Error {
    //         ERROR_INVALID_KEY = 1;
    //         ERROR_NOT_FOUND = 2;
    //     }
    // ",
    // );
    // let result = result.unwrap();
    // dbg!(&result);
    // validate(&result).unwrap();
    // generate(&result).unwrap();
}

#[cfg(test)]
mod test {
    // let p = PicoPBParser::parse(Rule::message_field, "required string name = 1")
}
