#![feature(box_into_inner)]
extern crate proc_macro;

#[cfg(test)]
mod function_tests;

use convert_case::{Case::Camel, Casing};
use fancy_regex::Regex;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::quote;
use serde_derive::Deserialize;
use tiny_keccak::{Hasher, Keccak};

type MayString = Option<String>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct AbiIO {
    #[serde(skip_serializing_if = "Option::is_none")]
    internal_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: MayString,
    r#type: String,
}

/// Different Contract mode will treat input and output in different
#[derive(Debug, PartialEq)]
enum ContractMode {
    /// Default mode, only return the error message, if any
    /// This is for a scenario that you just want to modify the data on
    /// chain only
    DefaultMode,
    /// Rusty mode, return serialized binary of `Result<T,E>`, this is good for rust client
    /// This is for a scenario that you are using a rust client to operation the contract
    RustyMode,
    /// Auto mode, return serialize binary for `T` if the Result is Ok, else return the serialize
    /// binary of E
    /// This is for a scenario that you take care the result but not using Rust client
    AutoMode,
}

/// Options can set in ewasm_main function
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
enum ContractOption {
    /// The default message that can be used in Default mode and Auto mode
    DefaultMessage(String),
}

fn parse_contract_mode_and_options(
    attr: String,
) -> Result<(ContractMode, Vec<ContractOption>), &'static str> {
    let (arg_str, option_str) = match attr.split_once(',') {
        Some((head, tail)) => (head, tail),
        None => {
            if attr.starts_with("default") {
                ("", attr.as_str())
            } else {
                (attr.as_str(), "")
            }
        }
    };
    let options = if let Ok(Some(cap)) =
        unsafe { Regex::new(r#"default.*=.*"(?<default>[^"]*)""#).unwrap_unchecked() }
            .captures(option_str)
    {
        let default = unsafe { cap.name("default").unwrap_unchecked() }.as_str();
        vec![ContractOption::DefaultMessage(default.into())]
    } else {
        vec![]
    };
    let output = match arg_str {
        "auto" => (ContractMode::AutoMode, options),
        "rusty" => {
            if options.len() != 0 {
                return Err("can not set default message for rusty mode");
            }
            (ContractMode::RustyMode, options)
        }
        _ => (ContractMode::DefaultMode, options),
    };
    Ok(output)
}

fn get_function_signature(function_prototype: &str) -> [u8; 4] {
    let mut sig = [0; 4];
    let mut hasher = Keccak::v256();
    hasher.update(function_prototype.as_bytes());
    hasher.finalize(&mut sig);
    sig
}

fn write_function_signature(sig_str: &str) -> String {
    let re = unsafe { Regex::new(r"^(?P<name>[^(]+?)\((?P<params>[^)]*?)\)").unwrap_unchecked() };
    if let Ok(Some(cap)) = re.captures(sig_str) {
        let fn_name = unsafe { cap.name("name").unwrap_unchecked() }.as_str();
        let params = unsafe { cap.name("params").unwrap_unchecked() }
            .as_str()
            .replace(" ", "");
        let canonical_fn = format!(
            "{}({})",
            fn_name,
            params
                .split(',')
                .map(|p| {
                    let p_split = p.split(':').collect::<Vec<_>>();
                    if p_split.len() == 2 {
                        p_split[1]
                    } else {
                        p_split[0]
                    }
                    .trim()
                })
                .collect::<Vec<_>>()
                .join(",")
        );
        format!(r"{:?}", get_function_signature(&canonical_fn))
    } else {
        format!(
            "{}_SIG",
            sig_str.to_string().replace(" ", "").to_ascii_uppercase()
        )
    }
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
/// `#[ewasm_main(auto)]`
/// Auto unwrap the output of the result object from ewasm_main function.
/// This is for a scenario that you are using a rust non-rust client,
/// and you are only care the happy case of executing the contract.
///
/// ```compile_fail
/// #[ewasm_main]
/// fn main() -> Result<()> {
///     let contract = Contract::new()?;
///     match contract.get_function_selector()? {
///         ewasm_fn_sig!(check_input_object) => ewasm_input_from!(contract move check_input_object)?,
///         _ => return Err(Error::UnknownHandle.into()),
///     };
///     Ok(())
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn ewasm_main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let name = &input.sig.ident;
    if !input.sig.inputs.is_empty() {
        abort!(
            input.sig.inputs,
            "ewasm_main only wrap the function without inputs"
        )
    }

    let output_type = match input.sig.clone().output {
        syn::ReturnType::Type(_, boxed) => match Box::into_inner(boxed) {
            syn::Type::Path(syn::TypePath { path: p, .. }) => {
                let mut ok_type: Option<String> = None;
                let mut segments = p.segments;
                while let Some(pair) = segments.pop() {
                    ok_type = match pair.into_value() {
                        syn::PathSegment {
                            arguments:
                                syn::PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments { args: a, .. },
                                ),
                            ..
                        } => match a.first() {
                            Some(syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                                path: p,
                                ..
                            }))) => {
                                if let Some(syn::PathSegment { ident: i, .. }) = p.segments.last() {
                                    Some(i.to_string())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        },
                        _ => None,
                    };
                    if ok_type.is_some() {
                        break;
                    }
                }
                ok_type
            }
            _ => None,
        },
        _ => None,
    };

    let (contract_mode, options) = match parse_contract_mode_and_options(attr.to_string()) {
        Ok(o) => o,
        Err(e) => abort_call_site!(e),
    };

    let mut default_message = None;
    for option in options.into_iter() {
        let ContractOption::DefaultMessage(s) = option;
        default_message = Some(s);
    }

    match contract_mode {
        ContractMode::AutoMode if Some("EwasmAny".to_string()) == output_type && default_message.is_some() => quote! {
            #[cfg(target_arch = "wasm32")]
            use sewup::bincode;
            #[cfg(target_arch = "wasm32")]
            use sewup::ewasm_api::finish_data;
            #[cfg(all(not(target_arch = "wasm32"), not(test)))]
            pub fn main() {}
            #[cfg(target_arch = "wasm32")]
            #[cfg(not(any(feature = "constructor", feature = "constructor-test")))]
            #[no_mangle]
            pub fn main() {
                #input
                match #name() {
                    Ok(r) =>  {
                        finish_data(&r.bin);
                    },
                    Err(e) => {
                        finish_data(#default_message.unwrap().as_bytes());
                    }
                }
            }
        },
        ContractMode::AutoMode if Some("EwasmAny".to_string()) == output_type  => quote! {
            #[cfg(target_arch = "wasm32")]
            use sewup::bincode;
            #[cfg(target_arch = "wasm32")]
            use sewup::ewasm_api::finish_data;
            #[cfg(all(not(target_arch = "wasm32"), not(test)))]
            pub fn main() {}
            #[cfg(target_arch = "wasm32")]
            #[cfg(not(any(feature = "constructor", feature = "constructor-test")))]
            #[no_mangle]
            pub fn main() {
                #input
                match #name() {
                    Ok(r) =>  {
                        finish_data(&r.bin);
                    },
                    Err(e) => {
                        let error_msg = e.to_string();
                        finish_data(&error_msg.as_bytes());
                    }
                }
            }
        },
        ContractMode::AutoMode if default_message.is_some() => quote! {
            #[cfg(target_arch = "wasm32")]
            use sewup::bincode;
            #[cfg(target_arch = "wasm32")]
            use sewup::ewasm_api::finish_data;
            #[cfg(all(not(target_arch = "wasm32"), not(test)))]
            pub fn main() {}
            #[cfg(target_arch = "wasm32")]
            #[cfg(not(any(feature = "constructor", feature = "constructor-test")))]
            #[no_mangle]
            pub fn main() {
                #input
                match #name() {
                    Ok(r) =>  {
                        let bin = bincode::serialize(&r).expect("The resuslt of `ewasm_main` should be serializable");
                        finish_data(&bin);
                    },
                    Err(e) => {
                        finish_data(#default_message.unwrap().as_bytes());
                    }
                }
            }
        },
        ContractMode::AutoMode => quote! {
            #[cfg(target_arch = "wasm32")]
            use sewup::bincode;
            #[cfg(target_arch = "wasm32")]
            use sewup::ewasm_api::finish_data;
            #[cfg(all(not(target_arch = "wasm32"), not(test)))]
            pub fn main() {}
            #[cfg(target_arch = "wasm32")]
            #[cfg(not(any(feature = "constructor", feature = "constructor-test")))]
            #[no_mangle]
            pub fn main() {
                #input
                match #name() {
                    Ok(r) =>  {
                        let bin = bincode::serialize(&r).expect("The resuslt of `ewasm_main` should be serializable");
                        finish_data(&bin);
                    },
                    Err(e) => {
                        let error_msg = e.to_string();
                        finish_data(&error_msg.as_bytes());
                    }
                }
            }
        },
        ContractMode::RustyMode if Some("EwasmAny".to_string()) == output_type  => quote! {
            #[cfg(target_arch = "wasm32")]
            use sewup::bincode;
            #[cfg(target_arch = "wasm32")]
            use sewup::ewasm_api::finish_data;
            #[cfg(all(not(target_arch = "wasm32"), not(test)))]
            pub fn main() {}
            #[cfg(target_arch = "wasm32")]
            #[cfg(not(any(feature = "constructor", feature = "constructor-test")))]
            #[no_mangle]
            pub fn main() {
                #input
                let r = #name().map(|any| any.bin);
                let bin = bincode::serialize(&r).expect("The resuslt of `ewasm_main` should be serializable");
                finish_data(&bin);
            }
        },
        ContractMode::RustyMode => quote! {
            #[cfg(target_arch = "wasm32")]
            use sewup::bincode;
            #[cfg(target_arch = "wasm32")]
            use sewup::ewasm_api::finish_data;
            #[cfg(all(not(target_arch = "wasm32"), not(test)))]
            pub fn main() {}
            #[cfg(target_arch = "wasm32")]
            #[cfg(not(any(feature = "constructor", feature = "constructor-test")))]
            #[no_mangle]
            pub fn main() {
                #input
                let r = #name();
                let bin = bincode::serialize(&r).expect("The resuslt of `ewasm_main` should be serializable");
                finish_data(&bin);
            }
        },
        ContractMode::DefaultMode if default_message.is_some() => quote! {
            #[cfg(target_arch = "wasm32")]
            use sewup::bincode;
            #[cfg(target_arch = "wasm32")]
            use sewup::ewasm_api::finish_data;
            #[cfg(all(not(target_arch = "wasm32"), not(test)))]
            pub fn main() {}
            #[cfg(target_arch = "wasm32")]
            #[cfg(not(any(feature = "constructor", feature = "constructor-test")))]
            #[no_mangle]
            pub fn main() {
                #input
                if let Err(e) = #name() {
                    finish_data(#default_message.unwrap().as_bytes());
                }
            }
        },
        ContractMode::DefaultMode => quote! {
            #[cfg(target_arch = "wasm32")]
            use sewup::bincode;
            #[cfg(target_arch = "wasm32")]
            use sewup::ewasm_api::finish_data;
            #[cfg(all(not(target_arch = "wasm32"), not(test)))]
            pub fn main() {}
            #[cfg(target_arch = "wasm32")]
            #[cfg(not(any(feature = "constructor", feature = "constructor-test")))]
            #[no_mangle]
            pub fn main() {
                #input
                if let Err(e) = #name() {
                    let error_msg = e.to_string();
                    finish_data(&error_msg.as_bytes());
                }
            }
        }
    }.into()
}

