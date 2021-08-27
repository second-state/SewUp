/// A example client to interact with rdb-contract
use std::str::FromStr;

use anyhow::Result;
use reqwest::Client;
use secp256k1::SecretKey;
use serde_derive::{Deserialize, Serialize};
use serde_json::{self, value::Value};
use tokio::{
    self,
    time::{sleep, Duration},
};
use web3::{
    types::{Address, CallRequest, TransactionParameters},
    Web3,
};

#[tokio::main]
async fn main() -> Result<()> {
    // NOTE: modify the contract addr after you deploy the contract
    let contract_addr = "0xe903bc1ef72215a2e6a74b6f1693add99b3afa10";

    let Deploy {
        url,
        private,
        address,
        ..
    } = get_deploy_config().await?;

    let transport = web3::transports::Http::new(&url)?;
    let web3 = Web3::new(transport);
    let prvk = SecretKey::from_str(secret_key)?;

    // let key = 100u32;
    // let value = vec![
    //     1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0,
    // ];
    // let input_pair = Pair(key, value.clone());

    // let mut input = vec![0, 0, 0, 1]; // signature for put_pair_to_bucket1
    // input.append(&mut bincode::serialize(&input_pair).unwrap());

    // let tx_object = TransactionParameters {
    //     data: input.into(),
    //     gas: 5000000.into(),
    //     gas_price: Some(1.into()),
    //     to: Some(Address::from_str(&contract_addr)?),
    //     ..Default::default()
    // };

    // let signed = web3.accounts().sign_transaction(tx_object, &prvk).await?;
    // let r = web3
    //     .eth()
    //     .send_raw_transaction(signed.raw_transaction)
    //     .await?;

    // let mut data_on_chain = false;
    // let mut retry_times = 5;

    // while !data_on_chain && retry_times > 0 {
    //     sleep(Duration::from_millis(6000)).await;

    //     let receipt: serde_json::Value = Client::new()
    //         .post(url)
    //         .json(&serde_json::json!({
    //             "jsonrpc": "2.0",
    //             "method": "eth_getTransactionReceipt",
    //             "params": [format!("{:?}", r)],
    //             "id": 1
    //         }))
    //         .send()
    //         .await?
    //         .json()
    //         .await?;
    //     match receipt {
    //         Value::Object(m) => match m.get("result") {
    //             Some(Value::Object(r)) => match r.get("status") {
    //                 Some(Value::String(code)) if code == "0x1" => {
    //                     data_on_chain = true;
    //                 }
    //                 _ => (),
    //             },
    //             _ => (),
    //         },
    //         _ => (),
    //     };
    //     retry_times -= 1;
    // }

    // if data_on_chain {
    //     println!("success put key value pair on chain: {:?}", r);
    // } else {
    //     eprintln!("fail to put key value pair on chain");
    //     return Ok(());
    // }

    // input = vec![0, 0, 0, 2]; // signature for get_value_to_bucket1
    // input.append(&mut bincode::serialize(&key).unwrap());

    // let call_req = CallRequest {
    //     from: Some(Address::from_str(&address)?),
    //     data: Some(input.into()),
    //     to: Some(Address::from_str(&contract_addr)?),
    //     ..Default::default()
    // };

    // let resp = web3.eth().call(call_req, None).await?;
    // println!("resp: {:?}", resp);
    // let expect_output: Option<Vec<u8>> = Some(value);
    // let expect_bytes = bincode::serialize(&expect_output).unwrap();
    // assert_eq!(resp.0, expect_bytes);
    Ok(())
}
