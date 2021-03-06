use std::str::FromStr;

use anyhow::{Context, Result};
use reqwest::Client;
use secp256k1::SecretKey;
use serde_json::{self, value::Value};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::time::{sleep, Duration};
use web3::{types::TransactionParameters, Web3};

use cargo_sewup::config::{get_deploy_config, Deploy};
use cargo_sewup::constants::{DEFAULT_GAS, DEFAULT_GAS_PRICE};
use cargo_sewup::deploy_wasm;

pub async fn run(contract_name: String, verbose: bool, debug: bool) -> Result<()> {
    let config = get_deploy_config().await?;
    if verbose {
        println!("{}", config);
    }
    let Deploy {
        url,
        private,
        gas,
        gas_price,
        ..
    } = config;
    let transport = web3::transports::Http::new(&url)?;
    let web3 = Web3::new(transport);

    let prvk = SecretKey::from_str(&private)?;

    let wasm_path = format!(deploy_wasm!(), contract_name);

    let mut file = File::open(&wasm_path)
        .await
        .context("Can not open .deploy.wasm")?;
    let mut contents = vec![];
    file.read_to_end(&mut contents)
        .await
        .context("Fail to read .deploy.wasm")?;

    let tx_object = TransactionParameters {
        data: contents.into(),
        gas: gas.unwrap_or(DEFAULT_GAS).into(),
        gas_price: Some(gas_price.unwrap_or(DEFAULT_GAS_PRICE).into()),
        ..Default::default()
    };

    let signed = web3.accounts().sign_transaction(tx_object, &prvk).await?;
    let result = web3
        .eth()
        .send_raw_transaction(signed.raw_transaction)
        .await?;

    if verbose {
        println!("contract deploy with hash: {:?}", &result);
    }
    let mut contract_address: Option<String> = None;
    let mut retry_times = 5;
    while contract_address.is_none() && retry_times > 0 {
        sleep(Duration::from_millis(6000)).await;
        let receipt: serde_json::Value = Client::new()
            .post(&url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_getTransactionReceipt",
                "params": [format!("{:?}", result)],
                "id": 1
            }))
            .send()
            .await?
            .json()
            .await?;

        if debug {
            println!(
                "==> Try get receipt in {} time:\n {:?}",
                retry_times, receipt
            );
        }

        contract_address = match receipt {
            Value::Object(m) => match m.get("result") {
                Some(Value::Object(r)) => match (r.get("contractAddress"), r.get("status")) {
                    (Some(Value::String(addr)), Some(Value::String(code))) => {
                        if code == "0x1" {
                            Some(addr.to_owned())
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };
        retry_times -= 1;
    }
    if let Some(contract_address) = contract_address {
        println!("contract address: {}", contract_address);
    } else {
        println!("contract deploy fail");
    }
    Ok(())
}
