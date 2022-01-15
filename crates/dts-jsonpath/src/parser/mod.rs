#![allow(unused_imports)]

mod ast;

use crate::Result;
use pest::Parser as ParserTrait;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar/jsonpath.pest"]
struct JsonPathParser;
