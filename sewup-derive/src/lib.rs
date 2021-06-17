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

#[proc_macro_attribute]
pub fn ewasm_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let re = Regex::new(r"fn (?P<name>[^(]+?)\(").unwrap();
    let fn_name = if let Some(cap) = re.captures(&item.to_string()) {
        cap.name("name").unwrap().as_str().to_owned()
    } else {
        panic!("parse function error")
    };
    format!(
        r#"
        #[no_mangle]
        pub fn main() {{
            use ewasm_api::finish_data;
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
}

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

#[proc_macro]
pub fn fn_sig(item: TokenStream) -> TokenStream {
    format!("_{}_SIG", item.to_string().to_ascii_uppercase())
        .parse()
        .unwrap()
}

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
