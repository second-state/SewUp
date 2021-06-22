# SewUp

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/second-state/SewUp/CI)
[![Generic badge](https://img.shields.io/badge/crate.io-0.0.2-green.svg)](https://crates.io/search?q=sewup)
[![Generic badge](https://img.shields.io/badge/SewUpDoc-main-green.svg)](https://second-state.github.io/SewUp/sewup/)
[![Generic badge](https://img.shields.io/badge/SewUpDeriveDoc-main-green.svg)](https://second-state.github.io/SewUp/sewup_derive/)

**S**econdstate **EW**asm **U**tility **P**rogram, a library to help you sew up your Ethereum project with Rust and just like develop in a common backend.

## Slides
| Date       | Event                 | Slides                                                      |
|------------|-----------------------|-------------------------------------------------------------|
| 2021/06/22 | Rust online meetup    | [v0.0.2](https://slides.com/yanganto/ethereum-wasm-in-rust) |
| 2021/06/19 | Rust meetup (Beijing) | [v0.0.1-pre](https://slides.com/yanganto/sewup)             |

## Usage
Add the `sewup` with the feature and the `sewup-derive` into Cargo.toml, and setup lib section as following, then you are ready to build contract with sewup.
```toml
[lib]
path = "src/lib.rs"
crate-type = ["cdylib"]

[dependencies]
sewup = { version = "0.0.2", features = ['kv'] }
sewup-derive = { version = "0.0.2" }
```
Besides, you can learn more from [examples/kv-contract](./examples/kv-contract),  [examples/default-contract](./examples/default-contract),  [examples/rusty-contract](./examples/rusty-contract) in the examples folder.

## Development
The workspace have several project, the contract project should build with target `wasm32-unknown-unknown` and the flag `-Clink-arg=--export-table`.
After placing the `.wasm` output into [/resources/test](./resources/test), you can run `cargo test -p sewup --features=kv` to check on the test for kv features.
It is easy to participate with help want issues and the good first issues.
