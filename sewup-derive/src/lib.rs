extern crate proc_macro;

use proc_macro::TokenStream;
use regex::Regex;
use tiny_keccak::{Hasher, Keccak};

fn get_function_signature(function_prototype: &str) -> [u8; 4] {
    let mut sig = [0; 4];
    let mut hasher = Keccak::v256();
    hasher.update(function_prototype.as_bytes());
    hasher.finalize(&mut sig);
    sig
}

/// helps you setup the main function of a contract
///
/// There are three different kind contract output.
///
/// `#[ewasm_main]`
/// The default contract output, the error will be return as a string message
/// This is for a scenario that you just want to modify the data on
/// chain only, and the error will to string than return.
///
/// `#[ewasm_main(rusty)]`
/// The rust styl output, the result object from ewasm_main function will be
/// returned, this is for a scenario that you are using a rust client to catch
/// and want to catch the result from the contract.
///
/// `#[ewasm_main(unwrap)]`
/// The unwrap the output of the result object from ewasm_main function.
/// This is for a scenario that you are using a rust non-rust client,
/// and you are only care the happy case of excuting the contract.
///
/// ```compile_fail
/// #[ewasm_main]
/// fn main() -> Result<()> {
///     let contract = Contract::new()?;
///     match contract.get_function_selector()? {
///         ewasm_fn_sig!(check_input_object) => ewasm_input_from!(contract, check_input_object)?,
///         _ => return Err(Error::UnknownHandle.into()),
///     };
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn ewasm_main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let re = Regex::new(r"fn (?P<name>[^(]+?)\(").unwrap();
    let fn_name = if let Some(cap) = re.captures(&item.to_string()) {
        cap.name("name").unwrap().as_str().to_owned()
    } else {
        panic!("parse function error")
    };

    return match attr.to_string().to_lowercase().as_str() {
        // Return the inner structure from unwrap result
        // This is for a scenario that you take care the result but not using Rust client
        "unwrap" => format!(
            r#"
            #[cfg(target_arch = "wasm32")]
            use sewup::bincode;
            #[cfg(target_arch = "wasm32")]
            use ewasm_api::finish_data;
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub fn main() {{
                {}
                match {}() {{
                    Ok(r) =>  {{
                        let bin = bincode::serialize(&r).expect("The resuslt of `ewasm_main` should be serializable");
                        finish_data(&bin);

                    }},
                    Err(e) => {{
                        let error_msg = e.to_string();
                        finish_data(&error_msg.as_bytes());

                    }}
                }}
            }}
        "#,
            item.to_string(),
            fn_name
        )
        .parse()
        .unwrap(),

        // Return all result structure
        // This is for a scenario that you are using a rust client to operation the contract
        "rusty" => format!(
            r#"
            #[cfg(target_arch = "wasm32")]
            use sewup::bincode;
            #[cfg(target_arch = "wasm32")]
            use ewasm_api::finish_data;
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub fn main() {{
                {}
                let r = {}();
                let bin = bincode::serialize(&r).expect("The resuslt of `ewasm_main` should be serializable");
                finish_data(&bin);
            }}
        "#,
            item.to_string(),
            fn_name
        )
        .parse()
        .unwrap(),

        // Default only return error message,
        // This is for a scenario that you just want to modify the data on
        // chain only
        _ => format!(
            r#"
            use sewup::bincode;
            #[cfg(target_arch = "wasm32")]
            use ewasm_api::finish_data;
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub fn main() {{
                {}
                if let Err(e) = {}() {{
                    let error_msg = e.to_string();
                    finish_data(&error_msg.as_bytes());
                }}
            }}
        "#,
            item.to_string(),
            fn_name
        )
        .parse()
        .unwrap()
    };
}

