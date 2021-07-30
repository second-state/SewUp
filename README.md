# SewUp

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/second-state/SewUp/CI)
[![Generic badge](https://img.shields.io/badge/sewup-0.0.6-green.svg)](https://crates.io/crates/sewup)
[![Generic badge](https://img.shields.io/badge/SewUpDoc-main-green.svg)](https://second-state.github.io/SewUp/sewup/)
[![Generic badge](https://img.shields.io/badge/sewup_derive-0.0.6-green.svg)](https://crates.io/crates/sewup-derive)
[![Generic badge](https://img.shields.io/badge/SewUpDeriveDoc-main-green.svg)](https://second-state.github.io/SewUp/sewup_derive/)
[![Generic badge](https://img.shields.io/badge/cargo-sewup-0.0.2-green.svg)](https://crates.io/crates/cargo-sewup)

**S**econdstate **EW**asm **U**tility **P**rogram, a library helps you sew up your Ethereum project with Rust and just like development in a common backend.
There is an [issue](https://github.com/second-state/SewUp/issues/116) on building document on Doc.rs, please kindly use the [document](https://second-state.github.io/SewUp/sewup/) of master instead.

## Slides
| Date       | Event                 | Slides                                                      |
|------------|-----------------------|-------------------------------------------------------------|
| 2021/06/22 | Rust online meetup    | [v0.0.2](https://slides.com/yanganto/ethereum-wasm-in-rust) |
| 2021/06/19 | Rust meetup (Beijing) | [v0.0.1-pre](https://slides.com/yanganto/sewup)             |

## Usage
Add `sewup` with the features and the `sewup-derive` into Cargo.toml, and setup other sections
as following, then you are ready to build contract with sewup.

Features list (should select none or one of following)
- kv - for writing contract as key value database
- rdb - for writing contract as relational database

Beside, we suggest you using `anyhow` to handle your result and error, but not limited to,
if you want to use other error crate please checkout `#[ewasm_main(rusty)]` and learn more.
If you want to write a contract return different type of data base on different handlers,
please checkout `#[ewasm_main(auto)]` and `EwasmAny` or the example of rdb feature to learn
how to write a flexible smart contract with ewasm.

```toml
[package]
name = "hello-contract"

[lib]
path = "src/lib.rs"
crate-type = ["cdylib"]

[dependencies]
sewup = { version = "*", features = ['kv'] }
sewup-derive = { version = "*", features = ['kv']  }

anyhow = "1.0"

[profile.release]
incremental = false
panic = "abort"
lto = true
opt-level = "z"

[profile.release.package.hello-contract]  # package name
incremental = false
opt-level = "z"
```
Place [.cargo/config](./examples/hello-contract/.cargo/config) file in your project to specify the flags for build.

Here is minimize example for writing a contract with sewup
```rust
// lib.rs
use anyhow::Result;

use sewup::primitives::Contract;
use sewup_derive::{ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[ewasm_fn]
fn hello() -> Result<String> {
    let target = "world";
    let greeting = "hello ".to_string() + sewup::ewasm_dbg!(target);
    Ok(greeting)
}

#[ewasm_main(auto)]
fn main() -> Result<String> {
    let contract = Contract::new()?;
    let greeting = match contract.get_function_selector()? {
        ewasm_fn_sig!(hello) => hello()?,
        _ => panic!("unknown handle"),
    };
    Ok(greeting)
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::ewasm_auto_assert_eq;

    #[ewasm_test]
    fn test_get_greeting() {
        ewasm_auto_assert_eq!(hello(), "hello world".to_string());
    }
}
```

### Testing
Run `cargo build --release --target=wasm32-unknown-unknown`, then the contract will build in `target/wasm32-unknown-unknown/release/*.wasm`
Besides, you can run deploy the ewasm contract on [WasmEdge](https://github.com/WasmEdge/WasmEdge) and run tests on it with `cargo test`.

### Debugging
Furthermore, you can debug your ewasm contract with debug macro `sewup::ewasm_dbg!`, and run the contract with message output by `cargo test -- --nocapture`.
To learn more about the usage, you check out the examples in the [example](./examples/) folder.

### Deployment
Once you want to deploy your contract to any network which support Ewasm by sweup command line tool, please read the [wiki](https://github.com/second-state/SewUp/wiki/Deploy-Guide).

## SewUp Development
There are two projects and several examples in the workspace, the contract project should build with target
`wasm32-unknown-unknown` and the flag `-C link-arg=--export-table`.
You can run `cargo test` in each example folder to check on the test your modification.

It is easy to participate with help want issues and the good first issues.
Less but not least, please feel free to open any issue on this project.
