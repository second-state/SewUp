use serde_derive::{Deserialize, Serialize};
use sewup::SingleBucket;
use sewup_derive::{ewasm_constructor, ewasm_main, ewasm_test, Value};

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Value)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

#[ewasm_constructor]
fn setup() {
    use sewup::{
        single_bucket::SingleBucket2,
        types::{Raw, Row},
    };
    // (Raw, Row), (Row, SimpleStruct)
    let bucket = SingleBucket2::<Raw, Row, Row, SimpleStruct>::default();
    bucket
        .commit()
        .expect("there is no return for constructor currently");
}

#[ewasm_main(auto)]
fn main() -> anyhow::Result<sewup::primitives::EwasmAny> {
    use sewup_derive::{ewasm_fn_sig, ewasm_input_from};

    let contract = sewup::primitives::Contract::new()?;

    let output = match contract.get_function_selector()? {
        _ => return panic!("unhandled"),
    };

    Ok(output)
}

#[ewasm_test]
mod tests {
    use super::*;
    #[ewasm_test]
    fn test_() {
        assert!(true)
    }
}