/// helps you to build your handlers in the contract
///
/// This macro also generate the function signature, you can use
/// `ewasm_fn_sig!` macro to get your function signature;
///
/// ```compile_fail
/// #[ewasm_main]
/// fn main() -> Result<()> {
///     let contract = Contract::new()?;
///     match contract.get_function_selector()? {
///         ewasm_fn_sig!(check_input_object) => ewasm_input_from!(contract, check_input_object)?,
///         _ => return Err(Error::UnknownHandle.into()),
///     };
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn ewasm_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let re = Regex::new(r"^fn (?P<name>[^(]+?)\((?P<params>[^)]*?)\)").unwrap();
    if let Some(cap) = re.captures(&item.to_string()) {
        let fn_name = cap.name("name").unwrap().as_str();
        let params = cap.name("params").unwrap().as_str().replace(" ", "");
        let canonical_fn = format!(
            "{}({})",
            fn_name,
            params
                .split(',')
                .map(|p| p.split(':').nth(1).unwrap_or("").trim())
                .collect::<Vec<_>>()
                .join(",")
        );
        format!(
            r#"
            pub(crate) const {}_SIG: [u8; 4] = {:?};
            #[cfg(target_arch = "wasm32")]
            {}
        "#,
            fn_name.to_ascii_uppercase(),
            get_function_signature(&canonical_fn),
            item.to_string()
        )
        .parse()
        .unwrap()
    } else {
        panic!("parsing ewsm function fails: {}", item.to_string());
    }
}

/// helps you to build your handler in other module
///
/// This macro will automatically generated as `{FUNCTION_NAME}_SIG`
///
/// ```compile_fail
/// // module.rs
///
/// use ss_ewasm_api as ewasm_api;
///
/// #[ewasm_lib_fn]
/// pub fn symbol(s: &str) {
///     let symbol = s.to_string().into_bytes();
///     ewasm_api::finish_data(&symbol);
/// }
/// ```
///
/// ```compile_fail
/// // lib.rs
///
/// use module::{symbol, SYMBOL_SIG};
///
/// #[ewasm_main]
/// fn main() -> Result<()> {
///     let contract = Contract::new()?;
///     match contract.get_function_selector()? {
///         SYMBOL_SIG => symbol("ETD"),
///         _ => return Err(Error::UnknownHandle.into()),
///     };
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn ewasm_lib_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let re = Regex::new(r"^pub fn (?P<name>[^(]+?)\((?P<params>[^)]*?)\)").unwrap();
    if let Some(cap) = re.captures(&item.to_string()) {
        let fn_name = cap.name("name").unwrap().as_str();
        let params = cap.name("params").unwrap().as_str().replace(" ", "");
        let canonical_fn = format!(
            "{}({})",
            fn_name,
            params
                .split(',')
                .map(|p| p.split(':').nth(1).unwrap_or("").trim())
                .collect::<Vec<_>>()
                .join(",")
        );
        format!(
            r#"
            /// The siganature for fn {}
            pub const {}_SIG: [u8; 4] = {:?};

            #[cfg(not(target_arch = "wasm32"))]
            pub fn {}({}) {{}}

            #[cfg(target_arch = "wasm32")]
            {}
        "#,
            fn_name,
            fn_name.to_ascii_uppercase(),
            get_function_signature(&canonical_fn),
            fn_name,
            params,
            item.to_string()
        )
        .parse()
        .unwrap()
    } else {
        panic!("parsing ewsm function fails: {}", item.to_string());
    }
}

