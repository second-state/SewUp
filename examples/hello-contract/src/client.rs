/// A example client to interact with kv-contract
use std::str::FromStr;

use anyhow::Result;
use cargo_sewup::config::{get_deploy_config, Deploy};
use sewup_derive::ewasm_input;
use tokio;
use web3::{
    types::{Address, CallRequest},
    Web3,
};

use hello_contract::*;

#[tokio::main]
async fn main() -> Result<()> {
    // NOTE: modify the contract addr after you deploy the contract
    let contract_addr = "0xc2f4023e7c181e4419da30eaaa01816afb23be08";

    let Deploy { url, address, .. } = get_deploy_config().await?;

    let transport = web3::transports::Http::new(&url)?;
    let web3 = Web3::new(transport);

    let input = ewasm_input!(None for hello);

    let call_req = CallRequest {
        from: Some(Address::from_str(&address)?),
        data: Some(input.into()),
        to: Some(Address::from_str(&contract_addr)?),
        ..Default::default()
    };

    let resp = web3.eth().call(call_req, None).await?;
    println!("resp from contract: {}", std::str::from_utf8(&resp.0)?);
    Ok(())
}
