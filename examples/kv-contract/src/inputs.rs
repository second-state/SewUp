use serde_derive::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Pair(pub u32, pub Vec<u8>);