/// helps you get you function signature
///
/// 1. provide function name to get function signature from the same namespace,
/// which function should be decorated with `#[ewasm_fn]`, for example,
/// `ewasm_fn_sig!(contract_handler)`
///
/// ```compile_fail
/// #[ewasm_fn]
/// fn decorated_handler(a: i32, b: String) -> Result<()> {
///     Ok(())
/// }
///
/// #[ewasm_main]
/// fn main() -> Result<()> {
///     let contract = Contract::new()?;
///     match contract.get_function_selector()? {
///         ewasm_fn_sig!(decorated_handler) => ewasm_input_from!(contract, decorated_handler)?,
///         _ => return Err(Error::UnknownHandle.into()),
///     };
///     Ok(())
/// }
/// ```
///
/// 2. provide a function name with input parameters then the macro will
/// calculate the correct functional signature for you.
/// ex: `ewasm_fn_sig!(undecorated_handler( a: i32, b: String ))`
///
/// ```compile_fail
/// // some_crate.rs
/// pub fn decorated_handler(a: i32, b: String) -> Result<()> {
///     Ok(())
/// }
/// ```
///
/// ```compile_fail
/// use some_crate::decorated_handler;
///
/// #[ewasm_main]
/// fn main() -> Result<()> {
///     let contract = Contract::new()?;
///     match contract.get_function_selector()? {
///         ewasm_fn_sig!(undecorated_handler(a: i32, b: String))
///             => ewasm_input_from!(contract, undecorated_handler)?,
///         _ => return Err(Error::UnknownHandle.into()),
///     };
///     Ok(())
/// }
/// ```
///
#[proc_macro]
pub fn ewasm_fn_sig(item: TokenStream) -> TokenStream {
    let re = Regex::new(r"^(?P<name>[^(]+?)\((?P<params>[^)]*?)\)").unwrap();
    if let Some(cap) = re.captures(&item.to_string()) {
        let fn_name = cap.name("name").unwrap().as_str();
        let params = cap.name("params").unwrap().as_str().replace(" ", "");
        let canonical_fn = format!(
            "{}({})",
            fn_name,
            params
                .split(',')
                .map(|p| p.split(':').nth(1).unwrap_or("").trim())
                .collect::<Vec<_>>()
                .join(",")
        );
        format!(r"{:?}", get_function_signature(&canonical_fn))
            .parse()
            .unwrap()
    } else {
        format!("{}_SIG", item.to_string().to_ascii_uppercase())
            .parse()
            .unwrap()
    }
}

/// helps you to get the input data from contract caller
///
/// This macro automatically deserialize input into handler
/// `ewasm_input_from!(contract, the_name_of_the_handler)`
/// ```compile_fail
/// #[ewasm_main]
/// fn main() -> Result<()> {
///     let contract = Contract::new()?;
///     match contract.get_function_selector()? {
///         ewasm_fn_sig!(check_input_object) => ewasm_input_from!(contract, check_input_object)?,
///         _ => return Err(Error::UnknownHandle.into()),
///     };
///  Ok(())
///  }
/// ```
///
/// Besides, you can map the error to your customized error when something wrong happened in
/// `ewasm_input_from!`, for example:
/// `ewasm_input_from!(contract, check_input_object, |_| Err("DeserdeError"))`
/// ```compile_fail
/// #[ewasm_main(rusty)]
/// fn main() -> Result<(), &'static str> {
///     let contract = Contract::new().map_err(|_| "NewContractError")?;
///     match contract.get_function_selector().map_err(|_| "FailGetFnSelector")? {
///         ewasm_fn_sig!(check_input_object) =>  ewasm_input_from!(contract, check_input_object, |_| "DeserdeError")?
///         _ => return Err("UnknownHandle"),
///     };
///     Ok(())
/// }
/// ```
#[proc_macro]
pub fn ewasm_input_from(item: TokenStream) -> TokenStream {
    let re = Regex::new(r"^(?P<contract>\w+),\s+(?P<name>\w+),?(?P<error_handler>.*)").unwrap();
    if let Some(cap) = re.captures(&item.to_string()) {
        let contract = cap.name("contract").unwrap().as_str();
        let fn_name = cap.name("name").unwrap().as_str();
        let error_handler = cap.name("error_handler").unwrap().as_str();
        if error_handler.is_empty() {
            format!(
                r#"
                    {}(bincode::deserialize(&{}.input_data[4..])?)
                "#,
                fn_name, contract
            )
            .parse()
            .unwrap()
        } else {
            format!(
                r#"
                    {}(bincode::deserialize(&{}.input_data[4..]).map_err({})?)
                "#,
                fn_name, contract, error_handler
            )
            .parse()
            .unwrap()
        }
    } else {
        panic!("fail to parsing function in fn_select");
    }
}

