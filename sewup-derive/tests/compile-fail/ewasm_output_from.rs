use sewup_derive::ewasm_output_from;

#[derive(Default)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

fn main() {
    let simple_struct = SimpleStruct::default();
    ewasm_output_from!(simple_struct);
    ewasm_output_from!(&simple_struct);
    ewasm_output_from!(|_| "Str");
}
