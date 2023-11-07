/// This file contains the proto definition parser for proto v2
/// It is implemented using pest
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser.pest"] // relative to src
pub struct PicoPBParser;