/// `Key` derive help you implement Key trait for the kv feature
///
/// ```
/// use sewup_derive::Key;
/// #[derive(Key)]
/// struct SimpleStruct {
///     trust: bool,
///     description: String,
/// }
/// ```
#[proc_macro_derive(Key)]
pub fn derive_key(item: TokenStream) -> TokenStream {
    let re = Regex::new(r"struct (?P<name>\w+)").unwrap();
    if let Some(cap) = re.captures(&item.to_string()) {
        let struct_name = cap.name("name").unwrap().as_str();
        format!(
            r#"
            #[cfg(target_arch = "wasm32")]
            impl sewup::kv::traits::Key for {} {{}}
        "#,
            struct_name,
        )
        .parse()
        .unwrap()
    } else {
        panic!("sewup-derive parsing struct fails: {}", item.to_string());
    }
}

/// `Value` derive help you implement Value trait for kv feature
///
/// ```
/// use sewup_derive::Value;
/// #[derive(Value)]
/// struct SimpleStruct {
///     trust: bool,
///     description: String,
/// }
/// ```
#[proc_macro_derive(Value)]
pub fn derive_value(item: TokenStream) -> TokenStream {
    let re = Regex::new(r"struct (?P<name>\w+)").unwrap();
    if let Some(cap) = re.captures(&item.to_string()) {
        let struct_name = cap.name("name").unwrap().as_str();
        format!(
            r#"
            #[cfg(target_arch = "wasm32")]
            impl sewup::kv::traits::Value for {} {{}}
        "#,
            struct_name,
        )
        .parse()
        .unwrap()
    } else {
        panic!("sewup-derive parsing struct fails: {}", item.to_string());
    }
}

