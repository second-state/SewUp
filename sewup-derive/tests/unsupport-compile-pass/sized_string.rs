// This test case need following issue support
// https://github.com/dtolnay/trybuild/issues/108
use sewup_derive::{SizedString, Table};

#[derive(Table)]
struct Blog {
    content: SizedString!(50),
}

fn main() {}
