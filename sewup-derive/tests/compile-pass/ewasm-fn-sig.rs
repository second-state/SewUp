use sewup_derive::ewasm_fn_sig;

const SOME_HANDLE_SIG: [u8; 4] = [0, 0, 0, 0];

fn main() {
    let _a = ewasm_fn_sig!(some_handle);
    let _b = ewasm_fn_sig!(other_handle());
}
