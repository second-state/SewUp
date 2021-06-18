# SewUp

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/second-state/SewUp/CI)
[![Generic badge](https://img.shields.io/badge/Doc-main-green.svg)](https://second-state.github.io/SewUp/sewup/)

**S**econdstate **EW**asm **U**tility **P**rogram, a library to help you sew up your Ethereum project with Rust and just like develop in a common backend.

## Slides
| Date       | Event                 | Slides                                          |
|------------|-----------------------|-------------------------------------------------|
| 2021/06/19 | Rust meetup (Beijing) | [v0.0.1-pre](https://slides.com/yanganto/sewup) |

## Usage
Add the `sewup` with the feature and the `sewup-derive` into Cargo.toml, and setup lib section as following, then you are ready to build contract with sewup.
```toml
[lib]
path = "src/lib.rs"
crate-type = ["cdylib"]

[dependencies]
sewup = { version = "0.0.1", features = ['kv'] }
sewup-derive = { version = "0.0.1" }
```
Besides, you can take [examples/kv-contract](./examples/kv-contract) as an example.

## Development
The workspace have several project, the contract project should build with target `wasm32-unknown-unknown` and the flag `-Clink-arg=--export-table`.
After placing the `.wasm` output into [/resources/test](./resources/test), you can run `cargo test -p sewup --features=kv` to check on the test for kv features.
It is easy to participate with help want issues and the good first issues.
