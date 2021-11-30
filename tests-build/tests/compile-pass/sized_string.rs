use serde_derive::{Deserialize, Serialize};
use sewup_derive::{SizedString, Table};

#[derive(Table, Serialize, Deserialize)]
struct Blog {
    content: SizedString!(50),
}

fn main() {}
