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

/// `ewasm_main` is a macro for the main function of the contract
/// There are three different contract output.
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
            use sewup::bincode;
            use ewasm_api::finish_data;
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
            use sewup::bincode;
            use ewasm_api::finish_data;
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
            use ewasm_api::finish_data;
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

/// The macro helps you to build your handler in the contract, and also
/// generate the function signature, you can use `fn_sig!` macro to get your
/// function signature of the function wrappered with `#[ewasm_fn]`
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
            pub(crate) const _{}_SIG: [u8; 4] = {:?};
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

/// The macro helps you to build your handler as a lib, which can used in the
/// contract, the function signature well automatically generated as
/// `{FUNCTION_NAME}_SIG`
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
            {}
        "#,
            fn_name,
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

/// `fn_sig` helps you get you function signature
/// 1. provide function name to get function signature from the same namespace,
/// which function should be decorated with `#[ewasm_fn]`, for example,
/// `fn_sig!(the_name_of_contract_handler)`
///
/// 2. provide a function name with input parameters then the macro will
/// calculate the correct functional signature for you.
/// ex: `fn_sig!(the_name_of_contract_handler( a: i32, b: String ))`
#[proc_macro]
pub fn fn_sig(item: TokenStream) -> TokenStream {
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
        format!("_{}_SIG", item.to_string().to_ascii_uppercase())
            .parse()
            .unwrap()
    }
}

/// `input_from` will help you to get the input data from contract caller, and
/// automatically deserialize input into handler
/// `input_from!(contract, the_name_of_the_handler)`
/// Besides, you can map the error to your customized error when something wrong happened in
/// `input_from!`, for example:
/// `input_from!(contract, check_input_object, |_| Err("DeserdeError"))`
#[proc_macro]
pub fn input_from(item: TokenStream) -> TokenStream {
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
/// `Value` derive help you implement Value trait for kv feature
#[proc_macro_derive(Value)]
pub fn derive_value(item: TokenStream) -> TokenStream {
    let re = Regex::new(r"struct (?P<name>\w+)").unwrap();
    if let Some(cap) = re.captures(&item.to_string()) {
        let struct_name = cap.name("name").unwrap().as_str();
        format!(
            r#"
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

/// `Key` derive help you implement Key trait for the kv feature
#[proc_macro_derive(Key)]
pub fn derive_key(item: TokenStream) -> TokenStream {
    let re = Regex::new(r"struct (?P<name>\w+)").unwrap();
    if let Some(cap) = re.captures(&item.to_string()) {
        let struct_name = cap.name("name").unwrap().as_str();
        format!(
            r#"
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