/// helps you setup the test mododule, and test cases in contract.
///
/// ```compile_fail
/// #[ewasm_test]
/// mod tests {
///     use super::*;
///
///     #[ewasm_test]
///     fn test_execute_basic_operations() {
///         ewasm_assert_ok!(contract_fn());
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn ewasm_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mod_re = Regex::new(r"mod (?P<mod_name>[^\{]*)\{").unwrap();
    let fn_re = Regex::new(r"fn (?P<fn_name>[^\(]*)\(").unwrap();

    if let Some(cap) = mod_re.captures(&item.to_string()) {
        let mod_name = cap.name("mod_name").unwrap().as_str().to_owned();
        let prefix = format!("mod {}{{", mod_name);
        return format!(
        r#"
            #[cfg(test)]
            mod {} {{
                use sewup::bincode;
                use sewup::runtimes::{{handler::ContractHandler, test::TestRuntime}};
                use sewup_derive::*;
                use std::cell::RefCell;
                use std::path::Path;
                use std::path::PathBuf;
                use std::process::Command;
                use std::sync::Arc;

                fn _build_wasm() -> String {{
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg("cargo build --release --target=wasm32-unknown-unknown")
                        .output()
                        .expect("failed to build wasm binary");
                    if !dbg!(output).status.success() {{
                        panic!("return code not success: fail to build wasm binary")
                    }}
                    let pkg_name = env!("CARGO_PKG_NAME");
                    let base_dir = env!("CARGO_MANIFEST_DIR");
                    let wasm_binary = format!(
                        "{{}}/target/wasm32-unknown-unknown/release/{{}}.wasm",
                        base_dir,
                        pkg_name.replace("-", "_")
                    );

                    if !Path::new(&wasm_binary).exists() {{
                        panic!("wasm binary missing")
                    }}
                    wasm_binary
                }}

                fn _build_runtime_and_runner() -> (
                    Arc<RefCell<TestRuntime>>,
                    impl Fn(Arc<RefCell<TestRuntime>>, &str, [u8; 4], Option<&[u8]>, Vec<u8>) -> (),
                ) {{
                    (
                        Arc::new(RefCell::new(TestRuntime::default())),
                        |runtime: Arc<RefCell<TestRuntime>>,
                         fn_name: &str,
                         sig: [u8; 4],
                         input_data: Option<&[u8]>,
                         expect_output: Vec<u8>| {{
                            let mut h = ContractHandler {{
                                call_data: Some(_build_wasm()),
                                ..Default::default()
                            }};

                            h.rt = Some(runtime.clone());

                            match h.execute(sig, input_data, 1_000_000_000_000) {{
                                Ok(r) => assert_eq!((fn_name, r.output_data), (fn_name, expect_output)),
                                Err(e) => {{
                                    panic!("vm error: {{:?}}", e);
                                }}
                            }}
                        }},
                    )
                }}

                #[test]
                fn _compile_test() {{
                    _build_wasm();
                }}

                {}
        "#,
            mod_name,
            item.to_string().trim_start_matches(&prefix)
        )
        .parse()
        .unwrap();
    } else if let Some(cap) = fn_re.captures(&item.to_string()) {
        let fn_name = cap.name("fn_name").unwrap().as_str().to_owned();
        let prefix = format!("fn {}()\n{{", fn_name);
        return format!(
            r#"
            #[test]
            fn {} () {{
                let (_runtime, _run_wasm_fn) = _build_runtime_and_runner();
                let mut _bin: Vec<u8> = Vec::new();

                {}
        "#,
            fn_name,
            item.to_string().trim_start_matches(&prefix)
        )
        .parse()
        .unwrap();
    } else {
        panic!("parse mod or function for testing error")
    }
}
/// helps you assert output from the handle of a contract with `Vec<u8>`.
///
/// ```compile_fail
/// #[ewasm_test]
/// mod tests {
///     use super::*;
///
///     #[ewasm_test]
///     fn test_execute_basic_operations() {
///         ewasm_assert_eq!(handler_fn(), vec![74, 111, 118, 121]);
///     }
/// }
/// ```
#[proc_macro]
pub fn ewasm_assert_eq(item: TokenStream) -> TokenStream {
    let re = Regex::new(r"^(?P<fn_name>[^(]+?)\((?P<params>[^)]*?)\),(?P<equivalence>.*)").unwrap();
    if let Some(cap) = re.captures(&item.to_string().replace("\n", "")) {
        let fn_name = cap.name("fn_name").unwrap().as_str();
        let params = cap.name("params").unwrap().as_str().replace(" ", "");
        let equivalence = cap.name("equivalence").unwrap().as_str();
        if params.is_empty() {
            format!(
                r#"
                    _run_wasm_fn(
                        _runtime.clone(),
                        "{}",
                        ewasm_fn_sig!({}),
                        None,
                        {}
                    );
                "#,
                fn_name, fn_name, equivalence
            )
            .parse()
            .unwrap()
        } else {
            format!(
                r#"
                    _bin = bincode::serialize(&{}).unwrap();
                    _run_wasm_fn(
                        _runtime.clone(),
                        "{}",
                        ewasm_fn_sig!({}),
                        Some(&_bin),
                        {}
                    );
                "#,
                params, fn_name, fn_name, equivalence
            )
            .parse()
            .unwrap()
        }
    } else {
        panic!("fail to parsing function in fn_select");
    }
}

/// helps you assert your handler without error and returns
///
/// ```compile_fail
/// #[ewasm_test]
/// mod tests {
///     use super::*;
///
///     #[ewasm_test]
///     fn test_execute_basic_operations() {
///         ewasm_assert_ok!(contract_fn());
///     }
/// }
/// ```
#[proc_macro]
pub fn ewasm_assert_ok(item: TokenStream) -> TokenStream {
    let re = Regex::new(r"^(?P<fn_name>[^(]+?)\((?P<params>[^)]*?)\)").unwrap();
    if let Some(cap) = re.captures(&item.to_string().replace("\n", "")) {
        let fn_name = cap.name("fn_name").unwrap().as_str();
        let params = cap.name("params").unwrap().as_str().replace(" ", "");
        if params.is_empty() {
            format!(
                r#"
                    _run_wasm_fn(
                        _runtime.clone(),
                        "{}",
                        ewasm_fn_sig!({}),
                        None,
                        Vec::with_capacity(0)
                    );
                "#,
                fn_name, fn_name
            )
            .parse()
            .unwrap()
        } else {
            format!(
                r#"
                    _bin = bincode::serialize(&{}).unwrap();
                    _run_wasm_fn(
                        _runtime.clone(),
                        "{}",
                        ewasm_fn_sig!({}),
                        Some(&_bin),
                        Vec::with_capacity(0)
                    );
                "#,
                params, fn_name, fn_name
            )
            .parse()
            .unwrap()
        }
    } else {
        panic!("fail to parsing function in fn_select");
    }
}

/// helps you assert return Ok(()) your handler with rusty ewasm_main, namely `#[ewasm_main(rusty)]`
///
/// This usage of the macro likes `ewasm_assert_ok`, this only difference is that the contract main
/// function should be decorated with `#[ewasm_main(rusty)]`.
#[proc_macro]
pub fn ewasm_rusty_assert_ok(item: TokenStream) -> TokenStream {
    let re = Regex::new(r"^(?P<fn_name>[^(]+?)\((?P<params>[^)]*?)\)").unwrap();
    if let Some(cap) = re.captures(&item.to_string().replace("\n", "")) {
        let fn_name = cap.name("fn_name").unwrap().as_str();
        let params = cap.name("params").unwrap().as_str().replace(" ", "");
        if params.is_empty() {
            format!(
                r#"
                    _run_wasm_fn(
                        _runtime.clone(),
                        "{}",
                        ewasm_fn_sig!({}),
                        None,
                        vec![0, 0, 0, 0]
                    );
                "#,
                fn_name, fn_name
            )
            .parse()
            .unwrap()
        } else {
            format!(
                r#"
                    _bin = bincode::serialize(&{}).unwrap();
                    _run_wasm_fn(
                        _runtime.clone(),
                        "{}",
                        ewasm_fn_sig!({}),
                        Some(&_bin),
                        vec![0, 0, 0, 0]
                    );
                "#,
                params, fn_name, fn_name
            )
            .parse()
            .unwrap()
        }
    } else {
        panic!("fail to parsing function in fn_select");
    }
}

/// helps you assert return Err your handler with rusty ewasm_main, namely `#[ewasm_main(rusty)]`
///
/// This usage of the macro likes `ewasm_err_output`, the contract main function should be
/// decorated with `#[ewasm_main(rusty)]`.
///
/// You should pass the complete Result type, as the following example
/// `ewasm_rusty_err_output!(Err("NotTrustedInput") as Result<(), &'static str>)`
/// such that you can easy to use any kind of rust error as you like.
#[proc_macro]
pub fn ewasm_rusty_err_output(item: TokenStream) -> TokenStream {
    format!(
        r#"bincode::serialize(&({})).expect("can not serialize the output expected from ewasm").to_vec()"#,
        &item.to_string()
    )
    .parse()
    .unwrap()
}

/// helps you to get the binary result of the thiserror,
///
/// such that you can assert your handler with error.
/// for example:
/// ```compile_fail
/// #[ewasm_test]
/// mod tests {
///    use super::*;
///
///    #[ewasm_test]
///    fn test_execute_basic_operations() {
///        let mut simple_struct = SimpleStruct::default();
///
///        ewasm_assert_eq!(
///            check_input_object(simple_struct),
///            ewasm_err_output!(Error::NotTrustedInput)
///        );
///    }
///}
/// ```
#[proc_macro]
pub fn ewasm_err_output(item: TokenStream) -> TokenStream {
    format!("{}.to_string().as_bytes().to_vec()", &item.to_string())
        .parse()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::*;
    #[test]
    fn test_function_signature() {
        let mut sig: [u8; 4] = hex!("c48d6d5e");
        assert_eq!(get_function_signature("sendMessage(string,address)"), sig);
        sig = hex!("70a08231");
        assert_eq!(get_function_signature("balanceOf(address)"), sig);
    }
}