fn parse_fn_attr(
    fn_name: String,
    attr: String,
) -> Result<(Option<String>, String, Option<String>), &'static str> {
    let attr_str = attr.replace(" ", "").replace("\n", "");
    return if attr_str.is_empty() {
        Ok((None, "{}".into(), None))
    } else if let Some((head, tail)) = attr_str.split_once(',') {
        if tail.is_empty() {
            Ok((Some(head.replace("\"", "")), "{}".into(), None))
        } else {
            let root_str = if let Ok(Some(cap)) =
                unsafe { Regex::new(r"only_by=(?P<account>[^,]*)").unwrap_unchecked() }
                    .captures(&attr_str)
            {
                Some(
                    unsafe { cap.name("account").unwrap_unchecked() }
                        .as_str()
                        .into(),
                )
            } else {
                None
            };

            let mut json = "{".to_string();
            if let Ok(Some(cap)) =
                unsafe { Regex::new(r"constant=(?P<constant>[^,]*)").unwrap_unchecked() }
                    .captures(&attr_str)
            {
                match unsafe { cap.name("constant").unwrap_unchecked() }.as_str() {
                    "true" => json.push_str(r#""constant":true,"#),
                    "false" => json.push_str(r#""constant":false,"#),
                    _ => return Err("constacnt should be true or false"),
                }
            } else {
                json.push_str(r#""constant":false,"#)
            }

            if let Ok(Some(cap)) =
                unsafe { Regex::new(r#"inputs=(?<inputs>\[.*?\])(?!")"#).unwrap_unchecked() }
                    .captures(&attr_str)
            {
                json.push_str(r#""inputs":"#);
                let inputs_str = unsafe { cap.name("inputs").unwrap_unchecked() }.as_str();
                for cap in Regex::new(r"\{[^\{\}]*\}").unwrap().find_iter(inputs_str) {
                    if let Ok(input_cap) = cap {
                        if serde_json::from_str::<AbiIO>(input_cap.as_str()).is_err() {
                            return Err("inputs are not valid format");
                        }
                    }
                }
                json.push_str(inputs_str);
                json.push(',');
            } else {
                json.push_str(r#""inputs":[],"#);
            }

            if let Ok(Some(cap)) = unsafe { Regex::new(r"name=(?P<name>[^,]*)").unwrap_unchecked() }
                .captures(&attr_str)
            {
                json.push_str(r#""name":""#);
                json.push_str(unsafe { cap.name("name").unwrap_unchecked() }.as_str());
                json.push_str(r#"","#);
            } else {
                json.push_str(&format!(r#""name":"{}","#, fn_name.to_case(Camel)));
            }

            if let Ok(Some(cap)) =
                unsafe { Regex::new(r#"outputs=(?<outputs>\[.*?\])(?!")"#).unwrap_unchecked() }
                    .captures(&attr_str)
            {
                json.push_str(r#""outputs":"#);
                let outputs_str = unsafe { cap.name("outputs").unwrap_unchecked() }.as_str();
                for cap in Regex::new(r"\{[^\{\}]*\}").unwrap().find_iter(outputs_str) {
                    if let Ok(output_cap) = cap {
                        if serde_json::from_str::<AbiIO>(output_cap.as_str()).is_err() {
                            return Err("outputs are not valid format");
                        }
                    }
                }
                json.push_str(outputs_str);
                json.push(',');
            } else {
                json.push_str(r#""outputs":[],"#);
            }

            if let Ok(Some(cap)) =
                unsafe { Regex::new(r"payable=(?P<payable>[^,]*)").unwrap_unchecked() }
                    .captures(&attr_str)
            {
                match unsafe { cap.name("payable").unwrap_unchecked() }.as_str() {
                    "true" => json.push_str(r#""payable":true,"#),
                    "false" => json.push_str(r#""payable":false,"#),
                    _ => return Err("payable should be true or false"),
                }
            } else {
                json.push_str(r#""payable":false,"#);
            }

            if let Ok(Some(cap)) = unsafe {
                Regex::new(r"stateMutability=(?P<stateMutability>[^,]*)").unwrap_unchecked()
            }
            .captures(&attr_str)
            {
                match unsafe { cap.name("stateMutability").unwrap_unchecked() }.as_str() {
                    "nonpayable" => json.push_str(r#""stateMutability":"nonpayable","#),
                    "view" => json.push_str(r#""stateMutability":"view","#),
                    _ => return Err("stateMutability should be nonpayable or view"),
                }
            } else {
                json.push_str(r#""stateMutability":"view","#);
            }

            json.push_str(r#""type":"function"}"#);
            if head.starts_with("only_by") {
                Ok((None, json, root_str))
            } else {
                Ok((Some(head.replace("\"", "")), json, root_str))
            }
        }
    } else if let Ok(Some(cap)) =
        unsafe { Regex::new(r"only_by=(?P<account>[^,]*)").unwrap_unchecked() }.captures(&attr_str)
    {
        Ok((
            None,
            "{}".into(),
            Some(
                unsafe { cap.name("account").unwrap_unchecked() }
                    .as_str()
                    .into(),
            ),
        ))
    } else {
        Ok((Some(attr_str.replace("\"", "")), "{}".into(), None))
    };
}

/// helps you to build your handlers in the contract
///
/// This macro also generate the function signature, you can use
/// `ewasm_fn_sig!` macro to get your function signature;
///
/// ```compile_fail
/// #[ewasm_fn]
/// fn check_input_object(s: SimpleStruct) -> anyhow::Result<()> {
///     Ok(())
/// }
///
/// #[ewasm_main]
/// fn main() -> Result<()> {
///     let contract = Contract::new()?;
///     match contract.get_function_selector()? {
///         ewasm_fn_sig!(check_input_object) => ewasm_input_from!(contract move check_input_object)?,
///         _ => return Err(Error::UnknownHandle.into()),
///     };
///     Ok(())
/// }
/// ```
///
/// There are two kind of inputs for `ewasm_fn` macro, first is functional signature, second and
/// more are the fields of abijson.
/// The functional signature can be specific as following ways.
/// `#[ewasm_fn(00fdd58e)]` or #[ewasm_fn("00fdd58e")]
///
/// ```compile_fail
/// #[ewasm_fn(00fdd58e,
/// constant=true,
/// inputs=[
///     { "internalType": "address", "name": "account", "type": "address" },
///     { "internalType": "uint256", "name": "token_id", "type": "uint256" }
/// ],
/// name=balanceOf,
/// outputs=[
///     { "internalType": "uint256", "name": "", "type": "uint256" }
/// ],
/// payable=false,
/// stateMutability=view
/// )]
/// ```
/// The fields are not required, it can use default value if not provided.
/// The default values of `constant`, `payable` are `false`; the default values of `inputs` and
/// `outputs` are `[]`; the default value of `stateMutability` is `view`; the default name is the
/// camel case style of the function name.
///
/// The handler also can be restricted by called special account with `only_by` attribute,
/// following are examples:
/// ```compile_fail
/// #[ewasm_fn(only_by=8663DBF0cC68AaF37fC8BA262F2df4c666a41993)]
/// #[ewasm_fn(only_by="0x8663DBF0cC68AaF37fC8BA262F2df4c666a41993")]
/// #[ewasm_fn(only_by=0x8663DBF0cC68AaF37fC8BA262F2df4c666a41993)]
/// #[ewasm_fn(only_by="8663DBF0cC68AaF37fC8BA262F2df4c666a41993")]
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn ewasm_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = syn::parse_macro_input!(item as syn::ItemFn);
    let syn::Block { stmts, .. } = *block;

    let name = &sig.ident;

    let (hex_str, abi_str, root_str) = match parse_fn_attr(name.to_string(), attr.to_string()) {
        Ok(o) => o,
        Err(e) => abort_call_site!(e),
    };

    let args = &sig
        .inputs
        .iter()
        .map(|fn_arg| match fn_arg {
            syn::FnArg::Receiver(r) => {
                abort!(r, "please use ewasm_fn for function not method")
            }
            syn::FnArg::Typed(p) => Box::into_inner(p.ty.clone()),
        })
        .map(|ty| match ty {
            syn::Type::Path(tp) => (
                tp.path
                    .segments
                    .first()
                    .expect("at least one segment")
                    .ident
                    .clone(),
                false,
            ),
            syn::Type::Reference(tr) => match Box::into_inner(tr.elem) {
                syn::Type::Path(tp) => (
                    tp.path
                        .segments
                        .first()
                        .expect("at least one segment")
                        .ident
                        .clone(),
                    true,
                ),
                _ => abort_call_site!("please pass Path type or Reference type to ewasm_fn_sig"),
            },
            _ => abort_call_site!("please pass Path type or Reference type to ewasm_fn_sig"),
        })
        .map(|(ident, is_ref)| {
            if is_ref {
                format!("&{}", ident).to_ascii_lowercase()
            } else {
                format!("{}", ident).to_ascii_lowercase()
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    let canonical_fn = format!("{}({})", name, args);
    let (sig_0, sig_1, sig_2, sig_3) = if let Some(hex_str) = hex_str {
        let fn_sig = hex::decode(hex_str).expect("function signature is not correct");
        (fn_sig[0], fn_sig[1], fn_sig[2], fn_sig[3])
    } else {
        let fn_sig = get_function_signature(&canonical_fn);
        (fn_sig[0], fn_sig[1], fn_sig[2], fn_sig[3])
    };
    let abi_info = Ident::new(
        &format!("{}_ABI", name.to_string().to_ascii_uppercase()),
        Span::call_site(),
    );
    let sig_name = Ident::new(
        &format!("{}_SIG", name.to_string().to_ascii_uppercase()),
        Span::call_site(),
    );
    let result = if let Some(root_str) = root_str {
        let addr = root_str.replace("\"", "");
        quote! {
            pub const #sig_name : [u8; 4] = [#sig_0, #sig_1, #sig_2, #sig_3];
            pub(crate) const #abi_info: &'static str = #abi_str;

            #[cfg(target_arch = "wasm32")]
            #[cfg(not(any(feature = "constructor", feature = "constructor-test")))]
            #(#attrs)*
            #vis #sig {
                if sewup::utils::caller() != sewup::types::Address::from_str(#addr)? {
                    return Err(sewup::errors::HandlerError::Unauthorized.into())
                }
                #(#stmts)*
            }
        }
    } else {
        quote! {
            pub const #sig_name : [u8; 4] = [#sig_0, #sig_1, #sig_2, #sig_3];
            pub(crate) const #abi_info: &'static str = #abi_str;

            #[cfg(target_arch = "wasm32")]
            #[cfg(not(any(feature = "constructor", feature = "constructor-test")))]
            #(#attrs)*
            #vis #sig {
                #(#stmts)*
            }
        }
    };
    result.into()
}

/// helps you to build your constructor for the contract
#[proc_macro_error]
#[proc_macro_attribute]
pub fn ewasm_constructor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(item as syn::ItemFn);
    let default_name = Ident::new("__constructor", Span::call_site());
    if input.sig.ident != "__constructor" {
        input.sig.ident = default_name;
    }
    let result = quote! {
        #[cfg(target_arch = "wasm32")]
        #[cfg(any(feature = "constructor", feature = "constructor-test"))]
        #[no_mangle]
        #input

        #[cfg(target_arch = "wasm32")]
        #[cfg(feature = "constructor-test")]
        #[no_mangle]
        pub fn main() {
            __constructor();
        }
    };
    result.into()
}

/// helps you to build your handler in other module
///
/// This macro will automatically generated as `{FUNCTION_NAME}_SIG`
///
/// ```compile_fail
/// // module.rs
///
/// use sewup::ewasm_api;
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
///
/// There are two kind of inputs for `ewasm_fn` macro, first is functional signature, second and
/// more are the fields of abijson.
/// The functional signature can be specific as following ways.
/// `#[ewasm_lib_fn(00fdd58e)]` or #[ewasm_lib_fn("00fdd58e")]
///
/// ``` #[ewasm_lib_fn(00fdd58e,
/// constant=true,
/// inputs=[
///     { "internalType": "address", "name": "account", "type": "address" },
///     { "internalType": "uint256", "name": "token_id", "type": "uint256" }
/// ],
/// name=balanceOf,
/// outputs=[
///     { "internalType": "uint256", "name": "", "type": "uint256" }
/// ],
/// payable=false,
/// stateMutability=view
/// )]
/// ```
/// The fields are not required, it can use default value if not provided.
/// The default values of `constant`, `payable` are `false`; the default values of `inputs` and
/// `outputs` are `[]`; the default value of `stateMutability` is `view`; the default name is the
/// camel case style of the function name.
///
#[proc_macro_error]
#[proc_macro_attribute]
pub fn ewasm_lib_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let name = &input.sig.ident;

    let (hex_str, abi_str, _root_str) = match parse_fn_attr(name.to_string(), attr.to_string()) {
        Ok(o) => o,
        Err(e) => abort_call_site!(e),
    };

    let inputs = &input.sig.inputs;

    let (sig_0, sig_1, sig_2, sig_3) = if let Some(hex_str) = hex_str {
        let fn_sig = hex::decode(hex_str).expect("function signature is not correct");
        (fn_sig[0], fn_sig[1], fn_sig[2], fn_sig[3])
    } else {
        let args = &inputs
            .iter()
            .map(|fn_arg| match fn_arg {
                syn::FnArg::Receiver(r) => {
                    abort!(r, "please use ewasm_fn for function not method")
                }
                syn::FnArg::Typed(p) => Box::into_inner(p.ty.clone()),
            })
            .map(|ty| match ty {
                syn::Type::Path(tp) => (
                    tp.path
                        .segments
                        .first()
                        .expect("at least one segment")
                        .ident
                        .clone(),
                    false,
                ),
                syn::Type::Reference(tr) => match Box::into_inner(tr.elem) {
                    syn::Type::Path(tp) => (
                        tp.path
                            .segments
                            .first()
                            .expect("at least one segment")
                            .ident
                            .clone(),
                        true,
                    ),
                    _ => {
                        abort_call_site!("please pass Path type or Reference type to ewasm_fn_sig")
                    }
                },
                _ => abort_call_site!("please pass Path type or Reference type to ewasm_fn_sig"),
            })
            .map(|(ident, is_ref)| {
                if is_ref {
                    format!("&{}", ident).to_ascii_lowercase()
                } else {
                    format!("{}", ident).to_ascii_lowercase()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        let canonical_fn = format!("{}({})", name, args);
        let fn_sig = get_function_signature(&canonical_fn);
        (fn_sig[0], fn_sig[1], fn_sig[2], fn_sig[3])
    };
    let sig_name = Ident::new(
        &format!("{}_SIG", name.to_string().to_ascii_uppercase()),
        Span::call_site(),
    );
    let abi_info = Ident::new(
        &format!("{}_ABI", name.to_string().to_ascii_uppercase()),
        Span::call_site(),
    );
    let result = quote! {
        pub const #sig_name: [u8; 4] = [#sig_0, #sig_1, #sig_2, #sig_3];
        pub const #abi_info: &'static str = #abi_str;

        #[cfg(not(target_arch = "wasm32"))]
        #[allow(unused)]
        pub fn #name(#inputs) {}

        #[cfg(target_arch = "wasm32")]
        #input
    };
    result.into()
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
///         ewasm_fn_sig!(decorated_handler) => ewasm_input_from!(contract move decorated_handler)?,
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
///             => ewasm_input_from!(contract move undecorated_handler)?,
///         _ => return Err(Error::UnknownHandle.into()),
///     };
///     Ok(())
/// }
/// ```
///
#[proc_macro_error]
#[proc_macro]
pub fn ewasm_fn_sig(item: TokenStream) -> TokenStream {
    write_function_signature(&item.to_string())
        .parse()
        .expect("can not generate function signature")
}

/// helps you generate the input raw data for specific contract handler
/// ```compile_fail
/// let create_input = person::protocol(person.clone());
/// let mut input = ewasm_input!(create_input for person::create);
/// ```
#[proc_macro]
pub fn ewasm_input(item: TokenStream) -> TokenStream {
    let re = unsafe { Regex::new(r"(?P<instance>.*)\s+for\s+(?P<sig>.*)").unwrap_unchecked() };
    if let Ok(Some(cap)) = re.captures(&item.to_string()) {
        let sig = unsafe { cap.name("sig").unwrap_unchecked() }.as_str();
        let instance = unsafe { cap.name("instance").unwrap_unchecked() }.as_str();
        let output = if instance == "None" {
            format!("{}.to_vec()", write_function_signature(sig),)
        } else {
            format!(
                "{{
                let mut input = {}.to_vec();
                input.append(&mut bincode::serialize(&{}).unwrap());
                input
            }}",
                write_function_signature(sig),
                instance
            )
        };
        output.parse().expect("ewasm input transform unexpected")
    } else {
        panic!("ewasm_input input incorrect")
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
///         ewasm_fn_sig!(check_input_object) => ewasm_input_from!(contract move check_input_object)?,
///         _ => return Err(Error::UnknownHandle.into()),
///     };
///  Ok(())
///  }
/// ```
///
/// Besides, you can map the error to your customized error when something wrong happened in
/// `ewasm_input_from!`, for example:
/// `ewasm_input_from!(contract move check_input_object, |_| Err("DeserdeError"))`
/// ```compile_fail
/// #[ewasm_main(rusty)]
/// fn main() -> Result<(), &'static str> {
///     let contract = Contract::new().map_err(|_| "NewContractError")?;
///     match contract.get_function_selector().map_err(|_| "FailGetFnSelector")? {
///         ewasm_fn_sig!(check_input_object) =>  ewasm_input_from!(contract move check_input_object, |_| "DeserdeError")?
///         _ => return Err("UnknownHandle"),
///     };
///     Ok(())
/// }
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn ewasm_input_from(item: TokenStream) -> TokenStream {
    let re = unsafe {
        Regex::new(r"^(?P<contract>\w+)\s+move\s+(?P<name>[^,]+),?(?P<error_handler>.*)")
            .unwrap_unchecked()
    };
    if let Ok(Some(cap)) = re.captures(&item.to_string()) {
        let contract = Ident::new(
            unsafe { cap.name("contract").unwrap_unchecked() }.as_str(),
            Span::call_site(),
        );
        let name_result: syn::Result<syn::ExprPath> =
            syn::parse_str(unsafe { cap.name("name").unwrap_unchecked() }.as_str());
        let name = if let Ok(name) = name_result {
            name
        } else {
            abort_call_site!(
                "`{}` is not an ExprPath",
                unsafe { cap.name("name").unwrap_unchecked() }.as_str()
            );
        };
        let error_handler = unsafe { cap.name("error_handler").unwrap_unchecked() }.as_str();
        return if error_handler.is_empty() {
            quote! {
                #name(sewup::bincode::deserialize(&#contract.input_data[4..])
                      .map_err(|e| anyhow::anyhow!("contract input deserialize error: {}", e))?
                )
            }
        } else {
            let closure: syn::Result<syn::ExprClosure> = syn::parse_str(error_handler);
            if let Ok(closure) = closure {
                quote! {
                    #name(sewup::bincode::deserialize(&#contract.input_data[4..]).map_err(#closure)?)
                }
            } else {
                abort_call_site!("`{}` is not an closure input for map_err", error_handler);
            }
        }
        .into();
    } else {
        abort_call_site!(
            r#"fail to parsing ewasm_input_from,
            please use
                `ewasm_input_from( contract move handler )
            or
                `ewasm_input_from( contract move handler, closure_for_map_err)`
            "#
        );
    }
}

/// help you generate the exactly contract output form rust instance
#[proc_macro]
pub fn ewasm_output_from(item: TokenStream) -> TokenStream {
    format!(
        r#"sewup::bincode::serialize(&{}).expect("fail to serialize in `ewasm_output_from`")"#,
        item,
    )
    .parse()
    .expect("ewasm output format unexpected")
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
#[cfg(feature = "kv")]
#[proc_macro_derive(Key)]
pub fn derive_key(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let sturct_name = &input.ident;
    return quote! {
        #[cfg(target_arch = "wasm32")]
        impl sewup::kv::traits::Key for #sturct_name {}
    }
    .into();
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
#[cfg(feature = "kv")]
#[proc_macro_derive(Value)]
pub fn derive_value(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let sturct_name = &input.ident;
    return quote! {
        #[cfg(target_arch = "wasm32")]
        impl sewup::kv::traits::Value for #sturct_name {}
    }
    .into();
}

/// provides the handers for CRUD and the Protocol struct to communicate with these handlers.
///
/// ```compile_fail
/// use sewup_derive::Table;
/// #[derive(Table)]
/// struct Person {
///     trusted: bool,
///     age: u8,
/// }
/// ```
///
/// The crud handlers are generated as `{struct_name}::get`, `{struct_name}::create`,
/// `{struct_name}::update`, `{struct_name}::delete`, you can easily used these handlers as
/// following example.
///
/// ```compile_fail
/// #[ewasm_main]
/// fn main() -> Result<()> {
///     let mut contract = Contract::new()?;
///
///     match contract.get_function_selector()? {
///         ewasm_fn_sig!(person::get) => ewasm_input_from!(contract move person::get)?,
///         ewasm_fn_sig!(person::create) => ewasm_input_from!(contract move person::create)?,
///         ewasm_fn_sig!(person::update) => ewasm_input_from!(contract move person::update)?,
///         ewasm_fn_sig!(person::delete) => ewasm_input_from!(contract move person::delete)?,
///         _ => return Err(RDBError::UnknownHandle.into()),
///     }
///
///     Ok(())
/// }
/// ```
///
/// The protocol is the input and also the output format of these handlers, besides these handlers
/// are easy to build by the `{struct_name}::protocol`, `{struct_name}::Protocol`, and use `set_id`
/// for specify the record you want to modify.
/// for examples.
///
/// ```compile_fail
/// let handler_input = person::protocol(person);
/// let mut default_person_input: person::Protocol = Person::default().into();
/// default_input.set_id(2);
/// ```
///
/// you can use `ewasm_output_from!` to get the exactly input/output binary of the protol, for
/// example:
/// ```compile_fail
/// let handler_input = person::protocol(person);
/// ewasm_output_from!(handler_input)
/// ```
///
/// Please note that the protocol default and the protocol for default instance may be different.
/// This base on the implementation of the default trait of the structure.
///
/// ```compile_fail
/// let default_input = person::Protocol::default();
/// let default_person_input: person::Protocol = Person::default().into();
/// assert!(default_input != default_person_input)
/// ```
///
/// The relation can be set with `#[belongs_to(Person)]` or `#[belongs_none_or(Person)]`
/// ```compile_fail
/// #[derive(Table, Default, Clone, PartialEq, Serialize, Deserialize)]
/// #[belongs_none_or(Location)]
/// pub struct Person {
///     pub trusted: bool,
///     pub age: u8,
///     pub location_id: Option<usize>,
/// }
///
/// #[derive(Table, Default, Clone, PartialEq, Serialize, Deserialize)]
/// #[belongs_to(Person)]
/// pub struct Post {
///     pub content: SizedString!(50),
///
///     // Currently, this field need to set up manually, this will be enhance later
///     pub person_id: usize,
/// }
///
/// #[derive(Table, Default, Clone, PartialEq, Serialize, Deserialize)]
/// pub struct Location {
///     pub address: Raw,
/// }
/// ```
/// After the directive setup, you can use `table_name.another_table_name` to get the related
/// instance, for example
/// ```compile_fail
/// let person: Person = post.person()?;
/// let home: Option<Location> = person.location()?;
/// ```
#[cfg(feature = "rdb")]
#[proc_macro_derive(Table, attributes(belongs_to, belongs_none_or))]
pub fn derive_table(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let attrs = &input.attrs;
    let mut belongs_to: Option<String> = None;
    let mut belongs_none_or: Option<String> = None;
    for a in attrs.iter() {
        let syn::Attribute { path, tokens, .. } = a;
        let attr_name = path.segments.first().map(|s| s.ident.to_string());
        if Some("belongs_to".to_string()) == attr_name {
            belongs_to = Some(
                tokens
                    .to_string()
                    .strip_prefix('(')
                    .expect("#[belongs_to(table_name)] is not correct")
                    .strip_suffix(')')
                    .expect("#[belongs_to(table_name)] is not correct")
                    .to_string(),
            );
        }
        if Some("belongs_none_or".to_string()) == attr_name {
            belongs_none_or = Some(
                tokens
                    .to_string()
                    .strip_prefix('(')
                    .expect("#[belongs_none_or(table_name)] is not correct")
                    .strip_suffix(')')
                    .expect("#[belongs_none_or(table_name)] is not correct")
                    .to_string(),
            );
        }
    }
    let struct_name = &input.ident;
    let fields_with_type = match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(f),
            ..
        }) => f
            .clone()
            .named
            .into_pairs()
            .map(|p| p.into_value())
            .map(|f| (f.ident.unwrap(), f.ty))
            .collect::<Vec<_>>(),
        _ => abort!(&input.ident, "Table derive only use for struct"),
    };

    let mut wrapper_fields = vec![(
        Ident::new("id", Span::call_site()),
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::parse("Option<usize>".parse().unwrap()).unwrap(),
        }),
    )];
    wrapper_fields.append(
        &mut fields_with_type
            .iter()
            .map(|(f, t)| {
                (
                    f.clone(),
                    syn::parse(quote!(Option<#t>).to_string().parse().unwrap()).unwrap(),
                )
            })
            .collect::<Vec<_>>(),
    );
    let wrapper_field_names = wrapper_fields.iter().map(|(f, _)| f);
    let wrapper_field_types = wrapper_fields.iter().map(|(_, t)| t);
    let field_names = fields_with_type.iter().map(|(f, _)| f);
    let clone_field_names = field_names.clone();
    let clone_field_names2 = field_names.clone();
    let clone_field_names3 = field_names.clone();
    let clone_field_names4 = field_names.clone();
    let clone_field_names5 = field_names.clone();
    let field_types = fields_with_type.iter().map(|(_, t)| t);

    let protocol_name = Ident::new(&format!("{}Protocol", struct_name), Span::call_site());
    let wrapper_name = Ident::new(&format!("{}Wrapper", struct_name), Span::call_site());
    let captal_name = Ident::new(
        &format!("{}", struct_name).to_ascii_uppercase(),
        Span::call_site(),
    );
    let lower_name = Ident::new(
        &format!("{}", struct_name).to_ascii_lowercase(),
        Span::call_site(),
    );
    let mut output = quote!(
        impl sewup::rdb::traits::Record for #struct_name {}

        #[cfg_attr(any(feature = "debug", test), derive(Debug))]
        #[derive(Clone, sewup::Serialize, sewup::Deserialize)]
        pub struct #protocol_name {
            pub select_fields: Option<std::collections::HashSet::<String>>,
            pub filter: bool,
            pub records: Vec<#wrapper_name>
        }

        impl #protocol_name {
            pub fn set_select_fields(&mut self, fields: Vec<String>) {
                if fields.is_empty() {
                    self.select_fields = None;
                } else {
                    let mut select_fields = std::collections::HashSet::<String>::new();
                    for field in fields.iter() {
                        select_fields.insert(field.into());
                    }
                    self.select_fields = Some(select_fields);
                }
            }
        }

        impl Default for #protocol_name {
            fn default() -> Self {
                Self {
                    select_fields: None,
                    filter: false,
                    records: vec![Default::default()]
                }
            }
        }

        impl From<#struct_name> for #protocol_name {
            fn from(instance: #struct_name) -> Self {
                Self {
                    select_fields: None,
                    filter: false,
                    records: vec![instance.into()]
                }
            }
        }

        impl From<(usize, #struct_name)> for #protocol_name {
            fn from(instance: (usize, #struct_name)) -> Self {
                Self {
                    select_fields: None,
                    filter: false,
                    records: vec![instance.into()]
                }
            }
        }

        impl From<Vec<#struct_name>> for #protocol_name {
            fn from(instances: Vec<#struct_name>) -> Self {
                Self {
                    select_fields: None,
                    filter: false,
                    records: instances.into_iter().map(|i| i.into()).collect::<Vec<_>>()
                }
            }
        }

        impl From<Vec<(usize, #struct_name)>> for #protocol_name {
            fn from(instances: Vec<(usize, #struct_name)>) -> Self {
                Self {
                    select_fields: None,
                    filter: false,
                    records: instances.into_iter().map(|i| i.into()).collect::<Vec<_>>()
                }
            }
        }

        impl From<Vec<#wrapper_name>> for #protocol_name {
            fn from(records: Vec<#wrapper_name>) -> Self {
                Self {
                    select_fields: None,
                    filter: false,
                    records,
                }
            }
        }

        pub mod #captal_name {
            use sewup_derive::ewasm_fn_sig;
            pub const GET_SIG: [u8; 4] = ewasm_fn_sig!(#struct_name::get());
            pub const CREATE_SIG: [u8; 4] = ewasm_fn_sig!(#struct_name::create());
            pub const UPDATE_SIG: [u8; 4] = ewasm_fn_sig!(#struct_name::update());
            pub const DELETE_SIG: [u8; 4] = ewasm_fn_sig!(#struct_name::delete());
        }

        #[cfg_attr(any(feature = "debug", test), derive(Debug))]
        #[derive(Default, Clone, sewup::Serialize, sewup::Deserialize)]
        pub struct #wrapper_name {
            #(pub #wrapper_field_names: #wrapper_field_types,)*
        }

        impl From<#struct_name> for #wrapper_name {
            fn from(instance: #struct_name) -> Self {
                Self {
                    id: None,
                    #(#field_names: Some(instance.#field_names),)*
                }
            }
        }

        impl From<(usize, #struct_name)> for #wrapper_name {
            fn from(t: (usize, #struct_name)) -> Self {
                Self {
                    id: Some(t.0),
                    #(#clone_field_names5: Some(t.1.#clone_field_names5),)*
                }
            }
        }

        impl From<#wrapper_name> for #struct_name {
            fn from(wrapper: #wrapper_name) -> Self {
                Self {
                    #(#clone_field_names: wrapper.#clone_field_names.expect("#clone_field_names field missing"),)*
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        pub mod #lower_name {
            use super::*;
            pub type Protocol = #protocol_name;
            pub type Wrapper = #wrapper_name;
            pub type _InstanceType = #struct_name;
            pub fn get(proc: Protocol) -> sewup::Result<sewup::primitives::EwasmAny> {
                let table = sewup::rdb::Db::load(None)?.table::<_InstanceType>()?;
                if proc.filter {
                    let mut raw_output: Vec<Wrapper> = Vec::new();
                    for (idx, r) in table.all_records()?.drain(..).enumerate(){
                        let mut all_field_match = true;
                        #(
                            paste::paste! {
                                let [<#clone_field_names2 _filed_filter>] : Option<#field_types> =
                                    sewup::utils::get_field_by_name(proc.records[0].clone(), stringify!(#clone_field_names2));

                                let  [<#clone_field_names2 _field>] : #field_types =
                                    sewup::utils::get_field_by_name(&r, stringify!(#clone_field_names2));

                                if [<#clone_field_names2 _filed_filter>].is_some() {
                                    all_field_match &=
                                        [<#clone_field_names2 _filed_filter>].unwrap()
                                            == [<#clone_field_names2 _field>];
                                }
                            }
                         )*

                        if all_field_match {
                            raw_output.push((idx + 1, r).into());
                        }
                    }
                    if let Some(select_fields) = proc.select_fields {
                        for w in raw_output.iter_mut() {
                            #(
                                if (! select_fields.contains(stringify!(#clone_field_names3)))
                                    && stringify!(#clone_field_names3) != "id" {
                                    w.#clone_field_names3 = None;
                                }
                            )*
                        }
                    }
                    let p: #protocol_name = raw_output.into();
                    Ok(p.into())
                } else {
                    let raw_output = table.get_record(proc.records[0].id.unwrap_or_default())?;
                    let mut output_proc: Protocol = raw_output.into();
                    output_proc.records[0].id = proc.records[0].id;
                    Ok(output_proc.into())
                }
            }
            pub fn create(proc: Protocol) -> sewup::Result<sewup::primitives::EwasmAny> {
                let mut table = sewup::rdb::Db::load(None)?.table::<_InstanceType>()?;
                let mut output_proc = proc.clone();
                output_proc.records[0].id = Some(table.add_record(proc.records[0].clone().into())?);
                table.commit()?;
                Ok(output_proc.into())
            }
            pub fn update(proc: Protocol) -> sewup::Result<sewup::primitives::EwasmAny> {
                let mut table = sewup::rdb::Db::load(None)?.table::<_InstanceType>()?;
                let id = proc.records[0].id.unwrap_or_default();
                table.update_record(id, Some(proc.records[0].clone().into()))?;
                table.commit()?;
                Ok(proc.into())
            }
            pub fn delete(proc: Protocol) -> sewup::Result<sewup::primitives::EwasmAny> {
                let mut table = sewup::rdb::Db::load(None)?.table::<_InstanceType>()?;
                let id = proc.records[0].id.unwrap_or_default();
                table.update_record(id, None)?;
                table.commit()?;
                Ok(proc.into())
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        pub mod #lower_name {
            use super::*;
            pub type Protocol = #protocol_name;
            pub type Wrapper = #wrapper_name;
            pub type _InstanceType = #struct_name;
            pub type Query = Wrapper;

            #[inline]
            pub fn protocol(instance: _InstanceType) -> Protocol {
                instance.into()
            }
            impl Protocol {
                pub fn set_id(&mut self, id: usize) {
                    self.records[0].id = Some(id);
                }
                pub fn is_empty(&self) -> bool {
                    #(self.records[0].#clone_field_names4.is_none() && )*
                    true
                }
            }
            pub fn query(instance: _InstanceType) -> Wrapper {
                instance.into()
            }

            impl From<Query> for Protocol {
                fn from(instance: Query) -> Self {
                    Self {
                        select_fields: None,
                        filter: true,
                        records: vec![instance.into()]
                    }
                }
            }
        }
    ).to_string();

    if let Some(parent_table) = belongs_to {
        let lower_parent_table = &format!("{}", &parent_table).to_ascii_lowercase();
        let parent_table = Ident::new(&parent_table, Span::call_site());
        let lower_parent_table_ident = Ident::new(&lower_parent_table, Span::call_site());
        let field_name = &format!("{}_id", lower_parent_table);

        output += &quote! {
            impl #struct_name {
                pub fn #lower_parent_table_ident (&self) -> sewup::Result<#parent_table> {
                    let id: usize = sewup::utils::get_field_by_name(self, #field_name);
                    let parent_table = sewup::rdb::Db::load(None)?.table::<#parent_table>()?;
                    parent_table.get_record(id)
                }
            }
        }
        .to_string();
    }

    if let Some(parent_table) = belongs_none_or {
        let lower_parent_table = &format!("{}", &parent_table).to_ascii_lowercase();
        let parent_table = Ident::new(&parent_table, Span::call_site());
        let lower_parent_table_ident = Ident::new(&lower_parent_table, Span::call_site());
        let field_name = &format!("{}_id", lower_parent_table);

        output += &quote! {
            impl #struct_name {
                pub fn #lower_parent_table_ident (&self) -> sewup::Result<Option<#parent_table>> {
                    let id: Option<usize> = sewup::utils::get_field_by_name(self, #field_name);
                    let parent_table = sewup::rdb::Db::load(None)?.table::<#parent_table>()?;
                    if let Some(id) = id {
                        match parent_table.get_record(id) {
                            Ok(r) => Ok(Some(r)),
                            Err(e) => Err(e)
                        }
                    } else {
                        Ok(None)
                    }
                }
            }
        }
        .to_string();
    }

    output.parse().unwrap()
}

/// helps you setup the test mododule, and test cases in contract.
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
/// The test runtime will be create in the module, and all the test case will use the same test
/// runtime, if you can create more runtimes for testing by setup more test modules.
/// You can setup a log file when running the test as following, then use `sewup::ewasm_dbg!` to debug the
/// ewasm contract in the executing in the runtime.
/// ```compile_fail
/// #[ewasm_test(log=/path/to/logfile)]
/// mod tests {
///     use super::*;
///
///     #[ewasm_test]
///     fn test_execute_basic_operations() {
///         ewasm_assert_ok!(contract_fn());
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn ewasm_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mod_re = unsafe {
        Regex::new(r"^mod (?P<mod_name>[^\{\s]*)(?P<to_first_bracket>[^\{]*\{)").unwrap_unchecked()
    };
    let fn_re = unsafe {
        Regex::new(r"^fn (?P<fn_name>[^\(\s]*)(?P<to_first_bracket>[^\{]*\{)").unwrap_unchecked()
    };
    let context = item.to_string();
    let mod_captures = mod_re.captures(&context).unwrap();
    let fn_captures = fn_re.captures(&context).unwrap();
    if mod_captures.is_some() {
        let attr_str = attr.to_string().replace(" ", "");
        let runtime_log_option = if attr_str.is_empty() {
            "".to_string()
        } else {
            let options = attr_str.split('=').collect::<Vec<_>>();
            match options[0].to_lowercase().as_str() {
                "log" => format!(".set_log_file({:?}.into())", options[1]),
                _ => abort_call_site!("no support option"),
            }
        };
        let template = r#"
            #[cfg(test)]
            mod $mod_name {
                use sewup::bincode;
                use sewup::runtimes::{handler::ContractHandler, test::TestRuntime};
                use std::cell::RefCell;
                use std::path::Path;
                use std::path::PathBuf;
                use std::process::Command;
                use std::sync::Arc;

                fn _build_wasm(opt: Option<String>) -> String {
                    let cargo_cmd = format!("cargo build --release --target=wasm32-unknown-unknown {}", opt.unwrap_or_default());
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(&cargo_cmd)
                        .output()
                        .expect("failed to build wasm binary");
                    if !output.status.success() {
                        panic!("return code not success: fail to build wasm binary")
                    }
                    let pkg_name = env!("CARGO_PKG_NAME");
                    let base_dir = env!("CARGO_MANIFEST_DIR");
                    let wasm_binary = format!(
                        "{}/target/wasm32-unknown-unknown/release/{}.wasm",
                        base_dir,
                        pkg_name.replace("-", "_")
                    );

                    if !Path::new(&wasm_binary).exists() {
                        panic!("wasm binary missing")
                    }
                    wasm_binary
                }

                fn _build_runtime_and_runner() -> (
                    Arc<RefCell<TestRuntime>>,
                    impl Fn(Arc<RefCell<TestRuntime>>, Option<&str>, &str, [u8; 4], Option<&[u8]>, Vec<u8>) -> (),
                ) {
                    let rt = Arc::new(RefCell::new(TestRuntime::default()"#.to_string()
                            + &runtime_log_option
                            + r#"));
                    let mut h = ContractHandler {
                        call_data: None,
                        rt: Some(rt.clone())
                    };

                    match h.run_fn(_build_wasm(Some("--features=constructor-test".to_string())), None, 1_000_000_000_000) {
                        Ok(_) => (),
                        Err(e) => {
                            panic!("vm run constructor error: {:?}", e);
                        }
                    };

                    (rt,
                        |runtime: Arc<RefCell<TestRuntime>>,
                        caller: Option<&str>,
                        fn_name: &str,
                        sig: [u8; 4],
                        input_data: Option<&[u8]>,
                        expect_output: Vec<u8>| {
                            let mut h = ContractHandler {
                                call_data: Some(_build_wasm(None)),
                                rt: Some(runtime.clone())
                            };

                            match h.execute(caller.clone(), sig, input_data, 1_000_000_000_000) {
                                Ok(r) => {
                                    if !(*r.output_data == *expect_output) {
                                        if let Some(caller) = caller {
                                            eprintln!("vm caller : {}", caller);
                                        }

                                        if let Ok(output_msg) = std::str::from_utf8(&r.output_data) {
                                            eprintln!("vm msg    : \"{}\"", output_msg);
                                        }
                                        eprintln!("vm output : {:?}", r.output_data);

                                        if let Ok(expect_msg) = std::str::from_utf8(&expect_output) {
                                            eprintln!("expected  : \"{}\"", expect_msg);
                                        }
                                        eprintln!("expected  : {:?}", expect_output);
                                        panic!("function `{}` output is unexpected", fn_name);
                                    }
                                },
                                Err(e) => {
                                    panic!("vm error: {:?}", e);
                                }
                            }
                        },
                    )
                }

                #[test]
                fn _compile_runtime_test() {
                    _build_wasm(None);
                }

                #[test]
                fn _compile_constructor_test() {
                    _build_wasm(Some("--features=constructor-test".to_string()));
                }"#;
        return mod_re
            .replace(&context, &template)
            .to_string()
            .parse()
            .unwrap();
    } else if fn_captures.is_some() {
        let attr_str = attr.to_string().replace(" ", "");
        if !attr_str.is_empty() {
            abort_call_site!("no support option when wrapping on function")
        };
        return fn_re
            .replace(
                &context,
                r#"
            #[test]
            fn $fn_name () {
                let (_runtime, _run_wasm_fn) = _build_runtime_and_runner();
                let mut _bin: Vec<u8> = Vec::new();"#,
            )
            .to_string()
            .parse()
            .unwrap();
    } else {
        abort_call_site!("parse mod or function for testing error")
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
///
/// Besides, you can run the handler as a block chan user with `by` syntax
/// ```compile_fail
/// ewasm_assert_eq!(handler_fn() by "eD5897cCEa7aee785D31cdcA87Cf59D1D041aAFC", vec![74, 111, 118, 121]);
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn ewasm_assert_eq(item: TokenStream) -> TokenStream {
    let re = unsafe {
        Regex::new(r#"^(?P<fn_name>[^(]+?)\((?P<params>[^)]*?)\)\s*(by)?\s*(?P<caller>"[^"]*")?\s*,(?P<equivalence>.*)"#).unwrap_unchecked()
    };
    if let Ok(Some(cap)) = re.captures(&item.to_string().replace("\n", "")) {
        let fn_name = unsafe { cap.name("fn_name").unwrap_unchecked() }
            .as_str()
            .replace(" ", "");
        let params = unsafe { cap.name("params").unwrap_unchecked() }
            .as_str()
            .replace(" ", "");
        let equivalence = unsafe { cap.name("equivalence").unwrap_unchecked() }.as_str();
        let caller = cap
            .name("caller")
            .map(|c| format!("Some({})", c.as_str()))
            .unwrap_or_else(|| "None".to_string());
        if params.is_empty() {
            format!(
                r#"_run_wasm_fn( _runtime.clone(), {}, "{}", ewasm_fn_sig!({}), None, {});"#,
                caller, fn_name, fn_name, equivalence
            )
            .parse()
            .unwrap()
        } else {
            format!(
                r#"_bin = bincode::serialize(&{}).unwrap();
                   _run_wasm_fn( _runtime.clone(), {}, "{}", ewasm_fn_sig!({}), Some(&_bin), {});"#,
                params, caller, fn_name, fn_name, equivalence
            )
            .parse()
            .unwrap()
        }
    } else {
        abort_call_site!("fail to parsing function in ewasm_assert_eq");
    }
}

/// helps you assert return instance from your handler with auto unwrap ewasm_main, namely `#[ewasm_main(auto)]`
///
/// This usage of the macro likes `ewasm_assert_eq`, but the contract main function should be
/// decorated with `#[ewasm_main(auto)]`, and the equivalence arm will be serialized into `Vec<u8>`
/// Besides, you can run the handler as a block chan user with `by` syntax as the same usage of `ewasm_assert_eq`.
#[proc_macro_error]
#[proc_macro]
pub fn ewasm_auto_assert_eq(item: TokenStream) -> TokenStream {
    let re = unsafe {
        Regex::new(r#"^(?P<fn_name>[^(]+?)\((?P<params>[^)]*?)\)\s*(by)?\s*(?P<caller>"[^"]*")?\s*,(?P<equivalence>.*)"#).unwrap_unchecked()
    };
    if let Ok(Some(cap)) = re.captures(&item.to_string().replace("\n", "")) {
        let fn_name = unsafe { cap.name("fn_name").unwrap_unchecked() }.as_str();
        let params = unsafe { cap.name("params").unwrap_unchecked() }
            .as_str()
            .replace(" ", "");
        let equivalence = unsafe { cap.name("equivalence").unwrap_unchecked() }.as_str();
        let caller = cap
            .name("caller")
            .map(|c| format!("Some({})", c.as_str()))
            .unwrap_or_else(|| "None".to_string());
        if params.is_empty() {
            format!(
                r#"_run_wasm_fn( _runtime.clone(), {}, "{}", ewasm_fn_sig!({}), None, sewup_derive::ewasm_output_from!({}));"#,
                caller, fn_name, fn_name, equivalence
            )
            .parse()
            .unwrap()
        } else {
            format!(
                r#"_bin = bincode::serialize(&{}).unwrap();
                   _run_wasm_fn( _runtime.clone(), {}, "{}", ewasm_fn_sig!({}), Some(&_bin), sewup_derive::ewasm_output_from!({}));"#,
                params, caller, fn_name, fn_name, equivalence
            )
            .parse()
            .unwrap()
        }
    } else {
        abort_call_site!("fail to parsing function in fn_select");
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
///
/// Besides, you can run the handler as a block chan user with `by` syntax.
/// ```compile_fail
/// ewasm_assert_ok!(contract_fn() by "eD5897cCEa7aee785D31cdcA87Cf59D1D041aAFC");
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn ewasm_assert_ok(item: TokenStream) -> TokenStream {
    let re = unsafe {
        Regex::new(
            r#"^(?P<fn_name>[^(]+?)\((?P<params>[^)]*?)\)\s*(by)?\s*(?P<caller>"[^"]*")?\s*"#,
        )
        .unwrap_unchecked()
    };
    if let Ok(Some(cap)) = re.captures(&item.to_string().replace("\n", "")) {
        let fn_name = unsafe { cap.name("fn_name").unwrap_unchecked() }.as_str();
        let params = unsafe { cap.name("params").unwrap_unchecked() }
            .as_str()
            .replace(" ", "");
        let caller = cap
            .name("caller")
            .map(|c| format!("Some({})", c.as_str()))
            .unwrap_or_else(|| "None".to_string());
        if params.is_empty() {
            format!(
                r#"_run_wasm_fn( _runtime.clone(), {}, "{}", ewasm_fn_sig!({}), None, Vec::with_capacity(0));"#,
                caller, fn_name, fn_name
            )
            .parse()
            .unwrap()
        } else {
            format!(
                r#"_bin = bincode::serialize(&{}).unwrap();
                   _run_wasm_fn( _runtime.clone(), {}, "{}", ewasm_fn_sig!({}), Some(&_bin), Vec::with_capacity(0));"#,
                params, caller, fn_name, fn_name
            )
            .parse()
            .unwrap()
        }
    } else {
        abort_call_site!("fail to parsing function in fn_select");
    }
}

/// helps you assert return Ok(()) your handler with rusty ewasm_main, namely `#[ewasm_main(rusty)]`
///
/// This usage of the macro likes `ewasm_assert_ok`, this only difference is that the contract main
/// function should be decorated with `#[ewasm_main(rusty)]`.
/// Besides, you can run the handler as a block chan user with `by` syntax as the same usage of `ewasm_assert_ok`.
#[proc_macro_error]
#[proc_macro]
pub fn ewasm_rusty_assert_ok(item: TokenStream) -> TokenStream {
    let re = unsafe {
        Regex::new(
            r#"^(?P<fn_name>[^(]+?)\((?P<params>[^)]*?)\)\s*(by)?\s*(?P<caller>"[^"]*")?\s*"#,
        )
        .unwrap_unchecked()
    };
    if let Ok(Some(cap)) = re.captures(&item.to_string().replace("\n", "")) {
        let fn_name = unsafe { cap.name("fn_name").unwrap_unchecked() }.as_str();
        let params = unsafe { cap.name("params").unwrap_unchecked() }
            .as_str()
            .replace(" ", "");
        let caller = cap
            .name("caller")
            .map(|c| format!("Some({})", c.as_str()))
            .unwrap_or_else(|| "None".to_string());
        if params.is_empty() {
            format!(
                r#"_run_wasm_fn( _runtime.clone(), {}, "{}", ewasm_fn_sig!({}), None, vec![0, 0, 0, 0]);"#,
                caller, fn_name, fn_name
            )
            .parse()
            .unwrap()
        } else {
            format!(
                r#"_bin = bincode::serialize(&{}).unwrap();
                   _run_wasm_fn( _runtime.clone(), {}, "{}", ewasm_fn_sig!({}), Some(&_bin), vec![0, 0, 0, 0]);"#,
                params, caller, fn_name, fn_name
            )
            .parse()
            .unwrap()
        }
    } else {
        abort_call_site!("fail to parsing function in fn_select");
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
#[proc_macro_error]
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
#[proc_macro_error]
#[proc_macro]
pub fn ewasm_err_output(item: TokenStream) -> TokenStream {
    format!("{}.to_string().as_bytes().to_vec()", &item.to_string())
        .parse()
        .unwrap()
}

/// help you write the field which storage string no longer than the specific size
///```compile_fail
///#[derive(Table)]
///pub struct Blog {
///     pub content: SizedString!(50),
///}
///```
#[allow(non_snake_case)]
#[proc_macro_error]
#[proc_macro]
pub fn SizedString(item: TokenStream) -> TokenStream {
    let num = item.to_string();

    if let Ok(num) = num.trim().parse::<usize>() {
        if num > 0 {
            let raw_size = num / 32usize + 1;
            return format!("[sewup::types::Raw; {}]", raw_size)
                .parse()
                .unwrap();
        }
    }
    panic!("The input of SizedString! should be a greator than zero integer")
}

/// helps you return handler when caller is not in access control list
/// ```compile_fail
/// ewasm_call_only_by!("8663..1993")
/// ```
#[proc_macro]
pub fn ewasm_call_only_by(item: TokenStream) -> TokenStream {
    let input = item.to_string().replace(" ", "");
    let output = if input.starts_with('"') {
        let addr = input.replace("\"", "");
        quote! {
            if sewup::utils::caller() != sewup::types::Address::from_str(#addr)? {
                return Err(sewup::errors::HandlerError::Unauthorized.into())
            }
        }
    } else {
        let addr = Ident::new(&input, Span::call_site());
        quote! {
            if sewup::utils::caller() != sewup::types::Address::from_str(#addr)? {
                return Err(sewup::errors::HandlerError::Unauthorized.into())
            }
        }
    };

    output.into()
}
