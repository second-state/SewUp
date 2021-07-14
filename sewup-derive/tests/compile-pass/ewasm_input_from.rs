use sewup::primitives::Contract;
use sewup_derive::ewasm_input_from;

fn original_handler(_a: u8) -> Result<(), &'static str> {
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let contract = Contract::mock();
    let _new_handler = ewasm_input_from!(contract move original_handler);
    Ok(())
}

fn rust_main() -> Result<(), &'static str> {
    let contract = Contract::mock();
    let _new_handler_with_err_handle =
        ewasm_input_from!(contract move original_handler, |_| "ErrorStr");
    Ok(())
}
