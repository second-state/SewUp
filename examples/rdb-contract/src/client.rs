/// A example client to interact with rdb-contract
use std::str::FromStr;

use anyhow::Result;
use cargo_sewup::config::{get_deploy_config, Deploy};
use reqwest::Client;
use secp256k1::SecretKey;
use serde_json::{self, value::Value};
use sewup_derive::ewasm_input;
use tokio::{
    self,
    time::{sleep, Duration},
};
use web3::{
    types::{Address, CallRequest, TransactionParameters},
    Web3,
};

// Share struct, table and sig definition in the contract
use rdb_contract::modules::*;

#[tokio::main]
async fn main() -> Result<()> {
    // NOTE: modify the contract addr after you deploy the contract
    let contract_addr = "0x1f903bcebacae5e7187cdf20838272c973360271";

    let Deploy {
        url,
        private,
        address,
        ..
    } = get_deploy_config().await?;

    let transport = web3::transports::Http::new(&url)?;
    let web3 = Web3::new(transport);
    let prvk = SecretKey::from_str(&private)?;

    let person = Person {
        trusted: true,
        age: 18,
    };
    let create_input = person::protocol(person.clone());
    let mut input = ewasm_input!(create_input for person::create);

    let tx_object = TransactionParameters {
        data: input.into(),
        gas: 5000000.into(),
        gas_price: Some(1.into()),
        to: Some(Address::from_str(&contract_addr)?),
        ..Default::default()
    };

    let signed = web3.accounts().sign_transaction(tx_object, &prvk).await?;
    let r = web3
        .eth()
        .send_raw_transaction(signed.raw_transaction)
        .await?;

    let mut data_on_chain = false;
    let mut retry_times = 5;

    while !data_on_chain && retry_times > 0 {
        sleep(Duration::from_millis(6000)).await;

        let receipt: serde_json::Value = Client::new()
            .post(&url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_getTransactionReceipt",
                "params": [format!("{:?}", r)],
                "id": 1
            }))
            .send()
            .await?
            .json()
            .await?;
        match receipt {
            Value::Object(m) => match m.get("result") {
                Some(Value::Object(r)) => match r.get("status") {
                    Some(Value::String(code)) if code == "0x1" => {
                        data_on_chain = true;
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        };
        retry_times -= 1;
    }

    if data_on_chain {
        println!("success put key value pair on chain: {:?}", r);
    } else {
        eprintln!("fail to put key value pair on chain");
        return Ok(());
    }

    let mut get_input: person::Protocol = Person::default().into();
    get_input.set_id(1);

    input = ewasm_input!(get_input for person::get);

    let call_req = CallRequest {
        from: Some(Address::from_str(&address)?),
        data: Some(input.into()),
        to: Some(Address::from_str(&contract_addr)?),
        ..Default::default()
    };

    let resp = web3.eth().call(call_req, None).await?;
    println!("resp: {:?}", resp);
    let mut expect_output = create_input.clone();
    expect_output.set_id(1);
    let expect_bytes = bincode::serialize(&expect_output).unwrap();
    assert_eq!(resp.0, expect_bytes);
    Ok(())
}
