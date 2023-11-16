use picopb::{generator::generate, parser::parse, validator::validate};

fn main() {
    let result = parse(
        "
        syntax = \"proto2\";
        message Query {
            required bytes key = 1; [(nanopb).max_size=128]
            required bytes value = 2; [(nanopb).max_size=64]
        }
    ",
    );
    let result = result.unwrap();
    dbg!(&result);
    validate(&result).unwrap();
    generate(&result).unwrap();
}
