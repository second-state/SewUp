// This test case need following issue support
// https://github.com/dtolnay/trybuild/issues/108
use sewup_derive::Table;

#[derive(Table)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

#[derive(Table)]
pub struct AnotherSimpleStruct {
    trust: bool,
    description: String,
}

fn main() {}
