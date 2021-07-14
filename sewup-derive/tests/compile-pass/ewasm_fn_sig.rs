use sewup_derive::ewasm_fn_sig;

const SOME_HANDLE_SIG: [u8; 4] = [0, 0, 0, 0];

mod other {
    pub type MyUsize = usize;
    pub type MyString = String;
}

fn main() {
    let _a = ewasm_fn_sig!(some_handle);
    let _b = ewasm_fn_sig!(other_handle());
    let _c = ewasm_fn_sig!(other_handle2(x: usize, y: String));
    let _d = ewasm_fn_sig!(other::mod::handle(x: usize, y: String));
    let _e = ewasm_fn_sig!(other::mod::handle(x: other::MyUsize, y: other::MyString));
    let _f = ewasm_fn_sig!(other_handle2(usize, String));
}
