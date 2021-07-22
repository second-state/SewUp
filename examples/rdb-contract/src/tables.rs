use serde_derive::{Deserialize, Serialize};

use sewup_derive::Table;

// Table derive provides the handers for CRUD,
// to communicate with these handler, you will need protocol.
// The relating utilities will be set into `mod {struct_name}` by macro
// The protocol is easy to build by the `{struct_name}::protocol`, `{struct_name}::Protocol`,
// please check out the test case in the end of this document
#[derive(Table, Default, Clone, Serialize, Deserialize)]
pub struct Person {
    trusted: bool,
    age: u8,
}
