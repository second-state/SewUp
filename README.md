# SewUp

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/second-state/SewUp/CI)
[![Generic badge](https://img.shields.io/badge/sewup-0.0.4-green.svg)](https://crates.io/crates/sewup)
[![Generic badge](https://img.shields.io/badge/SewUpDoc-main-green.svg)](https://second-state.github.io/SewUp/sewup/)
[![Generic badge](https://img.shields.io/badge/sewup_derive-0.0.4-green.svg)](https://crates.io/crates/sewup-derive)
[![Generic badge](https://img.shields.io/badge/SewUpDeriveDoc-main-green.svg)](https://second-state.github.io/SewUp/sewup_derive/)

**S**econdstate **EW**asm **U**tility **P**rogram, a library helps you sew up your Ethereum project with Rust and just like development in a common backend.

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
    Ok("hello world".to_string())
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

Run `cargo build --release --target=wasm32-unknown-unknown`, then the contract will build in `target/wasm32-unknown-unknown/release/*.wasm`
Besides, you also can easily run test with `cargo test`, the ewasm contract automatically test with [WasmEdge](https://github.com/WasmEdge/WasmEdge).
Furthermore, you can learn more from other examples in the [example](./examples/) folder.

## Development
The workspace have several project, the contract project should build with target
`wasm32-unknown-unknown` and the flag `-C link-arg=--export-table`.

You can run `cargo test` in each example to check on the test your modification.
It is easy to participate with help want issues and the good first issues.
