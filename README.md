# SewUp

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/second-state/SewUp/CI)
[![Generic badge](https://img.shields.io/badge/sewup-0.1.1-green.svg)](https://crates.io/crates/sewup)
[![Generic badge](https://img.shields.io/badge/SewUpDoc-main-green.svg)](https://second-state.github.io/SewUp/sewup/)
[![Generic badge](https://img.shields.io/badge/sewup_derive-0.1.1-green.svg)](https://crates.io/crates/sewup-derive)
[![Generic badge](https://img.shields.io/badge/SewUpDeriveDoc-main-green.svg)](https://second-state.github.io/SewUp/sewup_derive/)
[![Generic badge](https://img.shields.io/badge/cargo_sewup-0.1.1-green.svg)](https://crates.io/crates/cargo-sewup)

**S**econdstate **EW**asm **U**tility **P**rogram, a library helps you sew up your Ethereum project with Rust and just like development in a common backend.
There is an [issue](https://github.com/second-state/SewUp/issues/116) on building document on Doc.rs, please kindly use the [document](https://second-state.github.io/SewUp/sewup/) of master instead.
Furthermore, there is also [wiki site](https://github.com/second-state/SewUp/wiki) helps you work with sewup, once you got problems or confusing you can learn more on the wiki.

## Slides & Demo
| Date       | Event                 | Slides / Demo video                                                                                                                                                                            |
|------------|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 2021/09/07 | Version 0.1 release   | [Hello](https://youtu.be/kbe3uuxkBNQ), [KV](https://youtu.be/LUpYIFGG36s), [RDB](https://youtu.be/sJLOcJRheIw), [ERC-20](https://youtu.be/sVGEuNBY1dc), [ERC-721](https://youtu.be/ivZIqnhOAfA), [ERC-1155](https://youtu.be/BsbAFT5rNGw) |
| 2021/06/22 | Rust online meetup    | [v0.0.2](https://slides.com/yanganto/ethereum-wasm-in-rust)                                                                                                                                    |
| 2021/06/19 | Rust meetup (Beijing) | [v0.0.1-pre](https://slides.com/yanganto/sewup)                                                                                                                                                |

## Usage
Add `sewup` with the features and the `sewup-derive` into Cargo.toml, and setup other sections
as following, then you are ready to build contract with sewup.

Features list (should select none or one of following)
- kv - for writing contract as key value database
- rdb - for writing contract as relational database
- token - for writing ERC-20, ERC-721, ERC-1155 tokens

Beside, we suggest you using `anyhow` to handle your result and error, but not limited to,
if you want to use other error crate please checkout `#[ewasm_main(rusty)]` and learn more.
If you want to write a contract return different type of data base on different handlers,
please checkout `#[ewasm_main(auto)]` and `EwasmAny` or the example of rdb feature to learn
how to write a flexible smart contract with ewasm.

### Develop
It is easy to setup your sewup project with `cargo-sewup init`, and you can learn more about the project configure with the [Deploy Guide](https://github.com/second-state/SewUp/wiki/Develop-Guide) wiki page.

### Interact
There are so many clients can interact with contract.

For ERC tokens, we provide `web3js` examples in [wiki page](https://github.com/second-state/SewUp/wiki/ERC-Testing).
The example of clients interacting with contract with [kv](https://github.com/second-state/SewUp/blob/main/examples/kv-contract/src/client.rs) or [rdb](https://github.com/second-state/SewUp/blob/main/examples/rdb-contract/src/client.rs) features.
You can in the example projects for kv and rdb, then `Cargo run` to interact with the contract after modified the contract address.

### Testing
Run `cargo build --release --target=wasm32-unknown-unknown`, then the contract will build in `target/wasm32-unknown-unknown/release/*.wasm`
Besides, you can run deploy the ewasm contract on [WasmEdge](https://github.com/WasmEdge/WasmEdge) and run tests on it with `cargo test`,
furthermore the constructor will also run when the contract deploying on [WasmEdge](https://github.com/WasmEdge/WasmEdge).
If you want to learn more details about the testing flow, please check out [Test the contract](https://github.com/second-state/SewUp/wiki/Develop-Guide#test-the-contract) section of develop guide wiki page.


### Debugging
Furthermore, you can debug your ewasm contract with debug macro `sewup::ewasm_dbg!`, and run the contract with message output by `cargo test -- --nocapture`.
To learn more about the usage, you check out the examples in the [example](./examples/) folder.

### Deployment
Once you want to deploy your contract to any network which support Ewasm by sweup command line tool, please read the [Deploy Guide](https://github.com/second-state/SewUp/wiki/Deploy-Guide) wiki page.

## SewUp Development
There are two projects and several examples in the workspace, the contract project should build with target
`wasm32-unknown-unknown` and the flag `-C link-arg=--export-table`.
You can run `cargo test` in each example folder to check on the test your modification.

It is easy to participate with [help want issues](https://github.com/second-state/SewUp/issues?q=is%3Aopen+is%3Aissue+label%3A%22help+wanted%22) and the [good first issues](https://github.com/second-state/SewUp/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22).
Less but not least, please feel free to open any issue on this project.
