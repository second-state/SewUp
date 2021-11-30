use serde_derive::{Deserialize, Serialize};
use sewup_derive::Table;

#[derive(Table, Serialize, Deserialize)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

#[derive(Table, Serialize, Deserialize)]
pub struct AnotherSimpleStruct {
    trust: bool,
    description: String,
}

fn main() {}
