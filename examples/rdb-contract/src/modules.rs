use serde_derive::{Deserialize, Serialize};
use sewup_derive::Table;

// Table derive provides the handers for CRUD,
// to communicate with these handler, you will need protocol.
// The protocol is easy to build by the `{struct_name}::protocol`, `{struct_name}::Protocol`,
// please check out the test case in the end of this document
#[derive(Table, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Person {
    pub trusted: bool,
    pub age: u8,
}

#[derive(Table, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[belongs_to(Person)]
pub struct Post {
    pub words: u8,

    // Currently, this field need to set up manually, this will be enhance later
    pub person_id: usize,
}
