use serde_derive::Serialize;
use sewup_derive::ewasm_output_from;

#[derive(Default, Serialize)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

fn main() {
    let simple_struct = SimpleStruct::default();
    ewasm_output_from!(true);
    ewasm_output_from!(1);
    ewasm_output_from!(vec![1u8, 2u8, 3u8]);
    ewasm_output_from!("jovy".to_string());
    ewasm_output_from!(simple_struct);
    ewasm_output_from!(&simple_struct);
    ewasm_output_from!([1u8, 2u8, 3u8]);
}
