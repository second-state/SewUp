use sewup_derive::ewasm_fn;

#[ewasm_fn]
fn handler() -> Result<(), ()> {}

#[ewasm_fn]
fn handler_with_input(x: usize, y: String) -> Result<(), ()> {}

pub mod a {
    pub type MySize = usize;
}

#[ewasm_fn]
fn handler_with_mod_input(x: usize, y: a::MySize) -> Result<(), ()> {}

pub mod a_super_super_long_module_name {
    pub type MySize = usize;
}

#[ewasm_fn]
fn handler_with_long_mod_input(
    x: usize,
    y: Vec<a_super_super_long_module_name::MySize>,
) -> Result<(), ()> {
}

fn main() {
    let _sig = HANDLER_SIG;
    let _sig_2 = HANDLER_WITH_INPUT_SIG;
    let _sig_3 = HANDLER_WITH_MOD_INPUT_SIG;
    let _sig_4 = HANDLER_WITH_LONG_MOD_INPUT_SIG;
}
